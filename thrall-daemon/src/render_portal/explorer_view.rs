//! Web File Explorer, Share Mode UI, and Admin Tabs (Tab 4 & Tab 5) for THRALL.

use std::fs;
use std::path::Path;

use crate::device_store::{
    compute_share_token, resolve_jailed_path, syncthing_folder_id, MeshStore,
};
use crate::reconcile_mesh::FleetSnapshot;
use crate::render_portal::{escape_html, PORTAL_CSS};

pub fn render_public_share_dir(token: &str, sub: &str, dir_path: &Path) -> String {
    let (t_esc, s_esc, mut rows) = (escape_html(token), escape_html(sub), String::new());
    if let Ok(rd) = fs::read_dir(dir_path) {
        for e in rd.flatten() {
            let Ok(m) = e.file_type() else { continue };
            if m.is_symlink() {
                continue;
            }
            if let Some(n) = e.file_name().to_str().filter(|n| !n.starts_with('.')) {
                let is_dir = m.is_dir();
                let n_esc = escape_html(n);
                let next = if sub.is_empty() {
                    n_esc.clone()
                } else {
                    format!("{s_esc}/{n_esc}")
                };
                let icon = if is_dir { "&#128193;" } else { "&#128196;" };
                rows.push_str(&format!(
                    "<tr><td><a href=\"/s/{t_esc}?sub={next}\" style=\"color:var(--accent);text-decoration:none\">{icon} {n_esc}</a></td></tr>"
                ));
            }
        }
    }
    if rows.is_empty() {
        rows.push_str("<tr><td style=\"color:var(--muted)\">Empty shared folder.</td></tr>");
    }
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>THRALL Share</title><style>{PORTAL_CSS}</style></head><body><div class=\"container\"><div class=\"card\"><h2>Shared Folder (<code>/s/{t_esc}</code>)</h2><table><tbody>{rows}</tbody></table></div></div></body></html>"
    )
}

fn browse_scope(user: &str, store: &MeshStore, br: &str) -> String {
    if br.is_empty() || br == "personal" {
        let f = store
            .folders_for_user(user)
            .into_iter()
            .next()
            .unwrap_or_default();
        format!("personal:{user}:{f}")
    } else if let Some(pf) = br
        .strip_prefix("personal:")
        .or_else(|| br.strip_prefix("personal/"))
    {
        format!("personal:{user}:{pf}")
    } else {
        br.to_string()
    }
}

fn render_share_cell(user: &str, store: &MeshStore, br: &str, sub: &str, item: &str) -> String {
    let scope = browse_scope(user, store, br);
    let target = if item.is_empty() {
        scope
    } else {
        format!("{scope}/{item}")
    };
    let tok = compute_share_token(&target);
    let on = store.public_shares.contains_key(&tok);
    let (lbl, cls) = if on {
        ("Share: ON", "badge-online")
    } else {
        ("Share: OFF", "btn-secondary")
    };
    let link = if on {
        format!(" <a href=\"/s/{tok}\" target=\"_blank\" style=\"color:var(--accent);font-size:0.76rem;margin-left:0.35rem\">/s/{tok}</a>")
    } else {
        String::new()
    };
    let (br_e, sub_e, it_e) = (escape_html(br), escape_html(sub), escape_html(item));
    format!(
        "<form method=\"POST\" action=\"/shares/toggle\" style=\"display:inline\"><input type=\"hidden\" name=\"browse\" value=\"{br_e}\"><input type=\"hidden\" name=\"sub\" value=\"{sub_e}\"><input type=\"hidden\" name=\"item\" value=\"{it_e}\"><button type=\"submit\" class=\"{cls}\" style=\"padding:0.18rem 0.5rem;font-size:0.74rem\">{lbl}</button></form>{link}"
    )
}

