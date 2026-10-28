//! THRALL — Multi-Server Syncthing LDAP Control Plane & Self-Service Portal.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod device_store;
pub mod fleet_config;
pub mod ldap_bind;
pub mod peer_client;
pub mod portal_server;
pub mod reconcile_mesh;
pub mod render_portal;

use std::env;
use std::net::{TcpListener, TcpStream};
use std::process;
use std::time::Duration;

use crate::device_store::MeshStore;
use crate::fleet_config::load_or_create_fleet_config;
use crate::portal_server::handle_http_client;
use crate::reconcile_mesh::reconcile_fleet;

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
    let mut store = MeshStore::load(&cfg.portal.database_path);
    eprintln!("[thrall] Loaded state from {} ({} users)", cfg.portal.database_path, store.users.len());
    let snapshot = reconcile_fleet(&cfg, &mut store);
    let online_count = snapshot.nodes.iter().filter(|n| n.online).count();
    eprintln!("[thrall] Initial fleet reconciliation complete: {online_count}/{} hubs online", snapshot.nodes.len());
    if reconcile_only {
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
        assert!(sanitize_rel_subpath("../etc/passwd").is_err() && sanitize_rel_subpath(".ssh/id_rsa").is_err());
        assert_eq!(sanitize_rel_subpath("Documents/Notes").unwrap(), "Documents/Notes");
        assert_eq!(sanitize_folder_name("Project Alpha").unwrap(), "Project-Alpha");
        assert!(sanitize_folder_name(".config").is_err());
        let messy_id = "  device_id: aaaaaaa aaaaaaa\u{200b}-bbbbbbb_bbbbbbn ccccccc-cccccc2-ddddddd-ddddddh  ";
        let norm_id = normalize_and_verify_device_id(messy_id).unwrap();
        assert_eq!(norm_id, "AAAAAAA-AAAAAAA-BBBBBBB-BBBBBBN-CCCCCCC-CCCCCC2-DDDDDDD-DDDDDDH");
        let mut store = MeshStore::default();
        store.upsert_device("alice", messy_id, "").unwrap();
        assert_eq!(store.users["alice"][0].label, "dev-1-aaaaaaa");
        assert!(store.folders_for_user("alice").is_empty());
        store.ensure_user_folders("alice").push("Documents".into());
        store.ensure_user_folders("alice").push("Projects".into());
        assert_eq!(store.local_path_for("alice", "personal:Documents"), "~/Documents");
        assert_eq!(store.local_path_for("alice", "personal:Projects"), "~/Thrall/Projects");
        assert_eq!(store.local_path_for("alice", "shared:Ops"), "~/Thrall/Ops");
        assert!(crate::device_store::sanitize_local_path("../etc/passwd").is_err() && crate::device_store::sanitize_local_path("").is_err());
        assert_eq!(crate::device_store::sanitize_local_path("C:\\Users\\Alice\\Desktop").unwrap(), "C:/Users/Alice/Desktop");
        let xdg_multi = crate::portal_server::parse_form_multi("xdg=Desktop&xdg=Documents&xdg=..%2Fbad&folder=Custom", "xdg");
        assert!(xdg_multi.contains("Desktop") && xdg_multi.contains("Documents") && xdg_multi.len() == 2);
        store.folder_paths.insert(("alice".into(), "personal:Documents".into()), "~/Thrall/Documents".into());
        store.user_folders.insert("emptyuser".into(), Vec::new());
        let roundtrip = MeshStore::parse(&format!("{}\n@folder_path\talice\tbad_scope:..\t~/Bad\n", store.serialize()));
        assert!(roundtrip.folders_for_user("emptyuser").is_empty() && !roundtrip.folder_paths.contains_key(&("alice".into(), "bad_scope:..".into())));
        assert_eq!(roundtrip.local_path_for("alice", "personal:Documents"), "~/Thrall/Documents");
        assert!(resolve_jailed_path("emptyuser", &roundtrip, "personal", "").is_none());
        store.shared_folders.entry("Ops".into()).or_default().insert("alice".into());
        store.shared_folders.entry("Ops".into()).or_default().insert("bob".into());
        assert!(resolve_jailed_path("alice", &store, "shared:Ops", "Reports").is_some());
        assert!(resolve_jailed_path("charlie", &store, "shared:Ops", "Reports").is_none());
        assert!(resolve_jailed_path("alice", &store, "personal:Documents", "Notes").is_some());
        assert!(resolve_jailed_path("alice", &store, "personal:UnknownFolder", "").is_none());
        let cfg = crate::fleet_config::parse_fleet_yaml(crate::fleet_config::DEFAULT_FLEET_YAML).unwrap();
        let tok = compute_share_token(&cfg.portal.session_secret, "personal:alice:Documents");
        assert!(tok.len() == 32 && tok != compute_share_token("other-secret", "personal:alice:Documents"));
        store.public_shares.insert(tok.clone(), "personal:alice:Documents".into());
        let p = resolve_share_target("personal:alice:Documents", "spec.txt").unwrap();
        assert_eq!(p.to_str(), Some("/users/alice/Documents/spec.txt"));
        assert_eq!(sanitize_username("Alice@Example.com").unwrap(), "alice");

        let mut snap = FleetSnapshot { ldap_users: vec!["alice".into(), "bob".into()], ..FleetSnapshot::default() };
        store.shared_readonly.insert(("Ops".into(), "bob".into()));
        store.device_excluded_folders.insert((norm_id.clone(), "personal:Projects".into()));
        let rt_ro = MeshStore::parse(&store.serialize());
        assert!(rt_ro.is_shared_readonly("Ops", "bob") && !rt_ro.is_shared_readonly("Ops", "alice") && rt_ro.is_device_folder_excluded(&norm_id, "personal:Projects") && !rt_ro.is_device_folder_excluded(&norm_id, "personal:Documents"));
        assert!(crate::reconcile_mesh::build_folder_json("f1", "L", "/p", &[], true).contains("\"type\":\"sendonly\"") && crate::reconcile_mesh::build_folder_json("f1", "L", "/p", &[], false).contains("\"type\":\"sendreceive\""));
        let conn_j = format!("{{\"connections\":{{\"{norm_id}\":{{\"connected\":true}},\"BBBBBBB-BBBBBBB-BBBBBBB-BBBBBBB-BBBBBBB-BBBBBBB-BBBBBBB-BBBBBBB\":{{\"connected\":false}}}}}}");
        assert_eq!(crate::reconcile_mesh::parse_connected_device_ids(&conn_j), vec![norm_id.clone()]);
        let h = render_login_page(None, &snap, &store);
        assert!(!h.contains("login-sub") && !h.contains("sec-ticker") && !h.contains("sec-chibi"));
        assert!(h.contains("User or Email") && !h.contains("THRALL MESH DIRECTOR"));
        assert!(h.contains("spotlight-canvas") && h.contains("/login.js"));
        assert!(h.contains("site-header") && h.contains("[ SIGN IN ]") && h.contains("/icon.jpg") && h.contains("data-modal-open=\"about-thrall\""));
        assert!(h.contains("footer-dock") && h.contains("id=\"footer-accent-picker\"") && h.contains("id=\"footer-upload-graph\"") && !h.contains("id=\"footer-thrall-sprite\"") && h.contains("id=\"footer-hubs-eq\"") && h.contains("id=\"footer-mesh-graph\"") && h.contains("id=\"footer-vault-gauge\"") && h.contains("id=\"footer-uptime-clock\"") && !h.contains("id=\"footer-bg-btn\"") && !h.contains("id=\"footer-warp-btn\"") && h.contains("<div class=\"footer-left-slot\"><canvas id=\"footer-upload-graph\""));
        assert!(!h.contains("nav-arrow-up") && !h.contains("MESH HUB") && !h.contains("ENROLLED DEVICE") && h.contains("id=\"footer-status-msg\"") && !h.contains("Sign in with your account credentials"));
        let h_err = render_login_page(Some("Invalid credentials"), &snap, &store);
        let (h_err_hdr, h_err_rest) = h_err.split_once("</header>").unwrap();
        assert!(!h_err_hdr.contains("Invalid credentials") && h_err_rest.contains("id=\"footer-status-msg\" class=\"footer-toast toast-err\">Invalid credentials</span>"));
        assert!(h.contains("data-mesh-rtts=") && h.contains("data-mesh-masses="));
        let tj = crate::render_portal::render_telemetry_json(&snap, &store);
        assert!(tj.contains("\"hubs_total\":") && tj.contains("\"spokes\":1") && !tj.contains("alice") && !tj.contains("dev-1"));
        assert!(!h.contains("LDAP"));
        // The published portal must not carry references to the external
        // archive. The forbidden tokens are stored as hashes so the names
        // themselves never appear in this source tree.
        const ARCHIVE_TOKENS: [u64; 6] = [
            0x8baa_8108_53ce_3d7b,
            0x817f_b5b0_59a3_d5d0,
            0x7104_93a2_1d69_315e,
            0x037e_2b0d_eb9b_006d,
            0xbe36_4638_08f3_e590,
            0xea9e_60ad_81da_16f1,
        ];
        let leaked: Vec<&str> = h
            .split(|c: char| !c.is_ascii_alphanumeric())
            .filter(|w| w.len() >= 4)
            .filter(|w| {
                let mut v = 0xcbf2_9ce4_8422_2325u64;
                for b in w.bytes().map(|b| b.to_ascii_lowercase()) {
                    v = (v ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
                }
                ARCHIVE_TOKENS.contains(&v)
            })
            .collect();
        assert!(leaked.is_empty(), "archive reference leaked into the portal: {leaked:?}");
        let user_html = render_dashboard_page("bob", false, &cfg, &store, &snap, 4, "", "", Some("Share mode updated"), None);
        let (_, u_body) = user_html.split_once("<header class=\"site-header\"").unwrap();
        let (u_hdr, u_rest) = u_body.split_once("</header>").unwrap();
        let (u_mid, u_ftr) = u_rest.split_once("<footer class=\"footer-dock\"").unwrap();
        assert!(!u_hdr.contains("nav-tabs") && u_hdr.contains("/icon.jpg") && u_hdr.contains("href=\"/?tab=about\"") && !u_hdr.contains("<input") && !u_hdr.contains("USER //") && !u_hdr.contains("ROLE //") && u_hdr.contains("<form method=\"POST\" action=\"/logout\" autocomplete=\"off\" class=\"header-login-form\"><button type=\"submit\">[ SIGN OUT ]</button></form>"));
        assert!(u_mid.contains("<nav class=\"body-tabs\" aria-label=\"Portal Navigation\"><div class=\"body-tabs-left\"><a href=\"/?tab=personal\" class=\"body-tab active\">Personal</a><a href=\"/?tab=shared\" class=\"body-tab\">Shared</a><a href=\"/?tab=devices\" class=\"body-tab\">Devices</a></div></nav>") && !u_ftr.contains("body-tabs") && !u_ftr.contains("footer-nav-tabs"));
        assert!(user_html.contains("id=\"footer-status-msg\" class=\"footer-toast\">Share mode updated</span>") && !user_html.contains("class=\"alert"));
        assert!(user_html.contains("<th style=\"width:110px\">Share</th><th style=\"width:200px\">Link</th><th style=\"width:180px;text-align:right\">Action</th>"));
        assert!(user_html.contains("class=\"explorer-table\"") && user_html.contains("data-modal-open=\"create-standard-folder\"") && user_html.contains("data-modal-open=\"create-custom-folder\""));

        let common_html = render_dashboard_page("bob", false, &cfg, &store, &snap, 2, "", "", None, None);
        assert!(common_html.contains("&#127760; Ops/") && common_html.contains("READ-ONLY") && !common_html.contains("data-modal-open=\"upload-file\"") && !common_html.contains("Shared / Ops/") && !common_html.contains("class=\"card\""));
        assert!(common_html.contains("class=\"tree-toggle\" data-target=\"shared:Ops\""));
        assert!(!common_html.contains("Shared folders across your team"));

        let empty_h = render_dashboard_page("emptyuser", false, &cfg, &store, &snap, 1, "personal", "", None, Some("Invalid folder name"));
        assert!(empty_h.contains("0 folders accessible yet. Create a personal folder above.") && empty_h.contains("id=\"footer-status-msg\" class=\"footer-toast toast-err\">Invalid folder name</span>"));

        let root_a = render_dashboard_page("alice", false, &cfg, &store, &snap, 1, "", "", None, None);
        assert!(root_a.contains("&#128193; Documents/") && root_a.contains("data-existing=\"Documents,Projects\"") && !root_a.contains("class=\"card\""));
        assert!(root_a.contains("class=\"tree-toggle\" data-target=\"personal:Documents\"") && root_a.contains("data-drop-browse=\"personal:Documents\""));
        assert!(root_a.contains("data-del-action=\"/user/folders/delete\" data-del-name=\"Documents\"") && root_a.contains("<canvas id=\"footer-upload-graph\" class=\"footer-upload-graph\" width=\"128\" height=\"24\" title=\"Browser &lt;-&gt; Mesh Data Flow\""));
        assert!(root_a.contains("class=\"tree-child\" data-parent=\"personal:Documents\"") && root_a.contains("Empty directory") && root_a.contains("data-modal-open=\"mkdir\"") && root_a.contains("data-modal-open=\"upload-file\""));
        assert!(root_a.contains(&format!(">/s/{tok}</a></td>")) && !root_a.contains("Press + to expand a folder"));
        assert_eq!(crate::render_portal::share_jail::decode_hex("48656c6c6f"), b"Hello");

        let br = "personal:Documents";
        let in_h = render_dashboard_page("alice", false, &cfg, &store, &snap, 1, br, "", None, None);
        assert!(in_h.contains("All Folders") && in_h.contains("Personal / Documents") && !in_h.contains("class=\"card\""));

        let s3 = render_dashboard_page("alice", false, &cfg, &store, &snap, 3, "", "", None, None);
        assert!(s3.contains("class=\"explorer-table\"") && s3.contains("data-modal-open=\"enroll-device\" data-folders=\"personal:Documents,personal:Projects,shared:Ops\">+ Add Device</button>") && !s3.contains("class=\"card\""));
        assert!(s3.contains("data-target=\"device:") && s3.contains("/?tab=shared&amp;browse=shared:Ops") && s3.contains("data-modal-open=\"set-folder-path\"") && s3.contains("~/Thrall/Documents") && s3.contains("Sync: ON") && s3.contains("Sync: OFF") && s3.contains("Cloud Only (Not Synced)") && !s3.contains("~/THRALL"));
        let s3_bob = render_dashboard_page("bob", false, &cfg, &store, &snap, 3, "", "", None, None);
        assert!(s3_bob.contains("NO DEVICES") && s3_bob.contains("~/Thrall/Ops") && !s3_bob.contains(">Open</a>"));

        let about_html = render_dashboard_page("alice", false, &cfg, &store, &snap, 6, "", "", None, None);
        assert!(about_html.contains("What is THRALL?") && about_html.contains("tuition-not-coffee") && about_html.contains("Syncthing &amp; Block Exchange Protocol (BEP)") && about_html.contains("https://github.com/UberMetroid/thrall/blob/main/AGENTS.md") && about_html.contains("Architecture &amp; Rules"));

        let admin_html = render_dashboard_page("admin", true, &cfg, &store, &snap, 4, "", "", None, None);
        let (_, adm_body) = admin_html.split_once("<header class=\"site-header\"").unwrap();
        let (hdr_part, rest_part) = adm_body.split_once("</header>").unwrap();
        let (mid_part, ftr_part) = rest_part.split_once("<footer class=\"footer-dock\"").unwrap();
        assert!(!hdr_part.contains("nav-tabs") && hdr_part.contains("/icon.jpg") && hdr_part.contains("href=\"/?tab=about\"") && hdr_part.contains("<form method=\"POST\" action=\"/logout\" autocomplete=\"off\" class=\"header-login-form\"><button type=\"submit\">[ SIGN OUT ]</button></form>"));
        assert!(mid_part.contains("<div class=\"body-tabs-right\"><a href=\"/?tab=admin\" class=\"body-tab admin-tab active\">Admin</a></div>") && !ftr_part.contains("footer-nav-tabs"));
        assert!(admin_html.contains("data-target=\"admin:global\"") && admin_html.contains("data-target=\"admin:mesh\"") && admin_html.contains("data-target=\"admin:ldap\""));
        assert!(admin_html.contains("data-modal-open=\"create-shared-folder\"") && admin_html.contains("acl-save-btn") && admin_html.contains("name=\"ro_users\" value=\"bob\" checked") && admin_html.contains("data-modal-open=\"add-mesh-hub\"") && admin_html.contains("/admin/ldap/test") && admin_html.contains("/admin/ldap/update"));

        let tmp_base = std::env::temp_dir().join("thrall_test_jail");
        let _ = std::fs::create_dir_all(&tmp_base);
        let tmp_sym = tmp_base.join("escape_link");
        let _ = std::fs::remove_file(&tmp_sym);
        #[cfg(unix)]
        if std::os::unix::fs::symlink("/etc", &tmp_sym).is_ok() {
            assert!(!crate::render_portal::share_jail::verify_no_symlink_escape(&tmp_base, &tmp_sym));
            assert!(!crate::render_portal::share_jail::verify_no_symlink_escape(&tmp_base, &tmp_sym.join("nested/sub/path")));
            let _ = std::fs::remove_file(&tmp_sym);
        }
        let _ = std::fs::remove_dir_all(&tmp_base);
        snap.ldap_users.clear();
    }
}
