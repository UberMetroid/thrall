//! Dark-mode HTML/SVG UI renderer & Web File Explorer for THRALL.

use crate::admin_console::theme::{body_attrs, theme_css};
use crate::device_store::{syncthing_folder_id, MeshStore};
use crate::fleet_config::FleetConfig;
use crate::reconcile_mesh::FleetSnapshot;

pub mod about_ldap;
pub mod admin_actions;
pub mod explorer_view;
pub mod mesh_stage;
pub mod portal_actions;
pub mod reconcile_folders;
pub mod share_jail;
pub mod url;

pub const PORTAL_CSS: &str = include_str!("../../assets/portal.css");

pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn render_device_identicon_svg(device_id: &str) -> String {
    let (bytes, mut rects) = (device_id.as_bytes(), String::new());
    for row in 0..9usize {
        for col in 0..9usize {
            let f = (row < 2 && col < 2) || (row < 2 && col >= 7) || (row >= 7 && col < 2);
            let b = bytes
                .get((row * 9 + col) % bytes.len().max(1))
                .copied()
                .unwrap_or(0);
            if f || ((b.wrapping_add((row ^ col) as u8)) & 1 == 1) {
                let (x, y) = (col * 6, row * 6);
                rects.push_str(&format!(
                    "<rect x=\"{x}\" y=\"{y}\" width=\"5\" height=\"5\" fill=\"#0b0e14\"/>"
                ));
            }
        }
    }
    format!(
        "<div class=\"qr-grid\"><svg width=\"54\" height=\"54\" viewBox=\"0 0 54 54\" xmlns=\"http://www.w3.org/2000/svg\">{rects}</svg></div>"
    )
}

fn fmt_rate(bps: u64) -> String {
    match bps {
        1_000_000.. => format!("{:.1} Mbps", bps as f64 / 1_000_000.0),
        1_000.. => format!("{:.1} kbps", bps as f64 / 1_000.0),
        _ => format!("{bps} bps"),
    }
}

#[rustfmt::skip]
pub fn render_telemetry_json(snap: &FleetSnapshot, store: &MeshStore) -> String {
    let on = snap.nodes.iter().filter(|n| n.online).count();
    let tot = snap.nodes.len().max(1);
    let conns: u64 = snap.nodes.iter().map(|n| n.active_conns).sum();
    let all_devs: Vec<_> = store.users.values().flatten().collect();
    let mini = all_devs.len();
    let parents = all_devs.iter().enumerate().map(|(di, d)| snap.nodes.iter().position(|n| n.connected_ids.contains(&d.device_id)).unwrap_or(di % tot).to_string()).collect::<Vec<_>>().join(",");
    let rx = fmt_rate(snap.nodes.iter().map(|n| n.in_bps).sum());
    let tx = fmt_rate(snap.nodes.iter().map(|n| n.out_bps).sum());
    let up_h = snap.nodes.iter().map(|n| n.uptime_sec).max().unwrap_or(0) / 3600;
    let roster: Vec<_> = snap.nodes.iter().collect();
    let series = mesh_stage::hub_series(&roster);
    let flags = mesh_stage::online_flags_csv(snap);
    format!("{{\"hubs_online\":{on},\"hubs_total\":{tot},\"spokes\":{mini},\"active_links\":{conns},\"uptime_h\":{up_h},\"rx\":\"{rx}\",\"tx\":\"{tx}\",\"online\":[{flags}],\"rtts\":[{rtts}],\"bps\":[{bps}],\"masses\":[{masses}],\"bearings\":[{bearings}],\"parents\":[{parents}]}}\n", rtts = series.rtts, bps = series.bps, masses = series.masses, bearings = series.bearings)
}