#[rustfmt::skip]
pub fn render_device_badges(user: &str, scope_key: &str, store: &MeshStore, snap: &FleetSnapshot) -> (bool, String, String) {
    let devs: Vec<_> = store.users.get(user).into_iter().flatten().collect();
    if devs.is_empty() {
        return (false, "<span class=\"badge badge-warn\">NO CLIENT DEVICES</span>".into(), "<div class=\"st-reminder\">&#9888; Enroll a workstation in Tab 2 / Tab 3 first.</div>".into());
    }
    let (mut chips, mut missing) = (String::from("<div class=\"row\" style=\"gap:0.35rem;margin-top:0.35rem\">"), Vec::new());
    for d in &devs {
        let lbl = escape_html(&d.label);
        if snap.device_accepted.get(&(scope_key.to_string(), d.device_id.clone())).copied().unwrap_or(false) {
            chips.push_str(&format!("<span class=\"badge badge-online\">&#10003; {lbl}</span>"));
        } else {
            chips.push_str(&format!("<span class=\"badge badge-warn\">&#9203; {lbl}: unaccepted</span>"));
            missing.push(lbl);
        }
    }
    chips.push_str("</div>");
    if missing.is_empty() {
        (true, "<span class=\"badge badge-online\">ACCEPTED ON ALL DEVICES</span>".into(), chips)
    } else {
        let m_str = missing.join(", ");
        let badge = format!("<span class=\"badge badge-warn\">ACCEPTED {}/{} DEVICES</span>", devs.len() - missing.len(), devs.len());
        let rem = format!("{chips}<div class=\"st-reminder\">&#9888; Pending on <strong>{m_str}</strong> &mdash; open <a href=\"http://127.0.0.1:8384\" target=\"_blank\" rel=\"noopener\">http://127.0.0.1:8384</a> on {m_str} to accept this share.</div>");
        (false, badge, rem)
    }
}

#[rustfmt::skip]
pub fn render_file_explorer(user: &str, store: &MeshStore, browse: &str, sub: &str) -> String {
    let f = store
        .folders_for_user(user)
        .into_iter()
        .next()
        .unwrap_or_default();
    let def_br = format!("personal:{f}");
    let br = if browse.is_empty() || browse == "personal" {
        def_br.as_str()
    } else {
        browse
    };
    let Some((label, dir_path, rel)) = resolve_jailed_path(user, store, br, sub) else {
        return String::new();
    };
    let (br_esc, lbl_esc, rel_esc) = (escape_html(br), escape_html(&label), escape_html(&rel));
    let mut crumbs = format!(
        "<a href=\"/?tab=1&amp;browse={br_esc}\" class=\"btn-secondary\" style=\"padding:0.2rem 0.55rem;border-radius:4px;text-decoration:none;font-size:0.8rem\">&#128194; {lbl_esc}</a>"
    );
    let mut accum = String::new();
    for seg in rel.split('/').filter(|s| !s.is_empty()) {
        if !accum.is_empty() {
            accum.push('/');
        }
        accum.push_str(seg);
        let (a_esc, s_esc) = (escape_html(&accum), escape_html(seg));
        crumbs.push_str(&format!(
            " / <a href=\"/?tab=1&amp;browse={br_esc}&amp;sub={a_esc}\" class=\"btn-secondary\" style=\"padding:0.2rem 0.55rem;border-radius:4px;text-decoration:none;font-size:0.8rem\">{s_esc}</a>"
        ));
    }
    let dir_share = render_share_cell(user, store, br, &rel, &rel);
    let mut entries = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir_path) {
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_symlink() {
                continue;
            }
            if let Some(name) = e.file_name().to_str().filter(|n| !n.starts_with('.')) {
                let sz = e.metadata().map(|x| x.len()).unwrap_or(0);
                entries.push((!ft.is_dir(), name.to_string(), ft.is_dir(), sz));
            }
        }
    }
    entries.sort();
    let mut rows = String::new();
    for (_, name, is_dir, sz) in entries {
        let n_esc = escape_html(&name);
        let item_rel = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        let next = escape_html(&item_rel);
        let sh_btn = render_share_cell(user, store, br, &rel, &item_rel);
        if is_dir {
            rows.push_str(&format!(
                "<tr><td><a href=\"/?tab=1&amp;browse={br_esc}&amp;sub={next}\" style=\"color:var(--accent);text-decoration:none;font-weight:600\">&#128193; {n_esc}/</a></td><td>Directory</td><td style=\"text-align:right\">{sh_btn} <a href=\"/?tab=1&amp;browse={br_esc}&amp;sub={next}\" class=\"btn-secondary\" style=\"padding:0.2rem 0.55rem;border-radius:4px;text-decoration:none;font-size:0.78rem\">Open</a></td></tr>"
            ));
        } else {
            let sz_s = if sz >= 1024 {
                format!("{:.1} KiB", sz as f64 / 1024.0)
            } else {
                format!("{sz} B")
            };
            rows.push_str(&format!(
                "<tr><td><a href=\"/files?browse={br_esc}&amp;sub={next}\" style=\"color:var(--text);text-decoration:none\">&#128196; {n_esc}</a></td><td><code>{sz_s}</code></td><td style=\"text-align:right\">{sh_btn} <a href=\"/files?browse={br_esc}&amp;sub={next}\" class=\"btn-secondary\" style=\"padding:0.2rem 0.55rem;border-radius:4px;text-decoration:none;font-size:0.78rem\">Download</a></td></tr>"
            ));
        }
    }
    if rows.is_empty() {
        rows.push_str("<tr><td colspan=\"3\" style=\"color:var(--muted)\">Empty</td></tr>");
    }
    format!(
        "<div class=\"card\"><div class=\"node-header\"><h2>Explorer</h2><div class=\"row\">{crumbs} {dir_share}</div></div><table><thead><tr><th>Name</th><th>Size</th><th style=\"text-align:right\">Share / Action</th></tr></thead><tbody>{rows}</tbody></table><form method=\"POST\" action=\"/folders/mkdir\" style=\"margin-top:0.9rem;padding-top:0.8rem;border-top:1px solid var(--border)\"><input type=\"hidden\" name=\"browse\" value=\"{br_esc}\"><input type=\"hidden\" name=\"sub\" value=\"{rel_esc}\"><div class=\"row\"><div style=\"flex:1;min-width:200px\"><input type=\"text\" name=\"dirname\" required placeholder=\"Subfolder Name\"></div><button type=\"submit\" class=\"btn-secondary\">Create Subfolder</button></div></form></div>"
    )
}

