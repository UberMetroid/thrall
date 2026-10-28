//! Dark-mode HTML/SVG UI renderer & Web File Explorer for THRALL (`st-mesh-mgr`).

use crate::device_store::{syncthing_folder_id, MeshStore};
use crate::reconcile_mesh::FleetSnapshot;

pub mod admin_actions;
pub mod explorer_view;
pub mod portal_actions;
pub mod reconcile_folders;
pub mod share_jail;

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
    let (mut rtts, mut bps_list, mut mass_list, mut brg_list) = (String::new(), String::new(), String::new(), String::new());
    for idx in 0..tot {
        let n = snap.nodes.get(idx);
        let (in_b, out_b) = (n.map(|x| x.in_bps).unwrap_or(0), n.map(|x| x.out_bps).unwrap_or(0));
        let rtt = n.map(|x| x.rtt_ms).filter(|&r| r > 0).unwrap_or((idx as u64 + 1) * 14);
        let mass = n.map(|x| x.total_bytes).filter(|&m| m > 0).unwrap_or((idx as u64 + 1) * 1024);
        let brg = n.map(|x| x.bearing_deg).unwrap_or((idx as u64 * 120) % 360);
        if !rtts.is_empty() { rtts.push(','); bps_list.push(','); mass_list.push(','); brg_list.push(','); }
        rtts.push_str(&rtt.to_string());
        bps_list.push_str(&(in_b + out_b).to_string());
        mass_list.push_str(&mass.to_string());
        brg_list.push_str(&brg.to_string());
    }
    format!("{{\"hubs_online\":{on},\"hubs_total\":{tot},\"spokes\":{mini},\"active_links\":{conns},\"uptime_h\":{up_h},\"rx\":\"{rx}\",\"tx\":\"{tx}\",\"rtts\":[{rtts}],\"bps\":[{bps_list}],\"masses\":[{mass_list}],\"bearings\":[{brg_list}],\"parents\":[{parents}]}}\n")
}

