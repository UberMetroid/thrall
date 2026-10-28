//! THRALL (`st-mesh-mgr`) — Multi-Server Syncthing LDAP Control Plane & Self-Service Portal.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod device_store;
pub mod fleet_config;
pub mod ldap_bind;
pub mod portal_server;
pub mod reconcile_mesh;
pub mod render_portal;
pub mod ssh_syncthing;

use std::env;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{self, Child, Command};
use std::time::Duration;

use crate::device_store::MeshStore;
use crate::fleet_config::load_or_create_fleet_config;
use crate::portal_server::handle_http_client;
use crate::reconcile_mesh::reconcile_fleet;

fn maybe_spawn_local_syncthing() -> Option<Child> {
    if env::var("THRALL_LOCAL_SYNCTHING").unwrap_or_default() != "1" {
        return None;
    }
    let bin = ["/bin/syncthing", "/usr/bin/syncthing"]
        .into_iter()
        .find(|p| Path::new(p).exists())?;

    Command::new(bin)
        .args([
            "serve",
            "--no-browser",
            "--no-upgrade",
            "--home=/config/syncthing",
            "--gui-address=127.0.0.1:8384",
        ])
        .spawn()
        .ok()
}

fn run_healthcheck(listen_addr: &str) -> i32 {
    let port = listen_addr
        .rsplit_once(':')
        .map(|(_, p)| p)
        .unwrap_or("8080");
    let target = format!("127.0.0.1:{port}");
    match TcpStream::connect_timeout(
        &target
            .parse()
            .unwrap_or_else(|_| std::net::SocketAddr::from(([127, 0, 0, 1], 8080))),
        Duration::from_secs(3),
    ) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("[thrall] healthcheck failed against {target}: {e}");
            1
        }
    }
}

