//! Per-server Syncthing folder provisioning and remote acceptance verification for THRALL.

use crate::device_store::{sanitize_device_label, syncthing_folder_id, MeshStore};
use crate::fleet_config::ServerNodeConfig;
use crate::peer_client::{
    parse_json_string_field, parse_json_u64_field, prepare_folder, syncthing_rest,
};
use crate::reconcile_mesh::{build_device_json, build_folder_json, FleetSnapshot};

#[allow(clippy::too_many_arguments)]
fn sync_folder(
    srv: &ServerNodeConfig,
    key: &str,
    fid: &str,
    flbl: &str,
    cp: &str,
    devs: &[String],
    ro: bool,
) -> u64 {
    // A peer that shares the host also shares its storage, so the folder
    // directory is prepared here. A remote peer creates the folder path
    // itself when the folder configuration is applied, so there is nothing
    // to prepare and nothing to reach the host for.
    let _ = prepare_folder(srv, cp);
    let f_json = build_folder_json(fid, flbl, cp, devs, ro);
    let put_ep = format!("/rest/config/folders/{fid}");
    let scan_ep = format!("/rest/db/scan?folder={fid}");
    let comp_ep = format!("/rest/db/completion?folder={fid}");
    let _ = syncthing_rest(srv, key, "PUT", &put_ep, Some(&f_json));
    let _ = syncthing_rest(srv, key, "POST", &scan_ep, None);
    if ro {
        let _ = syncthing_rest(
            srv,
            key,
            "POST",
            &format!("/rest/db/override?folder={fid}"),
            None,
        );
    }
    let res = syncthing_rest(srv, key, "GET", &comp_ep, None);
    res.ok()
        .and_then(|c| parse_json_u64_field(&c, "completion"))
        .unwrap_or(100)
}

fn is_accepted(srv: &ServerNodeConfig, key: &str, fid: &str, dev_id: &str) -> bool {
    let url = format!("/rest/db/completion?folder={fid}&device={dev_id}");
    let res = syncthing_rest(srv, key, "GET", &url, None);
    res.ok()
        .and_then(|c| parse_json_string_field(&c, "remoteState"))
        .as_deref()
        == Some("valid")
}

fn short_id(dev_id: &str) -> String {
    dev_id
        .split('-')
        .next()
        .unwrap_or("dev")
        .to_ascii_lowercase()
}

type AccList = Vec<(String, bool)>;

#[rustfmt::skip]
pub fn sync_folder_group(srv: &ServerNodeConfig, key: &str, online_ids: &[String], store: &MeshStore, user: Option<&str>, fname: &str, check_acc: bool) -> (u64, AccList, AccList) {
    let flow = fname.to_ascii_lowercase();
    let (mfid, flbl, cp, scope_f, users): (_, _, _, _, Vec<String>) = if let Some(u) = user {
        (syncthing_folder_id(&format!("user:{u}:{flow}:mesh")), format!("{u}/{fname}"), format!("/users/{u}/{fname}"), format!("personal:{fname}"), vec![u.to_string()])
    } else {
        let r = fname == "Shared";
        let flbl = if r { "Shared".into() } else { format!("Shared ({fname})") };
        let cp = if r { "/shared".into() } else { format!("/shared/{fname}") };
        let us = store.shared_folders.get(fname).into_iter().flatten().cloned().collect();
        (syncthing_folder_id(&format!("shared:{flow}:mesh")), flbl, cp, format!("shared:{fname}"), us)
    };
    let mut pct = sync_folder(srv, key, &mfid, &flbl, &cp, online_ids, false);
    let (mut u_acc, mut d_acc) = (Vec::new(), Vec::new());
    for u in users {
        let ro = user.is_none() && store.is_shared_readonly(fname, &u);
        let devs: Vec<_> = store.users.get(&u).into_iter().flatten().collect();
        let (mut en_cnt, mut all_ok) = (0usize, true);
        for d in devs {
            let sid = short_id(&d.device_id);
            let cfid = if user.is_some() { syncthing_folder_id(&format!("user:{u}:{flow}:{sid}")) } else { syncthing_folder_id(&format!("shared:{flow}:{sid}")) };
            if store.is_device_folder_excluded(&d.device_id, &scope_f) {
                let _ = syncthing_rest(srv, key, "DELETE", &format!("/rest/config/folders/{cfid}"), None);
                continue;
            }
            en_cnt += 1;
            let mut cdevs = online_ids.to_vec();
            if !cdevs.contains(&d.device_id) {
                cdevs.push(d.device_id.clone());
            }
            pct = pct.min(sync_folder(srv, key, &cfid, &flbl, &cp, &cdevs, ro));
            if check_acc {
                let ok = is_accepted(srv, key, &cfid, &d.device_id);
                all_ok &= ok;
                d_acc.push((d.device_id.clone(), ok));
            }
        }
        u_acc.push((u, en_cnt > 0 && all_ok));
    }
    (pct, u_acc, d_acc)
}

