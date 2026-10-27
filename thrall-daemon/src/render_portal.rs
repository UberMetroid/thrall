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

pub fn render_login_page(
    flash_err: Option<&str>,
    snap: &FleetSnapshot,
    store: &MeshStore,
) -> String {
    let alert = flash_err
        .filter(|m| !m.is_empty())
        .map(|m| {
            format!(
                "<span class=\"badge badge-offline\">{}</span>",
                escape_html(m)
            )
        })
        .unwrap_or_default();
    let on = snap.nodes.iter().filter(|n| n.online).count();
    let tot = snap.nodes.len().max(1);
    let conns: u64 = snap.nodes.iter().map(|n| n.active_conns).sum();
    let mini: usize = store.users.values().map(Vec::len).sum();
    let rx = fmt_rate(snap.nodes.iter().map(|n| n.in_bps).sum());
    let tx = fmt_rate(snap.nodes.iter().map(|n| n.out_bps).sum());
    let up_h = snap.nodes.iter().map(|n| n.uptime_sec).max().unwrap_or(0) / 3600;
    let nodes_str = format!("{on}/{tot}");
    let up_str = format!("{up_h}h");
    let mut node_tickers = String::new();
    for idx in 0..tot {
        let n = snap.nodes.get(idx);
        let n_num = idx + 1;
        let (st_lbl, st_cls) = if n.map(|x| x.online).unwrap_or(false) {
            ("ONLINE", "badge-online")
        } else {
            ("STANDBY", "badge-offline")
        };
        let n_conns = n.map(|x| x.active_conns).unwrap_or(0);
        let n_rx = fmt_rate(n.map(|x| x.in_bps).unwrap_or(0));
        let n_tx = fmt_rate(n.map(|x| x.out_bps).unwrap_or(0));
        let n_up = n.map(|x| x.uptime_sec / 3600).unwrap_or(0);
        let trk_cls = if idx % 2 == 1 {
            "tenets-track tenets-track-rev"
        } else {
            "tenets-track"
        };
        let chips_a = format!("<span class=\"tenet-chip\">&#9889; THRALL NODE {n_num:02}: {st_lbl}</span><span class=\"tenet-chip\">&#128279; ACTIVE BEP LINKS: {n_conns}</span><span class=\"tenet-chip\">&#128229; SPEED IN: {n_rx}</span><span class=\"tenet-chip\">&#128228; SPEED OUT: {n_tx}</span><span class=\"tenet-chip\">&#9201; NODE UPTIME: {n_up}h</span><span class=\"tenet-chip\">&#128737; TLS 1.3 BEP MESH</span><span class=\"tenet-chip\">&#128683; ZERO DOTFILE SYNC</span>");
        let chips_b = chips_a.replace(
            "class=\"tenet-chip\"",
            "class=\"tenet-chip\" aria-hidden=\"true\"",
        );
        node_tickers.push_str(&format!("<div class=\"node-ticker-card spotlight-card\"><div class=\"tui-header-bar\" style=\"margin-bottom:6px;padding-bottom:6px\"><span class=\"eco-badge\">[ THRALL MESH NODE {n_num:02} // REGIONAL BACKBONE ]</span><span class=\"badge {st_cls}\">{st_lbl} &bull; {n_conns} BEP LINKS</span></div><div class=\"tenets-marquee-wrap eco-marquee-viewport\" aria-label=\"Thrall Mesh Node {n_num:02} Telemetry\"><div class=\"{trk_cls}\">{chips_a}{chips_b}</div></div></div>"));
    }
    let hdr = format!("<header class=\"site-header\" id=\"site-header\"><div class=\"site-header-inner\"><div class=\"auth-brand-row\"><img src=\"/icon.jpg\" alt=\"THRALL\" width=\"34\" height=\"34\"><div><div style=\"font-weight:800;font-size:0.95rem;letter-spacing:0.06em;color:#fff\">THRALL</div><div style=\"font-family:var(--mono);font-size:0.68rem;color:var(--muted)\">Zero-Trust Hub &amp; Spoke Mesh</div></div></div><form method=\"POST\" action=\"/login\" autocomplete=\"off\" class=\"header-login-form\">{alert}<input type=\"text\" name=\"username\" required autocomplete=\"off\" placeholder=\"User or Email\" aria-label=\"User or Email\"><input type=\"password\" name=\"password\" required autocomplete=\"off\" placeholder=\"Password\" aria-label=\"Password\"><button type=\"submit\">[ SIGN IN ]</button></form></div></header>");
    let sec_thrall = format!("<section class=\"scroll-section sec-thrall\" id=\"sec-thrall\"><div class=\"spotlight-badge\"><span class=\"spotlight-badge-dot\" aria-hidden=\"true\"></span><span>{tot} THRALL MESH NODES &bull; {mini} ENROLLED CLIENT WORKSTATIONS</span></div><div class=\"wordmark-hero\" id=\"wordmark-anchor\"><h1 class=\"visually-hidden\">THRALL</h1></div><a href=\"#sec-ticker\" class=\"spotlight-scroll-cue\"><span>SCROLL TO EXPLORE // {tot} MESH NODE TICKERS</span><span class=\"scroll-cue-arrow\" aria-hidden=\"true\">&darr;</span></a></section>");
    let sec_ticker = format!("<section class=\"scroll-section sec-ticker\" id=\"sec-ticker\"><div class=\"ticker-stage-wrap\"><div class=\"tui-header-bar\"><span class=\"eco-badge\">[ 02 // PER-NODE LIVE MESH TICKERS ({tot} THRALL SERVERS) ]</span><span style=\"font-family:var(--mono);font-size:11px;color:var(--muted)\">PAUSE ON HOVER &bull; 1 TICKER PER REGIONAL THRALL NODE</span></div>{node_tickers}</div></section>");
    let tui = format!("<section class=\"scroll-section sec-telemetry\" id=\"sec-telemetry\"><div class=\"tui-director-card spotlight-card\"><div class=\"tui-header-bar\"><span class=\"eco-badge\">[ 03 // HIVE MESH TELEMETRY ]</span><span style=\"font-family:var(--mono);font-size:11px;color:var(--muted)\">BEP / TLS 1.3</span></div><pre class=\"tui-screen-mock\"><code><span class=\"tui-line-dim\">&#9484;&#9472;&#9472;</span> <span class=\"tui-line-accent\">THRALL MESH DIRECTOR</span> <span class=\"tui-line-dim\">&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9488;</span>\n<span class=\"tui-line-dim\">&#9474;</span>  <span class=\"tui-line-cyan\">Nodes Online:</span> <span class=\"tui-line-hot\">{nodes_str:<8}</span>   <span class=\"tui-line-cyan\">Active BEP Links:</span> <span class=\"tui-line-green\">{conns:<6}</span>    <span class=\"tui-line-cyan\">Uptime:</span> <span class=\"tui-line-hot\">{up_str:<11}</span>  <span class=\"tui-line-dim\">&#9474;</span>\n<span class=\"tui-line-dim\">&#9500;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9508;</span>\n<span class=\"tui-line-dim\">&#9474;</span>  <span class=\"tui-line-accent\">[1] Speed In</span>      <span class=\"tui-line-green\">&#9658; {rx:<16}</span> <span class=\"tui-line-accent\">[2] Speed Out</span>     <span class=\"tui-line-hot\">&#9658; {tx:<16}</span> <span class=\"tui-line-dim\">&#9474;</span>\n<span class=\"tui-line-dim\">&#9474;</span>  <span class=\"tui-line-accent\">[3] Folder Policy</span> <span class=\"tui-line-green\">&#9658; Top-Level Only (Zero Dotfiles)</span>       <span class=\"tui-line-cyan\">TLS 1.3 Mesh</span>     <span class=\"tui-line-dim\">&#9474;</span>\n<span class=\"tui-line-dim\">&#9492;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9472;&#9496;</span></code></pre></div></section>");
    let head_links = "<link rel=\"icon\" href=\"/favicon.svg\" type=\"image/svg+xml\">";
    let sec_chibi = "<section class=\"scroll-section sec-chibi\" id=\"sec-chibi\"><div class=\"chibi-section spotlight-card\"><div class=\"tui-header-bar\"><span class=\"eco-badge\">[ 04 // CHIBI THRALL MASCOT &bull; SETUP HELPER ]</span><button type=\"button\" class=\"btn-secondary\" id=\"chibi-next-btn\" style=\"padding:0.2rem 0.65rem;font-size:0.74rem\">[ NEXT TIP &rarr; ]</button></div><div class=\"chibi-card-body\"><img src=\"/mascot.jpg\" alt=\"Chibi Thrall Mascot\" class=\"chibi-hero-img\" id=\"chibi-hero-img\" style=\"cursor:pointer\" title=\"Click for another hint!\"><div class=\"chibi-card-copy\"><div id=\"chibi-tip-title\" style=\"font-weight:700;font-size:1.05rem;color:var(--phosphor);margin-bottom:0.35rem\">HINT #1 // TWO-WAY HANDSHAKE</div><p id=\"chibi-tip-body\" style=\"font-family:var(--mono);font-size:0.84rem;color:var(--text);line-height:1.6\">After signing in, go to Tab 2 (Initial Setup): add the Gateway Thrall Node ID in your local Syncthing (http://127.0.0.1:8384), then paste your 56-character Client Device ID into THRALL.</p></div></div></div></section>";
    let dock = "<nav class=\"footer-dock\" id=\"footer-dock\" aria-label=\"8-Bit Navigation and Theme Palette\"><div class=\"footer-dock-inner\"><div class=\"dock-chibi\" id=\"chibi-dock\" data-tab=\"0\"><button type=\"button\" class=\"chibi-mascot\" id=\"chibi-mascot-btn\" title=\"Click Chibi Thrall for another tip!\"><img src=\"/mascot.jpg\" alt=\"Chibi Thrall Mascot\"></button><div class=\"chibi-bubble\"><strong id=\"chibi-dock-title\">TIP 1/4:</strong> <span id=\"chibi-dock-body\">Sign in at the top bar, then visit Tab 2 (Initial Setup) to link your workstation.</span></div><button type=\"button\" id=\"chibi-dock-next\" class=\"dock-mini-btn\" title=\"Next Tip\">&rarr;</button></div><div class=\"dock-controls\"><button type=\"button\" class=\"dock-arrow-btn dock-arrow-up\" id=\"nav-arrow-up\" aria-label=\"Scroll to previous section\" title=\"Previous section (Up Arrow)\"><span class=\"pixel-bracket\">[</span><span class=\"pixel-glyph\">&#9650;</span><span class=\"pixel-text\">UP</span><span class=\"pixel-bracket\">]</span></button><div class=\"footer-accent-picker\" id=\"footer-accent-picker\" role=\"radiogroup\" aria-label=\"8-Bit Rainbow Palette\"><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Coral\" style=\"--swatch:#ff5555;background-color:#ff5555;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Coral\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Orange\" style=\"--swatch:#ff7700;background-color:#ff7700;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Orange\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Amber\" style=\"--swatch:#ffb000;background-color:#ffb000;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Amber\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Yellow\" style=\"--swatch:#f1fa8c;background-color:#f1fa8c;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Yellow\"></button><button type=\"button\" class=\"dock-accent-dot active\" data-accent-name=\"Emerald\" style=\"--swatch:#50fa7b;background-color:#50fa7b;\" role=\"radio\" aria-checked=\"true\" tabindex=\"0\" aria-label=\"Emerald\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Cyan\" style=\"--swatch:#00e5ff;background-color:#00e5ff;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Cyan\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Blue\" style=\"--swatch:#3b82f6;background-color:#3b82f6;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Blue\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Violet\" style=\"--swatch:#bd93f9;background-color:#bd93f9;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Violet\"></button></div><a href=\"https://github.com/UberMetroid/thrall\" target=\"_blank\" rel=\"noopener\" class=\"dock-arrow-btn\"><span class=\"pixel-bracket\">[</span><span class=\"pixel-text\">GITHUB</span><span>&nearr;</span><span class=\"pixel-bracket\">]</span></a><button type=\"button\" class=\"dock-arrow-btn dock-arrow-down\" id=\"nav-arrow-down\" aria-label=\"Scroll to next section\" title=\"Next section (Down Arrow)\"><span class=\"pixel-bracket\">[</span><span class=\"pixel-text\">DOWN</span><span class=\"pixel-glyph\">&#9660;</span><span class=\"pixel-bracket\">]</span></button></div></div></nav>";
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">{head_links}<title>THRALL</title><style>{PORTAL_CSS}</style></head><body><canvas id=\"spotlight-canvas\" class=\"spotlight-canvas\" data-mesh-nodes=\"{tot}\" data-mini-nodes=\"{mini}\" aria-hidden=\"true\"></canvas>{hdr}<div class=\"middle-scroll\" id=\"middle-scroll\"><main class=\"login-stage\" id=\"intro\">{sec_thrall}{sec_ticker}{tui}{sec_chibi}</main></div>{dock}<script src=\"/login.js\"></script></body></html>"
    )
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
    let tab = if (1..=3).contains(&active_tab) || (is_admin && (4..=5).contains(&active_tab)) {
        active_tab
    } else {
        1
    };
    let user_esc = escape_html(username);
    let role_badge = if is_admin {
        "<span class=\"badge badge-admin\">ADMIN</span>"
    } else {
        "<span class=\"badge badge-online\">USER</span>"
    };
    let mut alerts = String::new();
    if let Some(ok) = flash_ok.filter(|s| !s.is_empty()) {
        alerts.push_str(&format!(
            "<div class=\"alert alert-ok\">{}</div>",
            escape_html(ok)
        ));
    }
    if let Some(err) = flash_err.filter(|s| !s.is_empty()) {
        alerts.push_str(&format!(
            "<div class=\"alert alert-err\">{}</div>",
            escape_html(err)
        ));
    }
    let online_nodes: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.online && !n.device_id.is_empty())
        .collect();

    let pane = match tab {
        2 => {
            let gw = online_nodes.first().copied().or_else(|| snap.nodes.first());
            let dev_id = escape_html(
                gw.map(|g| g.device_id.as_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("OFFLINE"),
            );
            let qr = gw
                .filter(|g| !g.device_id.is_empty())
                .map(|g| render_device_identicon_svg(&g.device_id))
                .unwrap_or_default();
            format!(
                "<div class=\"card\"><h2>2. Initial Setup (Two-Way Zero-Trust Handshake)</h2><div class=\"grid\"><div class=\"node-card\"><div class=\"node-header\"><strong>Step 1 — Client Device Trusts Thrall Mesh</strong><span class=\"badge badge-online\">ON YOUR LAPTOP / DESKTOP</span></div><ol style=\"font-size:0.84rem;color:var(--text);padding-left:1.2rem;margin:0.5rem 0\"><li>Open your local Syncthing GUI (<code>http://127.0.0.1:8384</code>).</li><li>Click <strong>Actions &rarr; Show ID</strong> and copy your 56-character Client Device ID for Step 2.</li><li>Click <strong>+ Add Remote Device</strong> and paste this <strong>Gateway Thrall Node ID</strong>:</li></ol><div class=\"mono\" id=\"gw-dev\" style=\"user-select:all\">{dev_id}</div><div class=\"row\" style=\"justify-content:space-between;margin-top:0.5rem\"><div style=\"font-size:0.82rem;color:var(--text)\">4. Under the <strong>Sharing</strong> tab, check:<br><code>[x] Introducer</code> &nbsp; <code>[x] Auto Accept</code> &rarr; <strong>Save</strong></div>{qr}</div></div><div class=\"node-card\"><div class=\"node-header\"><strong>Step 2 — Thrall Mesh Trusts Client Device</strong><span class=\"badge badge-admin\">IN THIS PORTAL</span></div><ol style=\"font-size:0.84rem;color:var(--text);padding-left:1.2rem;margin:0.5rem 0 0.85rem 0\"><li>Paste your Client Device's 56-character ID (from Step 1.2) below.</li><li>THRALL authorizes your Client Device across all Thrall Nodes (never connecting Client Devices to other Client Devices) and syncs your chosen top-level folders.</li></ol><form method=\"POST\" action=\"/devices/add\"><div style=\"margin-bottom:0.75rem\"><input type=\"text\" name=\"device_id\" required placeholder=\"Paste 56-Char Syncthing Client Device ID\"></div><button type=\"submit\" style=\"width:100%\">Authorize Client Device</button></form></div></div></div>"
            )
        }
        3 => {
            let mut devs_rows = String::new();
            for dev in store.users.get(username).into_iter().flatten() {
                let (lbl, did) = (escape_html(&dev.label), escape_html(&dev.device_id));
                devs_rows.push_str(&format!("<tr><td><strong>{lbl}</strong></td><td><code style=\"font-family:var(--mono);font-size:0.82rem\">{did}</code></td><td style=\"text-align:right\"><form method=\"POST\" action=\"/devices/remove\" style=\"display:inline\"><input type=\"hidden\" name=\"device_id\" value=\"{did}\"><button type=\"submit\" class=\"btn-danger\">Revoke</button></form></td></tr>"));
            }
            if devs_rows.is_empty() {
                devs_rows.push_str("<tr><td colspan=\"3\" style=\"color:var(--muted)\">0 enrolled client devices.</td></tr>");
            }
            format!("<div class=\"card\"><h2>3. Client Devices</h2><table><thead><tr><th>Client Device Name</th><th>Client Device ID</th><th style=\"text-align:right\">Action</th></tr></thead><tbody>{devs_rows}</tbody></table><form method=\"POST\" action=\"/devices/add\" style=\"margin-top:1.1rem;padding-top:1rem;border-top:1px solid var(--border)\"><div class=\"row\"><div style=\"flex:2;min-width:320px\"><input type=\"text\" name=\"device_id\" required placeholder=\"Paste 56-Char Syncthing Client Device ID\"></div><button type=\"submit\">Authorize Client Device</button></div></form></div>")
        }
        4 | 5 if is_admin => explorer_view::render_admin_pane(tab, store, snap),
        _ => {
            let (mut p_cards, mut sh_cards) = (String::new(), String::new());
            let sid = store.users.get(username).and_then(|d| d.first()).and_then(|d| d.device_id.split('-').next()).unwrap_or("mesh").to_ascii_lowercase();
            for f in store.folders_for_user(username) {
                let (f_esc, flow, k) = (escape_html(&f), f.to_ascii_lowercase(), format!("{username}:{f}"));
                let fid = escape_html(&syncthing_folder_id(&format!("user:{username}:{flow}:{sid}")));
                let pct = online_nodes.iter().filter_map(|n| snap.user_completion.get(&(k.clone(), n.name.clone())).copied()).min().unwrap_or(100);
                let (acc, badge, rem) = explorer_view::render_device_badges(username, &format!("user:{k}"), store, snap);
                let cls = if acc { "node-card" } else { "node-card pending-folder" };
                p_cards.push_str(&format!("<div class=\"{cls}\"><div class=\"node-header\"><a href=\"/?tab=1&amp;browse=personal:{f_esc}\" style=\"color:var(--accent);text-decoration:none;font-weight:700\">&#128193; {f_esc}/</a>{badge}</div>{rem}<div class=\"row\" style=\"justify-content:space-between;margin-top:0.45rem;font-size:0.8rem;font-family:var(--mono)\"><span><code>{fid}</code> &bull; <div class=\"progress-bar\"><div class=\"progress-fill\" style=\"width:{pct}%\"></div></div>{pct}%</span><div class=\"row\" style=\"gap:0.35rem\"><a href=\"/?tab=1&amp;browse=personal:{f_esc}\" class=\"btn-secondary\" style=\"padding:0.18rem 0.5rem;border-radius:4px;text-decoration:none;font-size:0.75rem\">Browse</a><form method=\"POST\" action=\"/user/folders/delete\" style=\"display:inline\"><input type=\"hidden\" name=\"folder\" value=\"{f_esc}\"><button type=\"submit\" class=\"btn-danger\" style=\"padding:0.18rem 0.5rem;font-size:0.75rem\">Delete</button></form></div></div></div>"));
            }
            if p_cards.is_empty() {
                p_cards.push_str("<div style=\"color:var(--muted);font-family:var(--mono);font-size:0.84rem\">0 personal folders created yet. Create one below.</div>");
            }
            for (fname, _) in store.shared_folders.iter().filter(|(_, a)| a.contains(username)) {
                let (f_esc, flow) = (escape_html(fname), fname.to_ascii_lowercase());
                let fid = escape_html(&syncthing_folder_id(&format!("shared:{flow}:{sid}")));
                let pct = snap.shared_completion.get(fname).copied().unwrap_or(100);
                let (acc, s_badge, s_rem) = explorer_view::render_device_badges(username, &format!("shared:{fname}"), store, snap);
                let s_cls = if acc { "node-card" } else { "node-card pending-folder" };
                sh_cards.push_str(&format!("<div class=\"{s_cls}\"><div class=\"node-header\"><a href=\"/?tab=1&amp;browse=shared:{f_esc}\" style=\"color:var(--purple);text-decoration:none;font-weight:700\">&#127760; Shared: {f_esc}/</a>{s_badge}</div>{s_rem}<div class=\"row\" style=\"justify-content:space-between;margin-top:0.45rem;font-size:0.8rem;font-family:var(--mono)\"><span><code>{fid}</code> &bull; <div class=\"progress-bar\"><div class=\"progress-fill\" style=\"width:{pct}%\"></div></div>{pct}%</span><a href=\"/?tab=1&amp;browse=shared:{f_esc}\" class=\"btn-secondary\" style=\"padding:0.18rem 0.5rem;border-radius:4px;text-decoration:none;font-size:0.75rem\">Browse</a></div></div>"));
            }
            if sh_cards.is_empty() {
                sh_cards.push_str("<div style=\"color:var(--muted);font-family:var(--mono);font-size:0.84rem\">0 shared folders assigned to your account.</div>");
            }
            let explorer = explorer_view::render_file_explorer(username, store, browse, sub);
            format!("<div class=\"card\"><div class=\"node-header\"><h2>1A. Personal Folders ({user_esc})</h2><span class=\"badge badge-online\">PRIVATE VAULTS &bull; /users/{user_esc}/&lt;Folder&gt;</span></div><div class=\"grid\">{p_cards}</div><form method=\"POST\" action=\"/user/folders/create\" style=\"margin-top:1rem;padding-top:0.85rem;border-top:1px solid var(--border)\"><div class=\"row\"><div style=\"flex:1;min-width:220px\"><input type=\"text\" name=\"folder\" required placeholder=\"New Personal Folder Name (e.g. Projects)\"></div><button type=\"submit\">Create Personal Folder</button></div></form><div class=\"st-reminder\" style=\"margin-top:0.65rem\">&#128161; After creating a folder, open your local Syncthing GUI at <a href=\"http://127.0.0.1:8384\" target=\"_blank\" rel=\"noopener\">http://127.0.0.1:8384</a> (<code>localhost:8384</code>) on your Client Device to accept &amp; add the new folder share.</div></div><div class=\"card\" style=\"border-color:rgba(189,147,249,0.4)\"><div class=\"node-header\"><h2>1B. Shared Folders (Multi-User Mesh)</h2><span class=\"badge badge-admin\">TEAM VAULTS &bull; /shared/&lt;Folder&gt;</span></div><div class=\"grid\">{sh_cards}</div></div>{explorer}")
        }
    };
    let c = |n: u8, adm: bool| match (tab == n, adm) {
        (true, true) => "tab-btn admin-tab active",
        (false, true) => "tab-btn admin-tab",
        (true, false) => "tab-btn active",
        (false, false) => "tab-btn",
    };
    let (c1, c2, c3) = (c(1, false), c(2, false), c(3, false));
    let (c4, c5) = (c(4, true), c(5, true));
    let admin_nav = if is_admin {
        format!("<a href=\"/?tab=4\" class=\"{c4}\">4. Access Control</a><a href=\"/?tab=5\" class=\"{c5}\">5. Thrall Nodes</a>")
    } else {
        String::new()
    };
    let footer = format!("<footer class=\"footer-dock\" id=\"footer-dock\" aria-label=\"Chibi Thrall Setup Helper and Theme Palette\"><div class=\"footer-dock-inner\"><div class=\"dock-chibi\" id=\"chibi-dock\" data-tab=\"{tab}\"><button type=\"button\" class=\"chibi-mascot\" id=\"chibi-mascot-btn\" title=\"Click Chibi Thrall for another tip!\"><img src=\"/mascot.jpg\" alt=\"Chibi Thrall Mascot\"></button><div class=\"chibi-bubble\"><strong id=\"chibi-dock-title\">TIP #1:</strong> <span id=\"chibi-dock-body\">Created a new folder? Open <a href=\"http://127.0.0.1:8384\" target=\"_blank\" rel=\"noopener\">localhost:8384</a> on your workstation and click Accept!</span></div><button type=\"button\" id=\"chibi-dock-next\" class=\"dock-mini-btn\" title=\"Next Tip\">&rarr;</button></div><div class=\"dock-controls\"><div class=\"footer-accent-picker\" id=\"footer-accent-picker\" role=\"radiogroup\" aria-label=\"8-Bit Rainbow Palette\"><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Coral\" style=\"--swatch:#ff5555;background-color:#ff5555;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Coral\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Orange\" style=\"--swatch:#ff7700;background-color:#ff7700;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Orange\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Amber\" style=\"--swatch:#ffb000;background-color:#ffb000;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Amber\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Yellow\" style=\"--swatch:#f1fa8c;background-color:#f1fa8c;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Yellow\"></button><button type=\"button\" class=\"dock-accent-dot active\" data-accent-name=\"Emerald\" style=\"--swatch:#50fa7b;background-color:#50fa7b;\" role=\"radio\" aria-checked=\"true\" tabindex=\"0\" aria-label=\"Emerald\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Cyan\" style=\"--swatch:#00e5ff;background-color:#00e5ff;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Cyan\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Blue\" style=\"--swatch:#3b82f6;background-color:#3b82f6;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Blue\"></button><button type=\"button\" class=\"dock-accent-dot\" data-accent-name=\"Violet\" style=\"--swatch:#bd93f9;background-color:#bd93f9;\" role=\"radio\" aria-checked=\"false\" tabindex=\"-1\" aria-label=\"Violet\"></button></div><a href=\"https://github.com/UberMetroid/thrall\" target=\"_blank\" rel=\"noopener\" class=\"dock-arrow-btn\"><span class=\"pixel-bracket\">[</span><span class=\"pixel-text\">GITHUB</span><span>&nearr;</span><span class=\"pixel-bracket\">]</span></a></div></div></footer>");
    format!("<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><link rel=\"icon\" href=\"/favicon.svg\" type=\"image/svg+xml\"><title>THRALL — Tool for Hierarchical Replication And Identity Linking ({user_esc})</title><style>{PORTAL_CSS}</style></head><body><header class=\"site-header\" id=\"site-header\"><div class=\"site-header-inner\"><div class=\"auth-brand-row\"><img src=\"/icon.jpg\" alt=\"THRALL\" width=\"34\" height=\"34\"><div><div style=\"font-weight:800;font-size:0.95rem;letter-spacing:0.06em;color:#fff\">THRALL</div><div class=\"visually-hidden\">THRALL — Tool for Hierarchical Replication And Identity Linking</div><div style=\"font-family:var(--mono);font-size:0.68rem;color:var(--muted)\">Zero-Trust Hub &amp; Spoke Mesh</div></div>{role_badge}</div><nav class=\"nav-tabs\"><a href=\"/?tab=1\" class=\"{c1}\">1. Files</a><a href=\"/?tab=2\" class=\"{c2}\">2. Initial Setup</a><a href=\"/?tab=3\" class=\"{c3}\">3. Client Devices</a>{admin_nav}</nav><div class=\"row\"><code style=\"font-size:0.8rem;color:var(--muted)\">{user_esc}</code><form method=\"POST\" action=\"/logout\" style=\"display:inline\"><button type=\"submit\" class=\"btn-secondary\" style=\"padding:0.32rem 0.65rem;font-size:0.76rem\">Sign Out</button></form></div></div></header><div class=\"middle-scroll\" id=\"middle-scroll\"><div class=\"container\">{alerts}{pane}</div></div>{footer}<script src=\"/login.js\"></script></body></html>")
}