#[rustfmt::skip]
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut config_path = env::var("THRALL_CONFIG").unwrap_or_else(|_| "/config/fleet.yaml".to_string());
    let (mut check_only, mut reconcile_only, mut i) = (false, false, 1);
    while i < args.len() {
        match args[i].as_str() {
            "--config" | "-c" => if let Some(p) = args.get(i + 1) { config_path = p.clone(); i += 1; },
            "--check" => check_only = true,
            "--reconcile-once" => reconcile_only = true,
            _ => {}
        }
        i += 1;
    }
    let cfg = load_or_create_fleet_config(&config_path).unwrap_or_else(|e| {
        eprintln!("[thrall] configuration failure ({config_path}): {e}");
        process::exit(78);
    });
    if check_only { process::exit(run_healthcheck(&cfg.portal.listen_addr)); }
    let mut local_st = maybe_spawn_local_syncthing();
    let mut store = MeshStore::load(&cfg.portal.database_path);
    eprintln!("[thrall] Loaded state from {} ({} users)", cfg.portal.database_path, store.users.len());
    let snapshot = reconcile_fleet(&cfg, &mut store);
    let online_count = snapshot.nodes.iter().filter(|n| n.online).count();
    eprintln!("[thrall] Initial fleet reconciliation complete: {online_count}/{} server nodes online", snapshot.nodes.len());
    if reconcile_only {
        if let Some(ref mut child) = local_st { let _ = child.kill(); }
        return;
    }
    let listener = TcpListener::bind(&cfg.portal.listen_addr).unwrap_or_else(|e| {
        eprintln!("[thrall] Failed to bind portal on {}: {e}", cfg.portal.listen_addr);
        process::exit(78);
    });
    eprintln!("[thrall] Portal listening on http://{}", cfg.portal.listen_addr);
    let state = std::sync::Arc::new(std::sync::Mutex::new((cfg, store, snapshot)));
    let bg_state = std::sync::Arc::clone(&state);
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(45));
        let Some((c, mut st)) = bg_state.lock().ok().map(|g| (g.0.clone(), g.1.clone())) else { continue };
        let snap = reconcile_fleet(&c, &mut st);
        if let Ok(mut g) = bg_state.lock() { g.2 = snap; }
    });
    for stream in listener.incoming().flatten() {
        if let Ok(mut g) = state.lock() {
            let (ref mut c, ref mut st, ref mut sn) = *g;
            handle_http_client(stream, c, st, sn);
        }
    }
    if let Some(ref mut child) = local_st { let _ = child.kill(); }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::device_store::{
        compute_share_token, normalize_and_verify_device_id, resolve_jailed_path,
        resolve_share_target, sanitize_folder_name, sanitize_rel_subpath, sanitize_username,
        MeshStore,
    };
    use crate::reconcile_mesh::FleetSnapshot;
    use crate::render_portal::{render_dashboard_page, render_login_page};

    #[test]
    #[rustfmt::skip]
    fn test_tabs_security_pending_folders_and_shares() {
        assert!(sanitize_rel_subpath("../etc/passwd").is_err());
        assert!(sanitize_rel_subpath(".ssh/id_rsa").is_err());
        assert_eq!(
            sanitize_rel_subpath("Documents/Notes").unwrap(),
            "Documents/Notes"
        );
        assert_eq!(
            sanitize_folder_name("Project Alpha").unwrap(),
            "Project-Alpha"
        );
        assert!(sanitize_folder_name(".config").is_err());

        let messy_id = "  device_id: aaaaaaa aaaaaaa\u{200b}-bbbbbbb_bbbbbbn ccccccc-cccccc2-ddddddd-ddddddh  ";
        let norm_id = normalize_and_verify_device_id(messy_id).unwrap();
        assert_eq!(
            norm_id,
            "AAAAAAA-AAAAAAA-BBBBBBB-BBBBBBN-CCCCCCC-CCCCCC2-DDDDDDD-DDDDDDH"
        );

        let mut store = MeshStore::default();
        store.upsert_device("alice", messy_id, "").unwrap();
        assert_eq!(store.users["alice"][0].label, "dev-1-aaaaaaa");
        assert!(store.folders_for_user("alice").is_empty());
        store.ensure_user_folders("alice").push("Documents".into());

        store.user_folders.insert("emptyuser".into(), Vec::new());
        let roundtrip = MeshStore::parse(&store.serialize());
        assert!(roundtrip.folders_for_user("emptyuser").is_empty());
        assert!(resolve_jailed_path("emptyuser", &roundtrip, "personal", "").is_none());

        store
            .shared_folders
            .entry("Ops".into())
            .or_default()
            .insert("alice".into());
        store
            .shared_folders
            .entry("Ops".into())
            .or_default()
            .insert("bob".into());
        assert!(resolve_jailed_path("alice", &store, "shared:Ops", "Reports").is_some());
        assert!(resolve_jailed_path("charlie", &store, "shared:Ops", "Reports").is_none());
        assert!(resolve_jailed_path("alice", &store, "personal:Documents", "Notes").is_some());
        assert!(resolve_jailed_path("alice", &store, "personal:UnknownFolder", "").is_none());

        let tok = compute_share_token("personal:alice:Documents");
        store
            .public_shares
            .insert(tok.clone(), "personal:alice:Documents".into());
        let p = resolve_share_target("personal:alice:Documents", "spec.txt").unwrap();
        assert_eq!(p.to_str(), Some("/users/alice/Documents/spec.txt"));

        assert_eq!(sanitize_username("Alice@Example.com").unwrap(), "alice");

        let snap = FleetSnapshot::default();
        let h = render_login_page(None, &snap, &store);
        assert!(!h.contains("login-sub") && !h.contains("sec-ticker") && !h.contains("sec-chibi"));
        assert!(h.contains("User or Email") && !h.contains("THRALL MESH DIRECTOR"));
        assert!(h.contains("spotlight-canvas") && h.contains("/login.js"));
        assert!(h.contains("site-header") && h.contains("[ SIGN IN ]"));
        assert!(h.contains("footer-dock") && h.contains("id=\"footer-accent-picker\""));
        assert!(!h.contains("nav-arrow-up") && !h.contains("MESH HUB") && !h.contains("ENROLLED SPOKE") && h.contains("id=\"footer-status-msg\"") && h.contains("Sign in with your account credentials in the top bar."));
        let h_err = render_login_page(Some("Invalid credentials"), &snap, &store);
        assert!(h_err.contains("id=\"footer-status-msg\">Invalid credentials</span>"));
        assert!(h.contains("data-mesh-rtts=") && h.contains("data-mesh-masses="));
        let tj = crate::render_portal::render_telemetry_json(&snap, &store);
        assert!(tj.contains("\"hubs_total\":") && tj.contains("\"spokes\":1") && !tj.contains("alice") && !tj.contains("dev-1"));
        for w in ["LDAP", "ahamkara", "dragon-pact", "Wish-Dragon"] {
            assert!(!h.contains(w));
        }
        assert!(!h.contains("Communion") && !h.contains("llms.txt") && !h.contains("bearer mine"));
        let user_html = render_dashboard_page("bob", false, &store, &snap, 4, "", "", Some("Share mode updated"), None);
        assert!(!user_html.contains(">Access<") && !user_html.contains(">Hubs<") && !user_html.contains("MESH HUB") && !user_html.contains("ENROLLED SPOKE"));
        assert!(user_html.contains(">Personal</a>") && user_html.contains(">Shared</a>") && user_html.contains(">Spokes</a>"));
        assert!(user_html.contains("id=\"footer-status-msg\">Share mode updated</span>") && !user_html.contains("class=\"alert"));
        assert!(user_html.contains("<th style=\"width:110px\">Share</th><th style=\"width:200px\">Link</th><th style=\"width:180px;text-align:right\">Action</th>"));
        assert!(user_html.contains("class=\"explorer-table\"") && user_html.contains("data-modal-open=\"create-personal-folder\""));
        assert!(!user_html.contains("LDAP") && !user_html.contains("ahamkara"));

        let common_html = render_dashboard_page("bob", false, &store, &snap, 2, "", "", None, None);
        assert!(common_html.contains("&#127760; Ops/") && !common_html.contains("Shared / Ops/") && !common_html.contains("class=\"card\""));
        assert!(common_html.contains("class=\"tree-toggle\" data-target=\"shared:Ops\""));
        assert!(common_html.contains("id=\"footer-status-msg\">Shared folders across your team"));

        let empty_h = render_dashboard_page("emptyuser", false, &store, &snap, 1, "personal", "", None, Some("Invalid folder name"));
        assert!(empty_h.contains("0 folders accessible yet. Create a personal folder above.") && empty_h.contains("id=\"footer-status-msg\">Invalid folder name</span>"));

        let root_a = render_dashboard_page("alice", false, &store, &snap, 1, "", "", None, None);
        assert!(root_a.contains("&#128193; Documents/") && !root_a.contains("Personal / Documents/") && !root_a.contains("class=\"card\""));
        assert!(root_a.contains("class=\"tree-toggle\" data-target=\"personal:Documents\"") && root_a.contains("data-drop-browse=\"personal:Documents\""));
        assert!(root_a.contains("data-del-action=\"/user/folders/delete\" data-del-name=\"Documents\"") && root_a.contains("id=\"footer-upload-graph\""));
        assert!(root_a.contains("class=\"tree-child\" data-parent=\"personal:Documents\"") && root_a.contains("Empty directory") && root_a.contains("data-modal-open=\"mkdir\"") && root_a.contains("data-modal-open=\"upload-file\""));
        assert!(root_a.contains(&format!(">/s/{tok}</a></td>")) && root_a.contains("id=\"footer-status-msg\">Press + to expand a folder"));
        assert_eq!(crate::render_portal::share_jail::decode_hex("48656c6c6f"), b"Hello");

        let br = "personal:Documents";
        let in_h = render_dashboard_page("alice", false, &store, &snap, 1, br, "", None, None);
        assert!(in_h.contains("All Folders") && in_h.contains("Personal / Documents") && !in_h.contains("class=\"card\""));

        let s3 = render_dashboard_page("alice", false, &store, &snap, 3, "", "", None, None);
        assert!(s3.contains("class=\"explorer-table\"") && s3.contains("data-modal-open=\"enroll-spoke\"") && !s3.contains("class=\"card\""));
        assert!(s3.contains("data-target=\"spoke:") && s3.contains("/?tab=2&amp;browse=shared:Ops"));

        let acl_html = render_dashboard_page("admin", true, &store, &snap, 4, "", "", None, None);
        assert!(acl_html.contains("class=\"explorer-table\"") && acl_html.contains("data-modal-open=\"create-shared-folder\"") && acl_html.contains("Apply ACL") && !acl_html.contains("class=\"card\""));

        let admin_html = render_dashboard_page("admin", true, &store, &snap, 5, "", "", None, None);
        assert!(admin_html.contains(">Access</a>") && admin_html.contains(">Hubs</a>") && !admin_html.contains("class=\"card\""));
        assert!(admin_html.contains("data-modal-open=\"add-mesh-hub\"") && !admin_html.contains("LDAP"));

        let tmp_base = std::env::temp_dir().join("thrall_test_jail");
        let _ = std::fs::create_dir_all(&tmp_base);
        let tmp_sym = tmp_base.join("escape_link");
        let _ = std::fs::remove_file(&tmp_sym);
        #[cfg(unix)]
        if std::os::unix::fs::symlink("/etc/hosts", &tmp_sym).is_ok() {
            assert!(!crate::render_portal::share_jail::verify_no_symlink_escape(
                &tmp_base, &tmp_sym
            ));
            let _ = std::fs::remove_file(&tmp_sym);
        }
        let _ = std::fs::remove_dir_all(&tmp_base);
    }
}
