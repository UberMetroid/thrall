//! Admin POST form handlers (Tab 4 shared folder ACLs & Tab 5 Mesh Hubs) for THRALL.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::net::TcpStream;

use crate::device_store::{
    sanitize_folder_name, sanitize_username, syncthing_folder_id, MeshStore,
};
use crate::fleet_config::{save_fleet_config, FleetConfig, ServerNodeConfig};
use crate::ldap_bind::{list_ldap_users, validate_ldap_connection};
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
        local: false,
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

#[rustfmt::skip]
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
        write_redirect(stream, "/?tab=admin&ok=Fleet+mesh+reconciled", "");
        return true;
    }
    if path == "/admin/ldap/test" || path == "/admin/ldap/update" {
        if path == "/admin/ldap/update" {
            let (u, bdn, bind, pass, filt) = (fval("url").trim(), fval("base_dn").trim(), fval("bind_dn").trim(), fval("bind_password"), fval("user_filter").trim());
            if !u.is_empty() { cfg.ldap.url = u.to_string(); }
            if !bdn.is_empty() { cfg.ldap.base_dn = bdn.to_string(); }
            if !bind.is_empty() { cfg.ldap.bind_dn = bind.to_string(); }
            if !pass.is_empty() { cfg.ldap.bind_password = pass.to_string(); }
            if !filt.is_empty() { cfg.ldap.user_filter = filt.to_string(); }
            let mut adms: Vec<String> = fval("admin_users").split(',').filter_map(|x| sanitize_username(x).ok()).collect();
            if !adms.iter().any(|a| a == username) { adms.push(username.to_string()); }
            cfg.ldap.admin_users = adms;
            let _ = save_fleet_config("/config/fleet.yaml", cfg);
        }
        snap.ldap_users = list_ldap_users(&cfg.ldap);
        for u in store.users.keys() {
            if u != "root" && !snap.ldap_users.contains(u) { snap.ldap_users.push(u.clone()); }
        }
        snap.ldap_users.sort();
        match validate_ldap_connection(&cfg.ldap) {
            Ok(cnt) => write_redirect(stream, &format!("/?tab=admin&ok=LDAP+verified+({cnt}+entries+discovered)"), ""),
            Err(e) => write_redirect(stream, &format!("/?tab=admin&err=LDAP+connection+failed:+{}", e.replace(' ', "+")), ""),
        }
        return true;
    }
    if path == "/admin/shared/delete" {
        if let Ok(fname) = sanitize_folder_name(fval("folder")) {
            let c = fval("confirm_name").trim();
            if !c.is_empty() && c != fname {
                write_redirect(stream, "/?tab=admin&err=Confirmation+name+mismatch", "");
                return true;
            }
            store.shared_folders.remove(&fname);
            store.shared_readonly.retain(|(f, _)| f != &fname);
            let sf_key = format!("shared:{fname}");
            store.folder_paths.retain(|(_, sf), _| sf != &sf_key);
            store.device_excluded_folders.retain(|(_, sf)| sf != &sf_key);
            let _ = store.save(&cfg.portal.database_path);
            let flow = fname.to_ascii_lowercase();
            purge_folder_from_fleet(cfg, &snap.nodes, &syncthing_folder_id(&format!("shared:{flow}:mesh")));
            for d in store.users.values().flatten() {
                let sid = short_id(&d.device_id);
                purge_folder_from_fleet(cfg, &snap.nodes, &syncthing_folder_id(&format!("shared:{flow}:{sid}")));
            }
            if fname != "Shared" {
                let _ = fs::remove_dir_all(format!("/shared/{fname}"));
            }
            snap.shared_completion.remove(&fname);
            write_redirect(stream, "/?tab=admin&ok=Shared+folder+deleted", "");
            return true;
        }
        write_redirect(stream, "/?tab=admin&err=Cannot+delete+shared+folder", "");
        return true;
    }
    if path.starts_with("/admin/shared/") {
        if let Ok(fname) = sanitize_folder_name(fval("folder")) {
            if fname != "Shared" {
                let _ = fs::create_dir_all(format!("/shared/{fname}/.stfolder"));
            }
            let (u, ro) = if path.ends_with("update") {
                (parse_form_multi(body, "users"), parse_form_multi(body, "ro_users"))
            } else {
                (BTreeSet::from([username.to_string()]), BTreeSet::new())
            };
            store.shared_readonly.retain(|(f, _)| f != &fname);
            for ru in ro.intersection(&u) {
                store.shared_readonly.insert((fname.clone(), ru.clone()));
            }
            let flow = fname.to_ascii_lowercase();
            for (usr, devs) in &store.users {
                if !u.contains(usr) {
                    for d in devs {
                        let sid = short_id(&d.device_id);
                        purge_folder_from_fleet(cfg, &snap.nodes, &syncthing_folder_id(&format!("shared:{flow}:{sid}")));
                    }
                }
            }
            store.shared_folders.insert(fname.clone(), u);
            let _ = store.save(&cfg.portal.database_path);
            provision_single_folder(cfg, snap, store, None, &fname);
            write_redirect(stream, "/?tab=admin&ok=Shared+folder+provisioned!+Authorized+users+should+open+http://127.0.0.1:8384+on+their+Device+to+accept+the+share.", "");
        } else {
            write_redirect(stream, "/?tab=admin&err=Invalid+shared+folder+name", "");
        }
        return true;
    }
    if path == "/admin/servers/add" {
        let host = fval("host").trim();
        let valid_h = !host.is_empty() && host.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'));
        if let Ok(name) = sanitize_folder_name(fval("name")) {
            let nlow = name.to_ascii_lowercase();
            if valid_h && !cfg.servers.iter().any(|s| s.name == nlow) {
                let node = new_node(&nlow, host, cfg.servers.first());
                cfg.servers.push(node);
                let _ = save_fleet_config("/config/fleet.yaml", cfg);
                *snap = reconcile_fleet(cfg, store);
                write_redirect(stream, "/?tab=admin&ok=Mesh+Hub+added+to+fleet", "");
                return true;
            }
        }
        write_redirect(stream, "/?tab=admin&err=Invalid+Mesh+Hub+name+or+host", "");
        return true;
    }
    let srv = fval("server").trim();
    if path == "/admin/servers/toggle" && cfg.servers.iter().any(|s| s.name == srv) {
        if !store.disabled_servers.remove(srv) {
            store.disabled_servers.insert(srv.to_string());
            if let Some(id) = snap.nodes.iter().find(|n| n.name == srv && !n.device_id.is_empty()).map(|n| n.device_id.clone()) {
                purge_device_from_fleet(cfg, &snap.nodes, &id);
            }
        }
        let _ = store.save(&cfg.portal.database_path);
        *snap = reconcile_fleet(cfg, store);
        write_redirect(stream, "/?tab=admin&ok=Server+mesh+connection+updated", "");
        return true;
    }
    false
}
