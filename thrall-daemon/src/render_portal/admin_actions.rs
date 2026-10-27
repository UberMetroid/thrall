//! Admin POST form handlers (Tab 4 shared folder ACLs & Tab 5 Thrall Nodes) for THRALL.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::net::TcpStream;

use crate::device_store::{sanitize_folder_name, syncthing_folder_id, MeshStore};
use crate::fleet_config::{save_fleet_config, FleetConfig, ServerNodeConfig};
use crate::portal_server::{parse_form_multi, write_redirect};
use crate::reconcile_mesh::{
    provision_single_folder, purge_device_from_fleet, purge_folder_from_fleet, reconcile_fleet,
    FleetSnapshot,
};

pub fn short_id(dev_id: &str) -> String {
    dev_id
        .split('-')
        .next()
        .unwrap_or("dev")
        .to_ascii_lowercase()
}

fn new_node(name: &str, host: &str, tmpl: Option<&ServerNodeConfig>) -> ServerNodeConfig {
    let (ip, _) = host.split_once(':').unwrap_or((host, "22"));
    let sh = if host.contains(':') {
        host.into()
    } else {
        format!("{host}:22")
    };
    let u = tmpl.map_or("root", |s| &s.ssh_user).into();
    let k = tmpl
        .map_or("/config/id_ed25519", |s| &s.ssh_key_path)
        .into();
    let ub = tmpl.map_or("/users", |s| &s.users_base_path).into();
    let sb = tmpl.map_or("/shared", |s| &s.shared_base_path).into();
    ServerNodeConfig {
        name: name.to_ascii_lowercase(),
        ssh_host: sh,
        ssh_user: u,
        ssh_key_path: k,
        syncthing_api_url: "http://127.0.0.1:8384".into(),
        syncthing_api_key: String::new(),
        syncthing_wan_addr: format!("tcp://{ip}:22000"),
        users_base_path: ub,
        shared_base_path: sb,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_admin_post_action(
    stream: &mut TcpStream,
    path: &str,
    body: &str,
    form: &BTreeMap<String, String>,
    username: &str,
    cfg: &mut FleetConfig,
    store: &mut MeshStore,
    snap: &mut FleetSnapshot,
) -> bool {
    let fval = |k: &str| form.get(k).map(String::as_str).unwrap_or("");
    if path == "/reconcile" {
        *snap = reconcile_fleet(cfg, store);
        write_redirect(stream, "/?tab=5&ok=Fleet+mesh+reconciled", "");
        return true;
    }
    if path == "/admin/shared/delete" {
        if let Ok(fname) = sanitize_folder_name(fval("folder")) {
            store.shared_folders.remove(&fname);
            let _ = store.save(&cfg.portal.database_path);
            let flow = fname.to_ascii_lowercase();
            for id in [
                "mesh-shared".into(),
                format!("shared-{flow}"),
                format!("shared-{flow}-mesh"),
                syncthing_folder_id(&format!("shared:{flow}:mesh")),
            ] {
                if fname == "Shared" || id != "mesh-shared" {
                    purge_folder_from_fleet(cfg, &snap.nodes, &id);
                }
            }
            for d in store.users.values().flatten() {
                let sid = short_id(&d.device_id);
                for cfid in [
                    format!("shared-{flow}-{sid}"),
                    syncthing_folder_id(&format!("shared:{flow}:{sid}")),
                ] {
                    purge_folder_from_fleet(cfg, &snap.nodes, &cfid);
                }
            }
            if fname != "Shared" {
                let _ = fs::remove_dir_all(format!("/shared/{fname}"));
            }
            snap.shared_completion.remove(&fname);
            write_redirect(stream, "/?tab=4&ok=Shared+folder+deleted", "");
            return true;
        }
        write_redirect(stream, "/?tab=4&err=Cannot+delete+shared+folder", "");
        return true;
    }
    if path.starts_with("/admin/shared/") {
        if let Ok(fname) = sanitize_folder_name(fval("folder")) {
            if fname != "Shared" {
                let _ = fs::create_dir_all(format!("/shared/{fname}/.stfolder"));
            }
            let u = if path.ends_with("update") {
                parse_form_multi(body, "users")
            } else {
                BTreeSet::from([username.to_string()])
            };
            let flow = fname.to_ascii_lowercase();
            for (usr, devs) in &store.users {
                if !u.contains(usr) {
                    for d in devs {
                        let sid = short_id(&d.device_id);
                        for cfid in [
                            format!("shared-{flow}-{sid}"),
                            syncthing_folder_id(&format!("shared:{flow}:{sid}")),
                        ] {
                            purge_folder_from_fleet(cfg, &snap.nodes, &cfid);
                        }
                    }
                }
            }
            store.shared_folders.insert(fname.clone(), u);
            let _ = store.save(&cfg.portal.database_path);
            provision_single_folder(cfg, snap, store, None, &fname);
            write_redirect(
                stream,
                "/?tab=4&ok=Shared+folder+provisioned!+Authorized+users+should+open+http://127.0.0.1:8384+on+their+Spoke+to+accept+the+share.",
                "",
            );
        } else {
            write_redirect(stream, "/?tab=4&err=Invalid+shared+folder+name", "");
        }
        return true;
    }
    if path == "/admin/servers/add" {
        let host = fval("host").trim();
        let valid_h = !host.is_empty()
            && host
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'));
        if let Ok(name) = sanitize_folder_name(fval("name")) {
            let nlow = name.to_ascii_lowercase();
            if valid_h && !cfg.servers.iter().any(|s| s.name == nlow) {
                let node = new_node(&nlow, host, cfg.servers.first());
                cfg.servers.push(node);
                let _ = save_fleet_config("/config/fleet.yaml", cfg);
                *snap = reconcile_fleet(cfg, store);
                write_redirect(stream, "/?tab=5&ok=Mesh+Hub+added+to+fleet", "");
                return true;
            }
        }
        write_redirect(stream, "/?tab=5&err=Invalid+Mesh+Hub+name+or+host", "");
        return true;
    }
    let srv = fval("server").trim();
    if path == "/admin/servers/toggle" && cfg.servers.iter().any(|s| s.name == srv) {
        if !store.disabled_servers.remove(srv) {
            store.disabled_servers.insert(srv.to_string());
            if let Some(id) = snap
                .nodes
                .iter()
                .find(|n| n.name == srv && !n.device_id.is_empty())
                .map(|n| n.device_id.clone())
            {
                purge_device_from_fleet(cfg, &snap.nodes, &id);
            }
        }
        let _ = store.save(&cfg.portal.database_path);
        *snap = reconcile_fleet(cfg, store);
        write_redirect(stream, "/?tab=5&ok=Server+mesh+connection+updated", "");
        return true;
    }
    false
}
