//! Authenticated POST form handlers for THRALL (`st-mesh-mgr`) portal.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::TcpStream;

use crate::device_store::{
    compute_share_token, normalize_and_verify_device_id, resolve_jailed_path, sanitize_folder_name,
    sanitize_local_path, syncthing_folder_id, MeshStore,
};
use crate::fleet_config::FleetConfig;
use crate::portal_server::{parse_form_multi, write_html, write_http, write_redirect};
use crate::reconcile_mesh::{
    provision_single_folder, purge_device_from_fleet, purge_folder_from_fleet, reconcile_fleet,
    verify_device_id_via_server, FleetSnapshot,
};
use crate::render_portal::admin_actions::{handle_admin_post_action, short_id};
use crate::render_portal::render_dashboard_page;
use crate::render_portal::share_jail::decode_hex;

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
pub fn handle_post_action(
    stream: &mut TcpStream,
    path: &str,
    body: &str,
    form: &BTreeMap<String, String>,
    username: &str,
    is_admin: bool,
    cfg: &mut FleetConfig,
    store: &mut MeshStore,
    snap: &mut FleetSnapshot,
) -> bool {
    let fval = |k: &str| form.get(k).map(String::as_str).unwrap_or("");
    if path == "/user/folders/create" || path == "/user/folders/create-xdg" {
        let xdg_list = parse_form_multi(body, "xdg");
        let custom = sanitize_folder_name(fval("folder")).ok();
        let mut to_create: Vec<String> = xdg_list.iter().cloned().collect();
        if let Some(c) = custom.as_ref().filter(|c| !to_create.iter().any(|x| x.eq_ignore_ascii_case(c))) {
            to_create.push(c.clone());
        }
        if !to_create.is_empty() {
            let xdg_thrall = fval("xdg_root") == "thrall";
            for fname in &to_create {
                let list = store.ensure_user_folders(username);
                if !list.iter().any(|f| f.eq_ignore_ascii_case(fname)) { list.push(fname.clone()); }
                if xdg_list.contains(fname) {
                    let sf = format!("personal:{fname}");
                    if xdg_thrall { store.folder_paths.insert((username.to_string(), sf), format!("~/Thrall/{fname}")); }
                    else { store.folder_paths.remove(&(username.to_string(), sf)); }
                }
                let _ = fs::create_dir_all(format!("/users/{username}/{fname}/.stfolder"));
            }
            if let (Some(c), Ok(lp)) = (custom.as_ref(), sanitize_local_path(fval("local_path"))) {
                store.folder_paths.insert((username.to_string(), format!("personal:{c}")), lp);
            }
            let _ = store.save(&cfg.portal.database_path);
            for fname in &to_create { provision_single_folder(cfg, snap, store, Some(username), fname); }
            let u = if to_create.len() == 1 {
                format!("/?tab=1&browse=personal:{}&ok=Personal+folder+created!+Open+http://127.0.0.1:8384+on+your+Device+to+accept+the+new+share.", to_create[0])
            } else { format!("/?tab=1&ok=Provisioned+{}+personal+folders!", to_create.len()) };
            write_redirect(stream, &u, "");
        } else if fval("folder").trim().is_empty() {
            write_redirect(stream, "/?tab=1&err=Select+at+least+one+XDG+folder+or+enter+a+custom+folder+name", "");
        } else {
            write_redirect(stream, "/?tab=1&err=Invalid+folder+name", "");
        }
        return true;
    }
    if path == "/user/folders/path" {
        let sf = fval("folder").trim();
        let valid_sf = sf.strip_prefix("personal:").is_some_and(|pf| store.folders_for_user(username).iter().any(|x| x == pf))
            || sf.strip_prefix("shared:").is_some_and(|sh| store.shared_folders.get(sh).is_some_and(|a| a.contains(username)));
        if let (true, Ok(lp)) = (valid_sf, sanitize_local_path(fval("local_path"))) {
            store.folder_paths.insert((username.to_string(), sf.to_string()), lp);
            let _ = store.save(&cfg.portal.database_path);
            write_redirect(stream, "/?tab=3&ok=Local+sync+path+updated", "");
        } else {
            write_redirect(stream, "/?tab=3&err=Invalid+folder+or+local+path", "");
        }
        return true;
    }
    if path == "/user/folders/delete" {
        if let Ok(fname) = sanitize_folder_name(fval("folder")) {
            let c = fval("confirm_name").trim();
            if !c.is_empty() && c != fname {
                write_redirect(stream, "/?tab=1&err=Confirmation+name+mismatch", "");
                return true;
            }
            store.ensure_user_folders(username).retain(|f| f != &fname);
            store.folder_paths.remove(&(username.to_string(), format!("personal:{fname}")));
            let _ = store.save(&cfg.portal.database_path);
            let (flow, base) = (fname.to_ascii_lowercase(), format!("user-{username}-{}", fname.to_ascii_lowercase()));
            for id in [base.clone(), format!("{base}-mesh"), syncthing_folder_id(&format!("user:{username}:{flow}:mesh"))] {
                purge_folder_from_fleet(cfg, &snap.nodes, &id);
            }
            for d in store.users.get(username).into_iter().flatten() {
                let sid = short_id(&d.device_id);
                for cfid in [format!("{base}-{sid}"), syncthing_folder_id(&format!("user:{username}:{flow}:{sid}"))] {
                    purge_folder_from_fleet(cfg, &snap.nodes, &cfid);
                }
            }
            let _ = fs::remove_dir_all(format!("/users/{username}/{fname}"));
            snap.user_accepted.remove(&format!("{username}:{fname}"));
            write_redirect(stream, "/?tab=1&ok=Personal+folder+deleted", "");
        } else {
            write_redirect(stream, "/?tab=1&err=Invalid+folder+name", "");
        }
        return true;
    }
    if path == "/folders/mkdir" {
        let (br, sub, rbr) = (fval("browse"), fval("sub"), form.get("ret_br").map(String::as_str).unwrap_or(fval("browse")));
        let etab = if rbr.starts_with("shared:") || br.starts_with("shared:") { 2 } else { 1 };
        if let (Ok(d), Some((_, base, rel))) = (sanitize_folder_name(fval("dirname")), resolve_jailed_path(username, store, br, sub)) {
            let _ = fs::create_dir_all(base.join(&d));
            let nsub = if rel.is_empty() { d } else { format!("{rel}/{d}") };
            let u = if rbr.is_empty() { format!("/?tab={etab}&ok=Folder+created") } else { format!("/?tab={etab}&browse={rbr}&sub={nsub}&ok=Folder+created") };
            write_redirect(stream, &u, "");
        } else {
            let u = if rbr.is_empty() { format!("/?tab={etab}&err=Invalid+subfolder+name") } else { format!("/?tab={etab}&browse={rbr}&sub={sub}&err=Invalid+subfolder+name") };
            write_redirect(stream, &u, "");
        }
        return true;
    }
    if path == "/files/upload" || path == "/files/create" {
        let (br, sub, fnm, rbr) = (fval("browse"), fval("sub"), fval("filename").trim(), form.get("ret_br").map(String::as_str).unwrap_or(fval("browse")));
        let etab = if rbr.starts_with("shared:") || br.starts_with("shared:") { 2 } else { 1 };
        let valid_fn = !fnm.is_empty() && !fnm.starts_with('.') && fnm.len() <= 128 && fnm.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
        if let (true, Some((_, base, _))) = (valid_fn, resolve_jailed_path(username, store, br, sub)) {
            let fpath = base.join(fnm);
            let data = if !fval("hex").is_empty() { decode_hex(fval("hex")) } else { fval("content").as_bytes().to_vec() };
            let ok = fs::create_dir_all(&base).is_ok() && if fval("append") == "1" {
                OpenOptions::new().create(true).append(true).open(&fpath).and_then(|mut f| f.write_all(&data)).is_ok()
            } else {
                fs::write(&fpath, &data).is_ok()
            };
            if path == "/files/upload" {
                let (st, msg) = if ok { ("200 OK", "{\"ok\":true}\n") } else { ("500 Internal Server Error", "{\"ok\":false}\n") };
                write_http(stream, st, "application/json", "", msg.as_bytes());
            } else {
                let u = if rbr.is_empty() { format!("/?tab={etab}&ok=File+saved") } else { format!("/?tab={etab}&browse={rbr}&sub={sub}&ok=File+saved") };
                write_redirect(stream, &u, "");
            }
        } else if path == "/files/upload" {
            write_http(stream, "400 Bad Request", "application/json", "", b"{\"ok\":false}\n");
        } else {
            let u = if rbr.is_empty() { format!("/?tab={etab}&err=Invalid+file+name") } else { format!("/?tab={etab}&browse={rbr}&sub={sub}&err=Invalid+file+name") };
            write_redirect(stream, &u, "");
        }
        return true;
    }
    if path == "/files/delete" {
        let (br, sub, item, c) = (fval("browse"), fval("sub"), fval("item"), fval("confirm_name").trim());
        let (s_br, i_rel) = if br.is_empty() { item.split_once('/').unwrap_or((item, "")) } else { (br, item) };
        let etab = if s_br.starts_with("shared:") || br.starts_with("shared:") { 2 } else { 1 };
        if let Some((_, fp, rel_i)) = resolve_jailed_path(username, store, s_br, i_rel) {
            let leaf = rel_i.rsplit('/').next().unwrap_or(&rel_i);
            if !c.is_empty() && c != leaf {
                let u = if br.is_empty() { format!("/?tab={etab}&err=Confirmation+name+mismatch") } else { format!("/?tab={etab}&browse={br}&sub={sub}&err=Confirmation+name+mismatch") };
                write_redirect(stream, &u, "");
                return true;
            }
            if !rel_i.is_empty() {
                if fp.is_dir() { let _ = fs::remove_dir_all(&fp); } else { let _ = fs::remove_file(&fp); }
            }
        }
        let u = if br.is_empty() { format!("/?tab={etab}&ok=Item+deleted") } else { format!("/?tab={etab}&browse={br}&sub={sub}&ok=Item+deleted") };
        write_redirect(stream, &u, "");
        return true;
    }
    if path == "/shares/toggle" {
        let (br, sub, item) = (fval("browse"), fval("sub"), fval("item"));
        let (s_br, i_rel) = if br.is_empty() { item.split_once('/').unwrap_or((item, "")) } else { (br, item) };
        let etab = if s_br.starts_with("shared:") || br.starts_with("shared:") { 2 } else { 1 };
        if let Some((_, _, clean_item)) = resolve_jailed_path(username, store, s_br, i_rel) {
            let scope = if let Some(pf) = s_br.strip_prefix("personal:").or_else(|| s_br.strip_prefix("personal/")) { format!("personal:{username}:{pf}") } else { s_br.to_string() };
            let target = if clean_item.is_empty() { scope } else { format!("{scope}/{clean_item}") };
            let tok = compute_share_token(&target);
            if store.public_shares.remove(&tok).is_none() { store.public_shares.insert(tok, target); }
            let _ = store.save(&cfg.portal.database_path);
            let u = if br.is_empty() { format!("/?tab={etab}&ok=Share+mode+updated") } else { format!("/?tab={etab}&browse={br}&sub={sub}&ok=Share+mode+updated") };
            write_redirect(stream, &u, "");
            return true;
        }
    }
    let raw_id = fval("device_id");
    if path == "/devices/add" {
        let res = normalize_and_verify_device_id(raw_id).and_then(|norm_id| {
            verify_device_id_via_server(cfg, &snap.nodes, &norm_id)?;
            store.upsert_device(username, &norm_id, fval("label"))?;
            store.save(&cfg.portal.database_path)
        });
        match res {
            Ok(()) => {
                *snap = reconcile_fleet(cfg, store);
                write_redirect(stream, "/?tab=3&ok=Device+provisioned+across+fleet", "");
            }
            Err(e) => {
                let p = render_dashboard_page(username, is_admin, store, snap, 3, "", "", None, Some(&e));
                write_html(stream, "400 Bad Request", &p);
            }
        }
        return true;
    }
    if path == "/devices/remove" {
        let rem_id = if raw_id.is_empty() { fval("item") } else { raw_id };
        if let Ok(norm_id) = normalize_and_verify_device_id(rem_id) {
            if store.remove_device(username, &norm_id).unwrap_or(false) {
                let _ = store.save(&cfg.portal.database_path);
                let sid = short_id(&norm_id);
                for fname in store.shared_folders.keys() {
                    let flow = fname.to_ascii_lowercase();
                    for cfid in [format!("shared-{flow}-{sid}"), syncthing_folder_id(&format!("shared:{flow}:{sid}"))] {
                        purge_folder_from_fleet(cfg, &snap.nodes, &cfid);
                    }
                }
                for f in store.folders_for_user(username) {
                    let flow = f.to_ascii_lowercase();
                    for cfid in [format!("user-{username}-{flow}-{sid}"), syncthing_folder_id(&format!("user:{username}:{flow}:{sid}"))] {
                        purge_folder_from_fleet(cfg, &snap.nodes, &cfid);
                    }
                }
                purge_device_from_fleet(cfg, &snap.nodes, &norm_id);
                let devs = store.users.get(username).map_or(0, Vec::len);
                for f in store.folders_for_user(username) {
                    let k = format!("{username}:{f}");
                    snap.device_accepted.remove(&(format!("user:{k}"), norm_id.clone()));
                    if devs == 0 { snap.user_accepted.remove(&k); }
                }
                for fname in store.shared_folders.keys() {
                    snap.device_accepted.remove(&(format!("shared:{fname}"), norm_id.clone()));
                    if devs == 0 { snap.shared_user_accepted.remove(&(fname.clone(), username.to_string())); }
                }
            }
        }
        write_redirect(stream, "/?tab=3&ok=Device+removed+from+fleet", "");
        return true;
    }
    if is_admin {
        return handle_admin_post_action(stream, path, body, form, username, cfg, store, snap);
    }
    false
}