#[rustfmt::skip]
pub fn render_login_page(flash_err: Option<&str>, snap: &FleetSnapshot, store: &MeshStore) -> String {
    let status_badge = if let Some(err) = flash_err.filter(|m| !m.is_empty()) {
        let err_esc = escape_html(err);
        format!("<span id=\"footer-status-msg\" class=\"footer-toast toast-err\">{err_esc}</span>")
    } else {
        "<span id=\"footer-status-msg\" class=\"footer-toast\" style=\"display:none\"></span>".to_string()
    };
    let tot = snap.nodes.len().max(1);
    let all_devs: Vec<_> = store.users.values().flatten().collect();
    let mini = all_devs.len();
    let parents: String = all_devs.iter().enumerate().map(|(di, d)| snap.nodes.iter().position(|n| n.connected_ids.contains(&d.device_id)).unwrap_or(di % tot).to_string()).collect::<Vec<_>>().join(",");
    let (mut rtts, mut bps_list, mut mass_list, mut brg_list) = (String::new(), String::new(), String::new(), String::new());
    for idx in 0..tot {
        let n = snap.nodes.get(idx);
        let (in_b, out_b) = (n.map(|x| x.in_bps).unwrap_or(0), n.map(|x| x.out_bps).unwrap_or(0));
        let rtt = n.map(|x| x.rtt_ms).filter(|&r| r > 0).unwrap_or((idx as u64 + 1) * 14);
        let mass = n.map(|x| x.total_bytes).filter(|&m| m > 0).unwrap_or((idx as u64 + 1) * 1024);
        let brg = n.map(|x| x.bearing_deg).unwrap_or((idx as u64 * 120) % 360);
        if !rtts.is_empty() { rtts.push(','); bps_list.push(','); mass_list.push(','); brg_list.push(','); }
        rtts.push_str(&rtt.to_string());
        bps_list.push_str(&(in_b + out_b).to_string());
        mass_list.push_str(&mass.to_string());
        brg_list.push_str(&brg.to_string());
    }
    let hdr = "<header class=\"site-header\" id=\"site-header\"><div class=\"site-header-inner\"><div class=\"auth-brand-row\"><img src=\"/icon.jpg\" alt=\"THRALL\" width=\"34\" height=\"34\"><div><canvas id=\"header-wordmark\" class=\"header-wordmark\" width=\"140\" height=\"34\" aria-label=\"THRALL\"></canvas><div class=\"visually-hidden\">THRALL</div><div style=\"font-family:var(--mono);font-size:0.65rem;color:var(--muted)\">Tool for Hierarchical Replication And Identity Linking</div></div></div><div></div><form method=\"POST\" action=\"/login\" autocomplete=\"off\" class=\"header-login-form\"><input type=\"text\" name=\"username\" required autocomplete=\"off\" placeholder=\"User or Email\" aria-label=\"User or Email\"><input type=\"password\" name=\"password\" required autocomplete=\"off\" placeholder=\"Password\" aria-label=\"Password\"><button type=\"submit\">[ SIGN IN ]</button></form></div></header>";
    let head_links = "<link rel=\"icon\" href=\"/favicon.svg\" type=\"image/svg+xml\">";
    let dock = format!("<footer class=\"footer-dock\" id=\"footer-dock\" aria-label=\"Theme Palette\"><div class=\"footer-dock-inner\"><div class=\"footer-left-slot\">{status_badge}</div><div></div><div class=\"dock-controls\"><canvas id=\"footer-upload-graph\" class=\"footer-upload-graph\" width=\"128\" height=\"24\" title=\"Browser &lt;-&gt; Mesh Data Flow\" aria-label=\"Browser &lt;-&gt; Mesh Data Flow\"></canvas><div class=\"footer-accent-picker\" id=\"footer-accent-picker\" role=\"radiogroup\" aria-label=\"8-Bit Rainbow Palette\"><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Coral\" style=\"--swatch:#ff5555;background-color:#ff5555;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Coral\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Orange\" style=\"--swatch:#ff7700;background-color:#ff7700;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Orange\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Amber\" style=\"--swatch:#ffb000;background-color:#ffb000;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Amber\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Yellow\" style=\"--swatch:#f1fa8c;background-color:#f1fa8c;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Yellow\"></button><button type=\"button\" class=\"dock-accent-dot active\" data-accent-name=\"Emerald\" style=\"--swatch:#50fa7b;background-color:#50fa7b;\" role=\"radio\" aria-checked=\"true\" tabindex=\"0\" aria-label=\"Emerald\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Cyan\" style=\"--swatch:#00e5ff;background-color:#00e5ff;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Cyan\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Blue\" style=\"--swatch:#3b82f6;background-color:#3b82f6;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Blue\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Violet\" style=\"--swatch:#bd93f9;background-color:#bd93f9;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Violet\"></button></div></div></div></footer>");
    format!("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">{head_links}<title>THRALL</title><style>{PORTAL_CSS}</style></head><body><canvas id=\"spotlight-canvas\" class=\"spotlight-canvas\" data-mesh-nodes=\"{tot}\" data-mini-nodes=\"{mini}\" data-mesh-rtts=\"{rtts}\" data-mesh-bps=\"{bps_list}\" data-mesh-masses=\"{mass_list}\" data-mesh-bearings=\"{brg_list}\" data-mini-parents=\"{parents}\" aria-hidden=\"true\"></canvas>{hdr}{dock}<script src=\"/login.js\"></script></body></html>")
}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
pub fn render_dashboard_page(
    username: &str,
    is_admin: bool,
    store: &MeshStore,
    snap: &FleetSnapshot,
    active_tab: u8,
    browse: &str,
    sub: &str,
    flash_ok: Option<&str>,
    flash_err: Option<&str>,
) -> String {
    let raw_tab = if (1..=3).contains(&active_tab) || (is_admin && (4..=5).contains(&active_tab)) {
        active_tab
    } else {
        1
    };
    let tab = if raw_tab == 1 && browse.starts_with("shared:") {
        2
    } else if raw_tab == 2 && browse.starts_with("personal:") {
        1
    } else {
        raw_tab
    };
    let user_esc = escape_html(username);
    let footer_status = if let Some(err) = flash_err.filter(|s| !s.is_empty()) {
        let err_esc = escape_html(err);
        format!("<span id=\"footer-status-msg\" class=\"footer-toast toast-err\">{err_esc}</span>")
    } else if let Some(ok) = flash_ok.filter(|s| !s.is_empty()) {
        let ok_esc = escape_html(ok);
        format!("<span id=\"footer-status-msg\" class=\"footer-toast\">{ok_esc}</span>")
    } else {
        "<span id=\"footer-status-msg\" class=\"footer-toast\" style=\"display:none\"></span>".to_string()
    };
    let online_nodes: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.online && !n.device_id.is_empty())
        .collect();

    let pane = match tab {
        3 => {
            let gw = online_nodes.first().copied().or_else(|| snap.nodes.first());
            let dev_id = escape_html(gw.map(|g| g.device_id.as_str()).filter(|s| !s.is_empty()).unwrap_or("OFFLINE"));
            let qr = gw.filter(|g| !g.device_id.is_empty()).map(|g| render_device_identicon_svg(&g.device_id)).unwrap_or_default();
            let u_devs: Vec<_> = store.users.get(username).into_iter().flatten().collect();
            let u_folders = store.folders_for_user(username);
            let s_folders: Vec<_> = store.shared_folders.iter().filter(|(_, a)| a.contains(username)).map(|(f, _)| f.clone()).collect();
            let tot_f = u_folders.len() + s_folders.len();
            let mut rows = String::new();
            for dev in &u_devs {
                let (lbl, did) = (escape_html(&dev.label), escape_html(&dev.device_id));
                let sid = dev.device_id.split('-').next().unwrap_or("mesh").to_ascii_lowercase();
                let tkey = format!("spoke:{did}");
                let mut acc_cnt = 0usize;
                let mut kids = String::new();
                for f in &u_folders {
                    let (f_esc, flow, k) = (escape_html(f), f.to_ascii_lowercase(), format!("{username}:{f}"));
                    let fid = escape_html(&syncthing_folder_id(&format!("user:{username}:{flow}:{sid}")));
                    let pct = online_nodes.iter().filter_map(|n| snap.user_completion.get(&(k.clone(), n.name.clone())).copied()).min().unwrap_or(100);
                    let ok = snap.device_accepted.get(&(format!("user:{k}"), dev.device_id.clone())).copied().unwrap_or(false);
                    if ok { acc_cnt += 1; }
                    let st_b = if ok { "<span class=\"badge badge-online\">&#10003; accepted</span>" } else { "<span class=\"badge badge-warn\">&#9203; unaccepted</span> <a href=\"http://127.0.0.1:8384\" target=\"_blank\" rel=\"noopener\" style=\"color:var(--warn);font-size:0.74rem\">127.0.0.1:8384</a>" };
                    kids.push_str(&format!("<tr class=\"tree-child\" data-parent=\"{tkey}\" style=\"display:none\"><td style=\"padding-left:1.75rem\"><span class=\"tree-branch\">&#9492;&#9472;</span><a href=\"/?tab=1&amp;browse=personal:{f_esc}\" style=\"color:var(--accent);text-decoration:none;font-weight:600\">&#128193; Personal / {f_esc}/</a></td><td><code>{fid}</code> &bull; {pct}%</td><td>{st_b}</td><td style=\"text-align:right\"><span style=\"color:var(--muted);font-size:0.76rem\">&mdash;</span></td></tr>"));
                }
                for fname in &s_folders {
                    let (f_esc, flow) = (escape_html(fname), fname.to_ascii_lowercase());
                    let fid = escape_html(&syncthing_folder_id(&format!("shared:{flow}:{sid}")));
                    let pct = snap.shared_completion.get(fname).copied().unwrap_or(100);
                    let ok = snap.device_accepted.get(&(format!("shared:{fname}"), dev.device_id.clone())).copied().unwrap_or(false);
                    if ok { acc_cnt += 1; }
                    let st_b = if ok { "<span class=\"badge badge-online\">&#10003; accepted</span>" } else { "<span class=\"badge badge-warn\">&#9203; unaccepted</span> <a href=\"http://127.0.0.1:8384\" target=\"_blank\" rel=\"noopener\" style=\"color:var(--warn);font-size:0.74rem\">127.0.0.1:8384</a>" };
                    kids.push_str(&format!("<tr class=\"tree-child\" data-parent=\"{tkey}\" style=\"display:none\"><td style=\"padding-left:1.75rem\"><span class=\"tree-branch\">&#9492;&#9472;</span><a href=\"/?tab=2&amp;browse=shared:{f_esc}\" style=\"color:var(--purple);text-decoration:none;font-weight:600\">&#127760; Shared / {f_esc}/</a></td><td><code>{fid}</code> &bull; {pct}%</td><td>{st_b}</td><td style=\"text-align:right\"><span style=\"color:var(--muted);font-size:0.76rem\">&mdash;</span></td></tr>"));
                }
                if kids.is_empty() {
                    kids.push_str(&format!("<tr class=\"tree-child\" data-parent=\"{tkey}\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:1.75rem;color:var(--muted);font-size:0.8rem\"><span class=\"tree-branch\">&#9492;&#9472;</span>0 folders provisioned yet.</td></tr>"));
                }
                let sync_b = if tot_f == 0 || acc_cnt == tot_f { format!("<span class=\"badge badge-online\">{acc_cnt}/{tot_f} Accepted</span>") } else { format!("<span class=\"badge badge-warn\">{acc_cnt}/{tot_f} Accepted</span>") };
                let tbtn = format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{tkey}\" aria-expanded=\"false\" aria-label=\"Expand {lbl}\">+</button>");
                let rev_b = format!("<button type=\"button\" class=\"btn-danger\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\" data-del-action=\"/devices/remove\" data-del-name=\"{lbl}\" data-del-item=\"{did}\">Revoke</button>");
                rows.push_str(&format!("<tr><td>{tbtn}<strong>&#128187; {lbl}</strong></td><td><code style=\"font-family:var(--mono);font-size:0.8rem\">{did}</code></td><td>{sync_b}</td><td style=\"text-align:right\">{rev_b}</td></tr>{kids}"));
            }
            if rows.is_empty() {
                rows.push_str("<tr><td colspan=\"4\" style=\"color:var(--muted)\">0 enrolled devices. Click <strong>+ Add Device</strong> above to pair a workstation.</td></tr>");
                for f in &u_folders {
                    let (f_esc, flow) = (escape_html(f), f.to_ascii_lowercase());
                    let fid = escape_html(&syncthing_folder_id(&format!("user:{username}:{flow}:mesh")));
                    rows.push_str(&format!("<tr class=\"pending-folder\"><td><a href=\"/?tab=1&amp;browse=personal:{f_esc}\" style=\"color:var(--accent);text-decoration:none;font-weight:600\">&#128193; Personal / {f_esc}/</a></td><td><code>{fid}</code></td><td><span class=\"badge badge-warn\">NO DEVICES</span></td><td style=\"text-align:right\"><span style=\"color:var(--muted);font-size:0.76rem\">&mdash;</span></td></tr>"));
                }
                for fname in &s_folders {
                    let (f_esc, flow) = (escape_html(fname), fname.to_ascii_lowercase());
                    let fid = escape_html(&syncthing_folder_id(&format!("shared:{flow}:mesh")));
                    rows.push_str(&format!("<tr class=\"pending-folder\"><td><a href=\"/?tab=2&amp;browse=shared:{f_esc}\" style=\"color:var(--purple);text-decoration:none;font-weight:600\">&#127760; Shared / {f_esc}/</a></td><td><code>{fid}</code></td><td><span class=\"badge badge-warn\">NO DEVICES</span></td><td style=\"text-align:right\"><span style=\"color:var(--muted);font-size:0.76rem\">&mdash;</span></td></tr>"));
                }
            }
            format!("<table class=\"explorer-table\"><thead><tr><th>Device &nbsp;<button type=\"button\" class=\"btn-secondary\" style=\"padding:0.14rem 0.48rem;font-size:0.72rem;text-transform:none\" data-modal-open=\"enroll-spoke\">+ Add Device</button></th><th>Device ID</th><th style=\"width:210px\">Folders Synced</th><th style=\"width:140px;text-align:right\">Action</th></tr></thead><tbody>{rows}</tbody></table><template id=\"tmpl-enroll-spoke\"><div class=\"mono\" id=\"gw-dev\" style=\"user-select:all\">{dev_id}</div>{qr}</template>")
        }
        4 | 5 if is_admin => explorer_view::render_admin_pane(tab, store, snap),
        _ => explorer_view::render_file_explorer(username, store, tab, browse, sub),
    };
    let c = |n: u8, adm: bool| match (tab == n, adm) {
        (true, true) => "tab-btn admin-tab active",
        (false, true) => "tab-btn admin-tab",
        (true, false) => "tab-btn active",
        (false, false) => "tab-btn",
    };
    let (c1, c2, c3) = (c(1, false), c(2, false), c(3, false));
    let (c4, c5) = (c(4, true), c(5, true));
    let admin_links = if is_admin {
        format!("<span class=\"footer-tab-sep\" aria-hidden=\"true\">|||</span><a href=\"/?tab=5\" class=\"{c5}\">Mesh</a><a href=\"/?tab=4\" class=\"{c4}\">Global Folders</a>")
    } else {
        String::new()
    };
    let footer_tabs = format!("<nav class=\"nav-tabs footer-nav-tabs\"><a href=\"/?tab=1\" class=\"{c1}\">Personal Folders</a><a href=\"/?tab=2\" class=\"{c2}\">Shared Folders</a><span class=\"footer-tab-sep\" aria-hidden=\"true\">|||</span><a href=\"/?tab=3\" class=\"{c3}\">Device setup</a>{admin_links}</nav>");
    let footer = format!("<footer class=\"footer-dock\" id=\"footer-dock\" aria-label=\"Theme Palette\"><div class=\"footer-dock-inner\"><div class=\"footer-left-slot\">{footer_tabs}{footer_status}</div><div></div><div class=\"dock-controls\"><canvas id=\"footer-upload-graph\" class=\"footer-upload-graph\" width=\"128\" height=\"24\" title=\"Browser &lt;-&gt; Mesh Data Flow\" aria-label=\"Browser &lt;-&gt; Mesh Data Flow\"></canvas><div class=\"footer-accent-picker\" id=\"footer-accent-picker\" role=\"radiogroup\" aria-label=\"8-Bit Rainbow Palette\"><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Coral\" style=\"--swatch:#ff5555;background-color:#ff5555;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Coral\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Orange\" style=\"--swatch:#ff7700;background-color:#ff7700;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Orange\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Amber\" style=\"--swatch:#ffb000;background-color:#ffb000;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Amber\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Yellow\" style=\"--swatch:#f1fa8c;background-color:#f1fa8c;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Yellow\"></button><button type=\"button\" class=\"dock-accent-dot active\" data-accent-name=\"Emerald\" style=\"--swatch:#50fa7b;background-color:#50fa7b;\" role=\"radio\" aria-checked=\"true\" tabindex=\"0\" aria-label=\"Emerald\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Cyan\" style=\"--swatch:#00e5ff;background-color:#00e5ff;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Cyan\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Blue\" style=\"--swatch:#3b82f6;background-color:#3b82f6;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Blue\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Violet\" style=\"--swatch:#bd93f9;background-color:#bd93f9;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Violet\"></button></div></div></div></footer>");
    format!("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><link rel=\"icon\" href=\"/favicon.svg\" type=\"image/svg+xml\"><title>THRALL — Tool for Hierarchical Replication And Identity Linking ({user_esc})</title><style>{PORTAL_CSS}</style></head><body><header class=\"site-header\" id=\"site-header\"><div class=\"site-header-inner\"><div class=\"auth-brand-row\"><img src=\"/icon.jpg\" alt=\"THRALL\" width=\"34\" height=\"34\"><div><canvas id=\"header-wordmark\" class=\"header-wordmark\" width=\"140\" height=\"34\" aria-label=\"THRALL\"></canvas><div class=\"visually-hidden\">THRALL — Tool for Hierarchical Replication And Identity Linking</div><div style=\"font-family:var(--mono);font-size:0.65rem;color:var(--muted)\">Tool for Hierarchical Replication And Identity Linking</div></div></div><div></div><form method=\"POST\" action=\"/logout\" autocomplete=\"off\" class=\"header-login-form\"><button type=\"submit\">[ SIGN OUT ]</button></form></div></header><div class=\"middle-scroll\" id=\"middle-scroll\"><div class=\"container\">{pane}</div></div>{footer}<script src=\"/login.js\"></script></body></html>")
}