#[rustfmt::skip]
pub fn render_admin_pane(tab: u8, store: &MeshStore, snap: &FleetSnapshot) -> String {
    if tab == 4 {
        let mut sh_rows = String::new();
        for (fname, allowed) in &store.shared_folders {
            let (f_esc, is_root) = (escape_html(fname), fname == "Shared");
            let fid = syncthing_folder_id(&format!("shared:{}:mesh", fname.to_ascii_lowercase()));
            let sub = if is_root { String::new() } else { format!("/{f_esc}") };
            let mut checks = String::new();
            for u in snap.ldap_users.iter().filter(|u| u.as_str() != "root") {
                let (u_esc, chk) = (escape_html(u), if allowed.contains(u) { " checked" } else { "" });
                let acc = snap.shared_user_accepted.get(&(fname.clone(), u.clone())).copied().unwrap_or(false);
                let mark = if acc { " &#10003;" } else { "" };
                checks.push_str(&format!("<label class=\"user-chip\"><input type=\"checkbox\" name=\"users\" value=\"{u_esc}\"{chk}>{u_esc}{mark}</label>"));
            }
            let del_btn = format!("<form method=\"POST\" action=\"/admin/shared/delete\" style=\"display:inline\"><input type=\"hidden\" name=\"folder\" value=\"{f_esc}\"><button type=\"submit\" class=\"btn-danger\">Delete</button></form>");
            let fid_esc = escape_html(&fid);
            sh_rows.push_str(&format!("<div class=\"node-card\"><div class=\"node-header\"><div><strong style=\"font-size:0.95rem\">{f_esc}</strong> &bull; <code style=\"font-size:0.78rem;color:var(--muted)\">{fid_esc}</code> &bull; <code style=\"font-size:0.78rem;color:var(--muted)\">/shared{sub}</code></div>{del_btn}</div><form method=\"POST\" action=\"/admin/shared/update\" style=\"margin-top:0.55rem\"><input type=\"hidden\" name=\"folder\" value=\"{f_esc}\"><div class=\"acl-user-grid\">{checks}</div><div style=\"margin-top:0.65rem;text-align:right\"><button type=\"submit\" class=\"btn-secondary\">Apply ACL</button></div></form></div>"));
        }
        return format!("<div class=\"card\"><h2>4. Access Control — Top-Level Shared Folders (Outside &lt;user&gt;)</h2><div class=\"grid\">{sh_rows}</div><form method=\"POST\" action=\"/admin/shared/create\" style=\"margin-top:1rem;padding-top:1rem;border-top:1px solid var(--border)\"><div class=\"row\"><div style=\"flex:1;min-width:240px\"><input type=\"text\" name=\"folder\" required placeholder=\"New Top-Level Shared Folder Name (e.g. Engineering)\"></div><button type=\"submit\">Create Shared Folder</button></div></form></div>");
    }
    let mut srv_cards = String::new();
    for node in &snap.nodes {
        let (badge, btn_lbl, btn_cls) = if node.disabled {
            ("<span class=\"badge badge-warn\">DISCONNECTED</span>", "Connect", "")
        } else if node.online {
            ("<span class=\"badge badge-online\">ONLINE</span>", "Disconnect", "btn-danger")
        } else {
            ("<span class=\"badge badge-offline\">OFFLINE</span>", "Disconnect", "btn-danger")
        };
        let (n_esc, wan_esc) = (escape_html(&node.name), escape_html(&node.wan_addr));
        let info_esc = escape_html(if node.device_id.is_empty() { &node.last_error } else { &node.device_id });
        srv_cards.push_str(&format!("<div class=\"node-card\"><div class=\"node-header\"><div><strong>{n_esc}</strong> &bull; <code style=\"font-size:0.78rem;color:var(--muted)\">{wan_esc}</code></div><div class=\"row\">{badge}<form method=\"POST\" action=\"/admin/servers/toggle\" style=\"display:inline\"><input type=\"hidden\" name=\"server\" value=\"{n_esc}\"><button type=\"submit\" class=\"{btn_cls}\" style=\"padding:0.25rem 0.65rem;font-size:0.78rem\">{btn_lbl}</button></form></div></div><div class=\"mono\" style=\"margin:0\">{info_esc}</div></div>"));
    }
    let mut all_rows = String::new();
    for (u, devs) in &store.users {
        let u_esc = escape_html(u);
        for d in devs {
            let (l_esc, id_esc) = (escape_html(&d.label), escape_html(&d.device_id));
            all_rows.push_str(&format!("<tr><td><code>{u_esc}</code></td><td>{l_esc}</td><td><code style=\"font-family:var(--mono);font-size:0.8rem\">{id_esc}</code></td><td><code>/users/{u_esc}/*</code></td></tr>"));
        }
    }
    if all_rows.is_empty() {
        all_rows.push_str("<tr><td colspan=\"4\" style=\"color:var(--muted)\">0 enrolled client devices.</td></tr>");
    }
    format!("<div class=\"card\"><div class=\"node-header\"><h2>5. Thrall Nodes</h2><form method=\"POST\" action=\"/reconcile\"><button type=\"submit\" class=\"btn-secondary\">Reconcile</button></form></div><div class=\"grid\" style=\"margin-bottom:1rem\">{srv_cards}</div><form method=\"POST\" action=\"/admin/servers/add\" style=\"margin:1rem 0;padding:0.9rem 0;border-top:1px solid var(--border);border-bottom:1px solid var(--border)\"><div class=\"row\"><div style=\"flex:1;min-width:180px\"><input type=\"text\" name=\"name\" required placeholder=\"Node Name (e.g. hyperion)\"></div><div style=\"flex:1;min-width:220px\"><input type=\"text\" name=\"host\" required placeholder=\"Tailscale IP / Host (e.g. 100.x.y.z)\"></div><button type=\"submit\">Add Thrall Node</button></div></form><table><thead><tr><th>Principal</th><th>Client Device Name</th><th>Client Device ID</th><th>Scope</th></tr></thead><tbody>{all_rows}</tbody></table></div>")
}