#[rustfmt::skip]
pub fn render_login_page(flash_err: Option<&str>, snap: &FleetSnapshot, store: &MeshStore) -> String {
    let status_badge = if let Some(err) = flash_err.filter(|m| !m.is_empty()) {
        let err_esc = escape_html(err);
        format!("<span id=\"footer-status-msg\" class=\"footer-toast toast-err\">{err_esc}</span>")
    } else {
        "<span id=\"footer-status-msg\" class=\"footer-toast\" style=\"display:none\"></span>".to_string()
    };
    let hubs = mesh_stage::online_hubs(snap);
    let tot = hubs.len();
    let roster_total = snap.nodes.len();
    let all_devs: Vec<_> = store.users.values().flatten().collect();
    let mini = all_devs.len();
    let dev_ids: Vec<String> = all_devs.iter().map(|d| d.device_id.clone()).collect();
    let parents = mesh_stage::device_parents(snap, &dev_ids);
    let roster: Vec<_> = snap.nodes.iter().collect();
    let series = mesh_stage::hub_series(&roster);
    let flags = mesh_stage::online_flags_csv(snap);
    let (rtts, bps_list, mass_list, brg_list) = (series.rtts, series.bps, series.masses, series.bearings);
    let hdr = "<header class=\"site-header\" id=\"site-header\"><div class=\"site-header-inner\"><a href=\"#\" data-modal-open=\"about-thrall\" class=\"auth-brand-row\" style=\"text-decoration:none;color:inherit\" title=\"About THRALL\"><img src=\"/icon.jpg\" alt=\"THRALL\" width=\"34\" height=\"34\"><div><canvas id=\"header-wordmark\" class=\"header-wordmark\" width=\"140\" height=\"34\" aria-label=\"THRALL\"></canvas><div class=\"visually-hidden\">THRALL</div><div style=\"font-family:var(--mono);font-size:0.65rem;color:var(--muted)\">Tool for Hierarchical Replication And Identity Linking</div></div></a><div></div><form method=\"POST\" action=\"/login\" autocomplete=\"off\" class=\"header-login-form\"><input type=\"text\" name=\"username\" required autocomplete=\"off\" placeholder=\"User or Email\" aria-label=\"User or Email\"><input type=\"password\" name=\"password\" required autocomplete=\"off\" placeholder=\"Password\" aria-label=\"Password\"><button type=\"submit\">[ SIGN IN ]</button></form></div></header>";
    let head_links = "<link rel=\"icon\" href=\"/favicon.svg\" type=\"image/svg+xml\">";
    let dock = about_ldap::render_footer_dock(&status_badge, &store.settings);
    let theme = theme_css(&store.settings);
    let attrs = body_attrs(&store.settings, true);
    let canvas = format!("<canvas id=\"spotlight-canvas\" class=\"spotlight-canvas\" data-hubs-online=\"{tot}\" data-hubs-total=\"{roster_total}\" data-hubs-flags=\"{flags}\" data-devices=\"{mini}\" data-mesh-rtts=\"{rtts}\" data-mesh-bps=\"{bps_list}\" data-mesh-masses=\"{mass_list}\" data-mesh-bearings=\"{brg_list}\" data-device-parents=\"{parents}\" aria-hidden=\"true\"></canvas>");
    let canvas = if store.settings.spotlight { canvas } else { String::new() };
    format!("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">{head_links}<title>THRALL</title><style>{PORTAL_CSS}{theme}</style></head><body{attrs}>{canvas}{hdr}{dock}<script src=\"/login.js\"></script></body></html>")
}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
pub fn render_dashboard_page(
    username: &str,
    is_admin: bool,
    cfg: &FleetConfig,
    store: &MeshStore,
    snap: &FleetSnapshot,
    active_tab: u8,
    browse: &str,
    sub: &str,
    flash_ok: Option<&str>,
    flash_err: Option<&str>,
) -> String {
    let norm_tab = if matches!(active_tab, 4 | 5 | 7) { 4 } else { active_tab };
    let raw_tab = if matches!(norm_tab, 1..=3 | 6) || (is_admin && norm_tab == 4) { norm_tab } else { 1 };
    let tab = if raw_tab == 1 && browse.starts_with("shared:") { 2 } else if raw_tab == 2 && browse.starts_with("personal:") { 1 } else { raw_tab };
    let user_esc = escape_html(username);
    let footer_status = if let Some(err) = flash_err.filter(|s| !s.is_empty()) {
        format!("<span id=\"footer-status-msg\" class=\"footer-toast toast-err\">{}</span>", escape_html(err))
    } else if let Some(ok) = flash_ok.filter(|s| !s.is_empty()) {
        format!("<span id=\"footer-status-msg\" class=\"footer-toast\">{}</span>", escape_html(ok))
    } else {
        "<span id=\"footer-status-msg\" class=\"footer-toast\" style=\"display:none\"></span>".to_string()
    };
    let online_nodes: Vec<_> = snap.nodes.iter().filter(|n| n.online && !n.device_id.is_empty()).collect();

    let pane = match tab {
        3 => {
            let gw = online_nodes.first().copied().or_else(|| snap.nodes.first());
            let dev_id = escape_html(gw.map(|g| g.device_id.as_str()).filter(|s| !s.is_empty()).unwrap_or("OFFLINE"));
            let qr = gw.filter(|g| !g.device_id.is_empty()).map(|g| render_device_identicon_svg(&g.device_id)).unwrap_or_default();
            let u_devs: Vec<_> = store.users.get(username).into_iter().flatten().collect();
            let u_folders = store.folders_for_user(username);
            let s_folders: Vec<_> = store.shared_folders.iter().filter(|(_, a)| a.contains(username)).map(|(f, _)| f.clone()).collect();
            let mut all_sf: Vec<String> = u_folders.iter().map(|f| format!("personal:{f}")).collect();
            all_sf.extend(s_folders.iter().map(|f| format!("shared:{f}")));
            let df_esc = escape_html(&all_sf.join(","));
            let mut rows = String::new();
            for dev in &u_devs {
                let (lbl, did) = (escape_html(&dev.label), escape_html(&dev.device_id));
                let sid = dev.device_id.split('-').next().unwrap_or("mesh").to_ascii_lowercase();
                let tkey = format!("device:{did}");
                let (mut acc_cnt, mut en_f, mut kids) = (0usize, 0usize, String::new());
                for f in &u_folders {
                    let (f_esc, flow, k, sf) = (escape_html(f), f.to_ascii_lowercase(), format!("{username}:{f}"), format!("personal:{f}"));
                    let (fid, lp_esc) = (escape_html(&syncthing_folder_id(&format!("user:{username}:{flow}:{sid}"))), escape_html(&store.local_path_for(username, &sf)));
                    let excl = store.is_device_folder_excluded(&dev.device_id, &sf);
                    let pct = online_nodes.iter().filter_map(|n| snap.user_completion.get(&(k.clone(), n.name.clone())).copied()).min().unwrap_or(100);
                    let ok = snap.device_accepted.get(&(format!("user:{k}"), dev.device_id.clone())).copied().unwrap_or(false);
                    if !excl { en_f += 1; if ok { acc_cnt += 1; } }
                    let st_b = if excl { "<span class=\"badge btn-secondary\" style=\"color:var(--muted)\">Cloud Only (Not Synced)</span>" } else if ok { "<span class=\"badge badge-online\">&#10003; accepted</span>" } else { "<span class=\"badge badge-warn\">&#9203; unaccepted</span> <a href=\"http://127.0.0.1:8384\" target=\"_blank\" rel=\"noopener\" style=\"color:var(--warn);font-size:0.74rem\">127.0.0.1:8384</a>" };
                    let (s_cls, s_lbl) = if excl { ("btn-secondary", "Sync: OFF") } else { ("badge-online", "Sync: ON") };
                    let s_btn = format!("<form method=\"POST\" action=\"/devices/folder-sync\" style=\"display:inline\"><input type=\"hidden\" name=\"device_id\" value=\"{did}\"><input type=\"hidden\" name=\"folder\" value=\"personal:{f_esc}\"><button type=\"submit\" class=\"{s_cls}\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\">{s_lbl}</button></form>");
                    let p_btn = format!("{s_btn} <button type=\"button\" class=\"btn-secondary\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\" data-modal-open=\"set-folder-path\" data-folder=\"personal:{f_esc}\" data-path=\"{lp_esc}\">Local Path</button>");
                    kids.push_str(&format!("<tr class=\"tree-child\" data-parent=\"{tkey}\" style=\"display:none\"><td style=\"padding-left:1.75rem\"><span class=\"tree-branch\">&#9492;&#9472;</span><a href=\"/?tab=personal&amp;browse=personal:{f_esc}\" style=\"color:var(--accent);text-decoration:none;font-weight:600\">&#128193; Personal / {f_esc}/</a></td><td><code>{fid}</code> &bull; <code>{lp_esc}</code> &bull; {pct}%</td><td>{st_b}</td><td style=\"text-align:right\">{p_btn}</td></tr>"));
                }
                for fname in &s_folders {
                    let (f_esc, flow, sf) = (escape_html(fname), fname.to_ascii_lowercase(), format!("shared:{fname}"));
                    let (fid, lp_esc) = (escape_html(&syncthing_folder_id(&format!("shared:{flow}:{sid}"))), escape_html(&store.local_path_for(username, &sf)));
                    let excl = store.is_device_folder_excluded(&dev.device_id, &sf);
                    let pct = snap.shared_completion.get(fname).copied().unwrap_or(100);
                    let ok = snap.device_accepted.get(&(format!("shared:{fname}"), dev.device_id.clone())).copied().unwrap_or(false);
                    if !excl { en_f += 1; if ok { acc_cnt += 1; } }
                    let st_b = if excl { "<span class=\"badge btn-secondary\" style=\"color:var(--muted)\">Cloud Only (Not Synced)</span>" } else if ok { "<span class=\"badge badge-online\">&#10003; accepted</span>" } else { "<span class=\"badge badge-warn\">&#9203; unaccepted</span> <a href=\"http://127.0.0.1:8384\" target=\"_blank\" rel=\"noopener\" style=\"color:var(--warn);font-size:0.74rem\">127.0.0.1:8384</a>" };
                    let (s_cls, s_lbl) = if excl { ("btn-secondary", "Sync: OFF") } else { ("badge-online", "Sync: ON") };
                    let s_btn = format!("<form method=\"POST\" action=\"/devices/folder-sync\" style=\"display:inline\"><input type=\"hidden\" name=\"device_id\" value=\"{did}\"><input type=\"hidden\" name=\"folder\" value=\"shared:{f_esc}\"><button type=\"submit\" class=\"{s_cls}\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\">{s_lbl}</button></form>");
                    let p_btn = format!("{s_btn} <button type=\"button\" class=\"btn-secondary\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\" data-modal-open=\"set-folder-path\" data-folder=\"shared:{f_esc}\" data-path=\"{lp_esc}\">Local Path</button>");
                    kids.push_str(&format!("<tr class=\"tree-child\" data-parent=\"{tkey}\" style=\"display:none\"><td style=\"padding-left:1.75rem\"><span class=\"tree-branch\">&#9492;&#9472;</span><a href=\"/?tab=shared&amp;browse=shared:{f_esc}\" style=\"color:var(--purple);text-decoration:none;font-weight:600\">&#127760; Shared / {f_esc}/</a></td><td><code>{fid}</code> &bull; <code>{lp_esc}</code> &bull; {pct}%</td><td>{st_b}</td><td style=\"text-align:right\">{p_btn}</td></tr>"));
                }
                if kids.is_empty() {
                    kids.push_str(&format!("<tr class=\"tree-child\" data-parent=\"{tkey}\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:1.75rem;color:var(--muted);font-size:0.8rem\"><span class=\"tree-branch\">&#9492;&#9472;</span>0 folders provisioned yet.</td></tr>"));
                }
                let sync_b = if en_f == 0 || acc_cnt == en_f { format!("<span class=\"badge badge-online\">{acc_cnt}/{en_f} Accepted</span>") } else { format!("<span class=\"badge badge-warn\">{acc_cnt}/{en_f} Accepted</span>") };
                let tbtn = format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{tkey}\" aria-expanded=\"false\" aria-label=\"Expand {lbl}\">+</button>");
                let rev_b = format!("<button type=\"button\" class=\"btn-danger\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\" data-del-action=\"/devices/remove\" data-del-name=\"{lbl}\" data-del-item=\"{did}\">Revoke</button>");
                rows.push_str(&format!("<tr><td>{tbtn}<strong>&#128187; {lbl}</strong></td><td><code style=\"font-family:var(--mono);font-size:0.8rem\">{did}</code></td><td>{sync_b}</td><td style=\"text-align:right\">{rev_b}</td></tr>{kids}"));
            }
            if rows.is_empty() {
                rows.push_str("<tr><td colspan=\"4\" style=\"color:var(--muted)\">0 enrolled devices. Click <strong>+ Add Device</strong> above to pair a workstation.</td></tr>");
                for f in &u_folders {
                    let (f_esc, flow, sf) = (escape_html(f), f.to_ascii_lowercase(), format!("personal:{f}"));
                    let (fid, lp_esc) = (escape_html(&syncthing_folder_id(&format!("user:{username}:{flow}:mesh"))), escape_html(&store.local_path_for(username, &sf)));
                    let p_btn = format!("<button type=\"button\" class=\"btn-secondary\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\" data-modal-open=\"set-folder-path\" data-folder=\"personal:{f_esc}\" data-path=\"{lp_esc}\">Local Path</button>");
                    rows.push_str(&format!("<tr class=\"pending-folder\"><td><a href=\"/?tab=personal&amp;browse=personal:{f_esc}\" style=\"color:var(--accent);text-decoration:none;font-weight:600\">&#128193; Personal / {f_esc}/</a></td><td><code>{fid}</code> &bull; <code>{lp_esc}</code></td><td><span class=\"badge badge-warn\">NO DEVICES</span></td><td style=\"text-align:right\">{p_btn}</td></tr>"));
                }
                for fname in &s_folders {
                    let (f_esc, flow, sf) = (escape_html(fname), fname.to_ascii_lowercase(), format!("shared:{fname}"));
                    let (fid, lp_esc) = (escape_html(&syncthing_folder_id(&format!("shared:{flow}:mesh"))), escape_html(&store.local_path_for(username, &sf)));
                    let p_btn = format!("<button type=\"button\" class=\"btn-secondary\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\" data-modal-open=\"set-folder-path\" data-folder=\"shared:{f_esc}\" data-path=\"{lp_esc}\">Local Path</button>");
                    rows.push_str(&format!("<tr class=\"pending-folder\"><td><a href=\"/?tab=shared&amp;browse=shared:{f_esc}\" style=\"color:var(--purple);text-decoration:none;font-weight:600\">&#127760; Shared / {f_esc}/</a></td><td><code>{fid}</code> &bull; <code>{lp_esc}</code></td><td><span class=\"badge badge-warn\">NO DEVICES</span></td><td style=\"text-align:right\">{p_btn}</td></tr>"));
                }
            }
            format!("<table class=\"explorer-table\"><thead><tr><th>Device &nbsp;<button type=\"button\" class=\"btn-secondary\" style=\"padding:0.14rem 0.48rem;font-size:0.72rem;text-transform:none\" data-modal-open=\"enroll-device\" data-folders=\"{df_esc}\">+ Add Device</button></th><th>Device ID / Local Path</th><th style=\"width:210px\">Folders Synced</th><th style=\"width:190px;text-align:right\">Action</th></tr></thead><tbody>{rows}</tbody></table><template id=\"tmpl-enroll-device\"><div class=\"mono\" id=\"gw-dev\" style=\"user-select:all\">{dev_id}</div>{qr}</template>")
        }
        6 => about_ldap::render_about_pane(),
        4 if is_admin => explorer_view::render_admin_pane(cfg, store, snap),
        _ => explorer_view::render_file_explorer(username, &cfg.portal.session_secret, store, tab, browse, sub),
    };
    let c = |n: u8, adm: bool| match (tab == n, adm) {
        (true, true) => "body-tab admin-tab active",
        (false, true) => "body-tab admin-tab",
        (true, false) => "body-tab active",
        (false, false) => "body-tab",
    };
    let (c1, c2, c3, c4) = (c(1, false), c(2, false), c(3, false), c(4, true));
    let admin_link = if is_admin { format!("<div class=\"body-tabs-right\"><a href=\"/?tab=admin\" class=\"{c4}\">Admin</a></div>") } else { String::new() };
    let body_tabs = format!("<nav class=\"body-tabs\" aria-label=\"Portal Navigation\"><div class=\"body-tabs-left\"><a href=\"/?tab=personal\" class=\"{c1}\">Personal</a><a href=\"/?tab=shared\" class=\"{c2}\">Shared</a><a href=\"/?tab=devices\" class=\"{c3}\">Devices</a></div>{admin_link}</nav>");
    let footer = about_ldap::render_footer_dock(&footer_status, &store.settings);
    let theme = theme_css(&store.settings);
    let attrs = body_attrs(&store.settings, false);
    format!("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><link rel=\"icon\" href=\"/favicon.svg\" type=\"image/svg+xml\"><title>THRALL — Tool for Hierarchical Replication And Identity Linking ({user_esc})</title><style>{PORTAL_CSS}{theme}</style></head><body{attrs}><header class=\"site-header\" id=\"site-header\"><div class=\"site-header-inner\"><a href=\"/?tab=about\" class=\"auth-brand-row\" style=\"text-decoration:none;color:inherit\" title=\"About THRALL\"><img src=\"/icon.jpg\" alt=\"THRALL\" width=\"34\" height=\"34\"><div><canvas id=\"header-wordmark\" class=\"header-wordmark\" width=\"140\" height=\"34\" aria-label=\"THRALL\"></canvas><div class=\"visually-hidden\">THRALL — Tool for Hierarchical Replication And Identity Linking</div><div style=\"font-family:var(--mono);font-size:0.65rem;color:var(--muted)\">Tool for Hierarchical Replication And Identity Linking</div></div></a><div></div><form method=\"POST\" action=\"/logout\" autocomplete=\"off\" class=\"header-login-form\"><button type=\"submit\">[ SIGN OUT ]</button></form></div></header><div class=\"middle-scroll\" id=\"middle-scroll\"><div class=\"container\">{body_tabs}{pane}</div></div>{footer}<script src=\"/login.js\"></script></body></html>")
}