fn discover_device_labels(srv: &ServerNodeConfig, key: &str, store: &mut MeshStore) {
    let _ = syncthing_rest(
        srv,
        key,
        "PATCH",
        "/rest/config/options",
        Some("{\"overwriteRemoteDeviceNamesOnConnect\":true}"),
    );
    let ev = syncthing_rest(srv, key, "GET", "/rest/events?timeout=0&limit=1000", None)
        .unwrap_or_default();
    let cfg = syncthing_rest(srv, key, "GET", "/rest/config/devices", None).unwrap_or_default();
    let pend =
        syncthing_rest(srv, key, "GET", "/rest/cluster/pending/devices", None).unwrap_or_default();
    for udevs in store.users.values_mut() {
        for d in udevs.iter_mut().filter(|d| d.label.starts_with("dev-")) {
            let mut found = None;
            for src in [&ev, &pend, &cfg] {
                for (pos, _) in src.match_indices(&d.device_id) {
                    let s = src.floor_char_boundary(pos.saturating_sub(220));
                    let e = src.ceil_char_boundary((pos + d.device_id.len() + 220).min(src.len()));
                    let win = &src[s..e];
                    if let Some(n) = parse_json_string_field(win, "deviceName")
                        .or_else(|| parse_json_string_field(win, "name"))
                    {
                        if !n.is_empty() && !n.contains("dev-") && n.len() < 50 {
                            found = sanitize_device_label(&n).ok();
                        }
                    }
                }
            }
            if let Some(lbl) = found {
                d.label = lbl;
            }
        }
    }
}

pub fn reconcile_server_node(
    srv: &ServerNodeConfig,
    key: &str,
    online_ids: &[String],
    folder_names: &[String],
    store: &mut MeshStore,
    snap: &mut FleetSnapshot,
) {
    // The peer manages its own firewall. THRALL never reaches into a host to
    // change its network rules.
    discover_device_labels(srv, key, store);
    for p in snap
        .nodes
        .iter()
        .filter(|p| p.online && !p.device_id.is_empty())
    {
        let j = build_device_json(&p.device_id, &p.name, &p.wan_addr, true);
        let ep = format!("/rest/config/devices/{}", p.device_id);
        let _ = syncthing_rest(srv, key, "PUT", &ep, Some(&j));
    }
    for udevs in store.users.values() {
        for d in udevs {
            let j = build_device_json(&d.device_id, &d.label, "dynamic", false);
            let ep = format!("/rest/config/devices/{}", d.device_id);
            let _ = syncthing_rest(srv, key, "PUT", &ep, Some(&j));
        }
    }
    for fname in folder_names {
        let (pct, u_acc, d_acc) = sync_folder_group(srv, key, online_ids, store, None, fname, true);
        for (u, _) in u_acc.into_iter().filter(|(_, a)| *a) {
            snap.shared_user_accepted.insert((fname.clone(), u), true);
        }
        for (did, _) in d_acc.into_iter().filter(|(_, a)| *a) {
            snap.device_accepted
                .insert((format!("shared:{fname}"), did), true);
        }
        snap.shared_completion.insert(fname.clone(), pct);
    }
    let mut all_users: Vec<String> = snap.ldap_users.clone();
    for u in store.users.keys() {
        if !all_users.contains(u) {
            all_users.push(u.clone());
        }
    }
    for u in &all_users {
        for f in store.folders_for_user(u) {
            let (pct, u_acc, d_acc) =
                sync_folder_group(srv, key, online_ids, store, Some(u), &f, true);
            let k = format!("{u}:{f}");
            for (did, _) in d_acc.into_iter().filter(|(_, a)| *a) {
                snap.device_accepted
                    .insert((format!("user:{k}"), did), true);
            }
            let cur = snap
                .user_completion
                .get(&(k.clone(), srv.name.clone()))
                .copied()
                .unwrap_or(100);
            snap.user_completion
                .insert((k.clone(), srv.name.clone()), cur.min(pct));
            if u_acc.iter().any(|(_, a)| *a) {
                snap.user_accepted.insert(k, true);
            }
        }
    }
}

#[rustfmt::skip]
pub fn trigger_web_scan(cfg: &crate::fleet_config::FleetConfig, snap: &FleetSnapshot, store: &MeshStore, user: &str, br: &str, sub: &str) {
    let mfid = if let Some(sh) = br.strip_prefix("shared:") {
        syncthing_folder_id(&format!("shared:{}:mesh", sh.to_ascii_lowercase()))
    } else {
        let pf = br.strip_prefix("personal:").or_else(|| br.strip_prefix("personal/")).map(String::from).unwrap_or_else(|| store.folders_for_user(user).into_iter().next().unwrap_or_default());
        syncthing_folder_id(&format!("user:{user}:{}:mesh", pf.to_ascii_lowercase()))
    };
    let rel = crate::device_store::sanitize_rel_subpath(sub).unwrap_or_default();
    if let Some((srv, st)) = cfg.servers.iter().zip(snap.nodes.iter()).find(|(_, s)| s.online).map(|(srv, s)| (srv.clone(), s.api_key.clone())) {
        // Percent-encode the sub path: it is a query-string component, and it
        // must not be able to introduce a separator or a shell metacharacter.
        let ep = if rel.is_empty() { format!("/rest/db/scan?folder={mfid}") } else { format!("/rest/db/scan?folder={mfid}&sub={}", crate::render_portal::url::percent_encode(&rel)) };
        std::thread::spawn(move || { let _ = syncthing_rest(&srv, &st, "POST", &ep, None); });
    }
}
