//! Authenticated POST form handlers for THRALL (`st-mesh-mgr`) portal.

use std::collections::BTreeMap;
use std::fs;
use std::net::TcpStream;

use crate::device_store::{
    compute_share_token, normalize_and_verify_device_id, resolve_jailed_path, sanitize_folder_name,
    syncthing_folder_id, MeshStore,
};
use crate::fleet_config::FleetConfig;
use crate::portal_server::{write_html, write_redirect};
use crate::reconcile_mesh::{
    provision_single_folder, purge_device_from_fleet, purge_folder_from_fleet, reconcile_fleet,
    verify_device_id_via_server, FleetSnapshot,
};
use crate::render_portal::admin_actions::{handle_admin_post_action, short_id};
use crate::render_portal::render_dashboard_page;

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
    if path == "/user/folders/create" {
        if let Ok(fname) = sanitize_folder_name(fval("folder")) {
            let list = store.ensure_user_folders(username);
            if !list.iter().any(|f| f.eq_ignore_ascii_case(&fname)) {
                list.push(fname.clone());
            }
            let _ = fs::create_dir_all(format!("/users/{username}/{fname}/.stfolder"));
            let _ = store.save(&cfg.portal.database_path);
            provision_single_folder(cfg, snap, store, Some(username), &fname);
            let u = format!("/?tab=1&browse=personal:{fname}&ok=Personal+folder+created!+Open+http://127.0.0.1:8384+on+your+Spoke+to+accept+the+new+share.");
            write_redirect(stream, &u, "");
        } else {
            write_redirect(stream, "/?tab=1&err=Invalid+folder+name", "");
        }
        return true;
    }
    if path == "/user/folders/delete" {
        if let Ok(fname) = sanitize_folder_name(fval("folder")) {
            store.ensure_user_folders(username).retain(|f| f != &fname);
            let _ = store.save(&cfg.portal.database_path);
            let flow = fname.to_ascii_lowercase();
            let base = format!("user-{username}-{flow}");
            for id in [
                base.clone(),
                format!("{base}-mesh"),
                syncthing_folder_id(&format!("user:{username}:{flow}:mesh")),
            ] {
                purge_folder_from_fleet(cfg, &snap.nodes, &id);
            }
            for d in store.users.get(username).into_iter().flatten() {
                let sid = short_id(&d.device_id);
                for cfid in [
                    format!("{base}-{sid}"),
                    syncthing_folder_id(&format!("user:{username}:{flow}:{sid}")),
                ] {
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
        let (br, sub) = (fval("browse"), fval("sub"));
        if let (Ok(d), Some((_, base, rel))) = (
            sanitize_folder_name(fval("dirname")),
            resolve_jailed_path(username, store, br, sub),
        ) {
            let _ = fs::create_dir_all(base.join(&d));
            let nsub = if rel.is_empty() {
                d
            } else {
                format!("{rel}/{d}")
            };
            let u = format!("/?tab=1&browse={br}&sub={nsub}&ok=Folder+created");
            write_redirect(stream, &u, "");
        } else {
            let u = format!("/?tab=1&browse={br}&sub={sub}&err=Invalid+subfolder+name");
            write_redirect(stream, &u, "");
        }
        return true;
    }
    if path == "/files/create" {
        let (br, sub, fnm) = (fval("browse"), fval("sub"), fval("filename").trim());
        let valid_fn = !fnm.is_empty()
            && !fnm.starts_with('.')
            && fnm.len() <= 64
            && fnm
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'));
        if let (true, Some((_, base, _))) =
            (valid_fn, resolve_jailed_path(username, store, br, sub))
        {
            let _ = fs::write(base.join(fnm), fval("content").as_bytes());
            write_redirect(
                stream,
                &format!("/?tab=1&browse={br}&sub={sub}&ok=File+saved"),
                "",
            );
        } else {
            write_redirect(
                stream,
                &format!("/?tab=1&browse={br}&sub={sub}&err=Invalid+file+name"),
                "",
            );
        }
        return true;
    }
    if path == "/files/delete" {
        let (br, sub, item) = (fval("browse"), fval("sub"), fval("item"));
        if let Some((_, fp, rel_i)) = resolve_jailed_path(username, store, br, item) {
            if !rel_i.is_empty() {
                if fp.is_dir() {
                    let _ = fs::remove_dir_all(&fp);
                } else {
                    let _ = fs::remove_file(&fp);
                }
            }
        }
        write_redirect(
            stream,
            &format!("/?tab=1&browse={br}&sub={sub}&ok=Item+deleted"),
            "",
        );
        return true;
    }
    if path == "/shares/toggle" {
        let (br, sub, item) = (fval("browse"), fval("sub"), fval("item"));
        let (s_br, i_rel) = if br.is_empty() {
            (item, "")
        } else {
            (br, item)
        };
        if let Some((_, _, clean_item)) = resolve_jailed_path(username, store, s_br, i_rel) {
            let scope = if let Some(pf) = s_br
                .strip_prefix("personal:")
                .or_else(|| s_br.strip_prefix("personal/"))
            {
                format!("personal:{username}:{pf}")
            } else {
                s_br.to_string()
            };
            let target = if clean_item.is_empty() {
                scope
            } else {
                format!("{scope}/{clean_item}")
            };
            let tok = compute_share_token(&target);
            if store.public_shares.remove(&tok).is_none() {
                store.public_shares.insert(tok, target);
            }
            let _ = store.save(&cfg.portal.database_path);
            let u = format!("/?tab=1&browse={br}&sub={sub}&ok=Share+mode+updated");
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
                write_redirect(stream, "/?tab=2&ok=Spoke+provisioned+across+fleet", "");
            }
            Err(e) => {
                let p = render_dashboard_page(
                    username,
                    is_admin,
                    store,
                    snap,
                    2,
                    "",
                    "",
                    None,
                    Some(&e),
                );
                write_html(stream, "400 Bad Request", &p);
            }
        }
        return true;
    }
    if path == "/devices/remove" {
        if let Ok(norm_id) = normalize_and_verify_device_id(raw_id) {
            if store.remove_device(username, &norm_id).unwrap_or(false) {
                let _ = store.save(&cfg.portal.database_path);
                let sid = short_id(&norm_id);
                for fname in store.shared_folders.keys() {
                    let flow = fname.to_ascii_lowercase();
                    for cfid in [
                        format!("shared-{flow}-{sid}"),
                        syncthing_folder_id(&format!("shared:{flow}:{sid}")),
                    ] {
                        purge_folder_from_fleet(cfg, &snap.nodes, &cfid);
                    }
                }
                for f in store.folders_for_user(username) {
                    let flow = f.to_ascii_lowercase();
                    for cfid in [
                        format!("user-{username}-{flow}-{sid}"),
                        syncthing_folder_id(&format!("user:{username}:{flow}:{sid}")),
                    ] {
                        purge_folder_from_fleet(cfg, &snap.nodes, &cfid);
                    }
                }
                purge_device_from_fleet(cfg, &snap.nodes, &norm_id);
                let devs = store.users.get(username).map_or(0, Vec::len);
                for f in store.folders_for_user(username) {
                    let k = format!("{username}:{f}");
                    snap.device_accepted
                        .remove(&(format!("user:{k}"), norm_id.clone()));
                    if devs == 0 {
                        snap.user_accepted.remove(&k);
                    }
                }
                for fname in store.shared_folders.keys() {
                    snap.device_accepted
                        .remove(&(format!("shared:{fname}"), norm_id.clone()));
                    if devs == 0 {
                        snap.shared_user_accepted
                            .remove(&(fname.clone(), username.to_string()));
                    }
                }
            }
        }
        write_redirect(stream, "/?tab=2&ok=Spoke+removed+from+fleet", "");
        return true;
    }
    if is_admin {
        return handle_admin_post_action(stream, path, body, form, username, cfg, store, snap);
    }
    false
}
