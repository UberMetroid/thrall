//! Web File Explorer, Share Mode UI, and Admin Tabs (Tab 3 & Tab 4) for THRALL.

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

#[rustfmt::skip]
fn render_share_cell(user: &str, store: &MeshStore, s_br: &str, f_br: &str, sub: &str, item: &str) -> String {
    let (sc_br, i_rel) = if s_br.is_empty() { item.split_once('/').unwrap_or((item, "")) } else { (s_br, item.strip_prefix(&format!("{s_br}/")).unwrap_or(item)) };
    let scope = browse_scope(user, store, sc_br);
    let target = if i_rel.is_empty() { scope } else { format!("{scope}/{i_rel}") };
    let tok = compute_share_token(&target);
    let on = store.public_shares.contains_key(&tok);
    let (lbl, cls) = if on { ("Share: ON", "badge-online") } else { ("Share: OFF", "btn-secondary") };
    let link = if on { format!(" <a href=\"/s/{tok}\" target=\"_blank\" style=\"color:var(--accent);font-size:0.76rem;margin-left:0.35rem\">/s/{tok}</a>") } else { String::new() };
    let (br_e, sub_e, it_e) = (escape_html(f_br), escape_html(sub), escape_html(item));
    format!("<form method=\"POST\" action=\"/shares/toggle\" style=\"display:inline\"><input type=\"hidden\" name=\"browse\" value=\"{br_e}\"><input type=\"hidden\" name=\"sub\" value=\"{sub_e}\"><input type=\"hidden\" name=\"item\" value=\"{it_e}\"><button type=\"submit\" class=\"{cls}\" style=\"padding:0.18rem 0.5rem;font-size:0.74rem\">{lbl}</button></form>{link}")
}

#[rustfmt::skip]
pub fn render_device_badges(user: &str, scope_key: &str, store: &MeshStore, snap: &FleetSnapshot) -> (bool, String, String) {
    let devs: Vec<_> = store.users.get(user).into_iter().flatten().collect();
    if devs.is_empty() {
        return (false, "<span class=\"badge badge-warn\">NO SPOKES</span>".into(), "<div class=\"st-reminder\">&#9888; Enroll a Spoke in Step 1 &amp; Step 2 below first.</div>".into());
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
        (true, "<span class=\"badge badge-online\">ACCEPTED ON ALL SPOKES</span>".into(), chips)
    } else {
        let m_str = missing.join(", ");
        let badge = format!("<span class=\"badge badge-warn\">ACCEPTED {}/{} SPOKES</span>", devs.len() - missing.len(), devs.len());
        let rem = format!("{chips}<div class=\"st-reminder\">&#9888; Pending on <strong>{m_str}</strong> &mdash; open <a href=\"http://127.0.0.1:8384\" target=\"_blank\" rel=\"noopener\">http://127.0.0.1:8384</a> on {m_str} to accept this share.</div>");
        (false, badge, rem)
    }
}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
fn render_dir_rows(user: &str, store: &MeshStore, br_scope: &str, form_br: &str, dir_path: &Path, rel: &str, pkey: &str, depth: usize, out: &mut String) {
    let (br_e, fbr_e, rel_e, pk_e) = (escape_html(br_scope), escape_html(form_br), escape_html(rel), escape_html(pkey));
    let (st, pad) = if pkey.is_empty() { ("", 0.5) } else { (" style=\"display:none\"", 0.5 + depth as f64 * 1.25) };
    let p_attr = if pkey.is_empty() { String::new() } else { format!(" class=\"tree-child\" data-parent=\"{pk_e}\"{st}") };
    let pfx = if pkey.is_empty() { "" } else { "<span class=\"tree-branch\">&#9492;&#9472;</span>" };
    let mut entries = Vec::new();
    if let Ok(rd) = fs::read_dir(dir_path) {
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            if !ft.is_symlink() {
                if let Some(n) = e.file_name().to_str().filter(|n| !n.starts_with('.')) {
                    entries.push((!ft.is_dir(), n.to_string(), ft.is_dir(), e.metadata().map(|x| x.len()).unwrap_or(0), e.path()));
                }
            }
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    if entries.is_empty() {
        let msg = if pkey.is_empty() { "Empty" } else { "Empty directory" };
        out.push_str(&format!("<tr{p_attr}><td colspan=\"3\" style=\"padding-left:{pad:.2}rem;color:var(--muted);font-size:0.8rem\">{pfx}{msg}</td></tr>"));
        return;
    }
    for (_, name, is_dir, sz, cpath) in entries {
        let n_esc = escape_html(&name);
        let item_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
        let next = escape_html(&item_rel);
        let form_it = if form_br.is_empty() { format!("{br_scope}/{item_rel}") } else { item_rel.clone() };
        let fit_e = escape_html(&form_it);
        let sh_btn = render_share_cell(user, store, br_scope, form_br, &rel_e, &form_it);
        let del_f = format!("<form method=\"POST\" action=\"/files/delete\" style=\"display:inline\"><input type=\"hidden\" name=\"browse\" value=\"{fbr_e}\"><input type=\"hidden\" name=\"sub\" value=\"{rel_e}\"><input type=\"hidden\" name=\"item\" value=\"{fit_e}\"><button type=\"submit\" class=\"btn-danger\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\">Delete</button></form>");
        if is_dir {
            let ckey = format!("{br_e}:{next}");
            let tbtn = if depth < 3 { format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{ckey}\" aria-expanded=\"false\" aria-label=\"Expand {n_esc}\">+</button>") } else { String::new() };
            out.push_str(&format!("<tr{p_attr}><td style=\"padding-left:{pad:.2}rem\">{pfx}{tbtn}<a href=\"/?tab=1&amp;browse={br_e}&amp;sub={next}\" style=\"color:var(--accent);text-decoration:none;font-weight:600\">&#128193; {n_esc}/</a></td><td>{sh_btn}</td><td style=\"text-align:right\">{del_f}</td></tr>"));
            if depth < 3 { render_dir_rows(user, store, br_scope, form_br, &cpath, &item_rel, &ckey, depth + 1, out); }
        } else {
            let sz_s = if sz >= 1024 { format!("{:.1} KiB", sz as f64 / 1024.0) } else { format!("{sz} B") };
            out.push_str(&format!("<tr{p_attr}><td style=\"padding-left:{pad:.2}rem\">{pfx}<a href=\"/files?browse={br_e}&amp;sub={next}\" style=\"color:var(--text);text-decoration:none\">&#128196; {n_esc}</a> <code style=\"color:var(--muted);font-size:0.74rem;margin-left:0.35rem\">{sz_s}</code></td><td>{sh_btn}</td><td style=\"text-align:right\"><a href=\"/files?browse={br_e}&amp;sub={next}\" class=\"btn-secondary\" style=\"padding:0.2rem 0.55rem;border-radius:4px;text-decoration:none;font-size:0.78rem\">Download</a> {del_f}</td></tr>"));
        }
    }
}

#[rustfmt::skip]
pub fn render_file_explorer(user: &str, store: &MeshStore, browse: &str, sub: &str) -> String {
    if browse.is_empty() || browse == "personal" {
        let (u_esc, mut u_rows, mut s_rows) = (escape_html(user), String::new(), String::new());
        for f in store.folders_for_user(user) {
            let (f_esc, br_p) = (escape_html(&f), format!("personal:{f}"));
            let k_esc = escape_html(&br_p);
            let sh_btn = render_share_cell(user, store, "", "", "", &br_p);
            let del_f = format!("<form method=\"POST\" action=\"/user/folders/delete\" style=\"display:inline\"><input type=\"hidden\" name=\"folder\" value=\"{f_esc}\"><button type=\"submit\" class=\"btn-danger\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\">Delete</button></form>");
            let tbtn = format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{k_esc}\" aria-expanded=\"false\" aria-label=\"Expand {f_esc}\">+</button>");
            u_rows.push_str(&format!("<tr><td>{tbtn}<a href=\"/?tab=1&amp;browse=personal:{f_esc}\" style=\"color:var(--accent);text-decoration:none;font-weight:700\">&#128193; {f_esc}/</a></td><td>{sh_btn}</td><td style=\"text-align:right\">{del_f}</td></tr>"));
            let p = Path::new("/users").join(user).join(&f);
            render_dir_rows(user, store, &br_p, "", &p, "", &br_p, 1, &mut u_rows);
        }
        if u_rows.is_empty() {
            u_rows.push_str("<tr><td colspan=\"3\" style=\"color:var(--muted)\">0 folders accessible yet. Create a personal folder below.</td></tr>");
        }
        for (fname, _) in store.shared_folders.iter().filter(|(_, a)| a.contains(user)) {
            let (f_esc, br_s) = (escape_html(fname), format!("shared:{fname}"));
            let k_esc = escape_html(&br_s);
            let sh_btn = render_share_cell(user, store, "", "", "", &br_s);
            let tbtn = format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{k_esc}\" aria-expanded=\"false\" aria-label=\"Expand {f_esc}\">+</button>");
            s_rows.push_str(&format!("<tr><td>{tbtn}<a href=\"/?tab=1&amp;browse=shared:{f_esc}\" style=\"color:var(--purple);text-decoration:none;font-weight:700\">&#127760; {f_esc}/</a></td><td>{sh_btn}</td><td style=\"text-align:right\"><span style=\"color:var(--muted);font-size:0.76rem\">&mdash;</span></td></tr>"));
            let p = if fname == "Shared" { Path::new("/shared").to_path_buf() } else { Path::new("/shared").join(fname) };
            render_dir_rows(user, store, &br_s, "", &p, "", &br_s, 1, &mut s_rows);
        }
        if s_rows.is_empty() {
            s_rows.push_str("<tr><td colspan=\"3\" style=\"color:var(--muted)\">0 shared folders assigned to your account.</td></tr>");
        }
        return format!("<div class=\"card\"><div class=\"node-header\"><h2>1A. User Vault ({u_esc})</h2><span class=\"badge badge-online\">PERSONAL VAULT</span></div><table><thead><tr><th>Folder</th><th>Share</th><th style=\"text-align:right\">Action</th></tr></thead><tbody>{u_rows}</tbody></table><form method=\"POST\" action=\"/user/folders/create\" style=\"margin-top:0.95rem;padding-top:0.85rem;border-top:1px solid var(--border)\"><div class=\"row\"><div style=\"flex:1;min-width:220px\"><input type=\"text\" name=\"folder\" required placeholder=\"New Personal Folder Name (e.g. Projects)\"></div><button type=\"submit\">+ Create Personal Folder</button></div></form></div><div class=\"card\"><div class=\"node-header\"><h2>1B. Shared Vault</h2><span class=\"badge badge-admin\">SHARED VAULT</span></div><table><thead><tr><th>Folder</th><th>Share</th><th style=\"text-align:right\">Action</th></tr></thead><tbody>{s_rows}</tbody></table></div>");
    }
    let Some((label, dir_path, rel)) = resolve_jailed_path(user, store, browse, sub) else { return String::new(); };
    let (br_esc, lbl_esc, rel_esc) = (escape_html(browse), escape_html(&label), escape_html(&rel));
    let mut crumbs = format!("<a href=\"/?tab=1\" class=\"btn-secondary\" style=\"padding:0.2rem 0.55rem;border-radius:4px;text-decoration:none;font-size:0.8rem\">All Folders</a> / <a href=\"/?tab=1&amp;browse={br_esc}\" class=\"btn-secondary\" style=\"padding:0.2rem 0.55rem;border-radius:4px;text-decoration:none;font-size:0.8rem\">&#128194; {lbl_esc}</a>");
    let mut accum = String::new();
    for seg in rel.split('/').filter(|s| !s.is_empty()) {
        if !accum.is_empty() { accum.push('/'); }
        accum.push_str(seg);
        let (a_esc, s_esc) = (escape_html(&accum), escape_html(seg));
        crumbs.push_str(&format!(" / <a href=\"/?tab=1&amp;browse={br_esc}&amp;sub={a_esc}\" class=\"btn-secondary\" style=\"padding:0.2rem 0.55rem;border-radius:4px;text-decoration:none;font-size:0.8rem\">{s_esc}</a>"));
    }
    let dir_share = render_share_cell(user, store, browse, browse, &rel, &rel);
    let mut rows = String::new();
    render_dir_rows(user, store, browse, browse, &dir_path, &rel, "", 0, &mut rows);
    format!("<div class=\"card\"><div class=\"node-header\"><h2>1. Explorer</h2><div class=\"row\">{crumbs} {dir_share}</div></div><table><thead><tr><th>Name</th><th>Share</th><th style=\"text-align:right\">Action</th></tr></thead><tbody>{rows}</tbody></table><div class=\"row\" style=\"margin-top:0.9rem;padding-top:0.8rem;border-top:1px solid var(--border);gap:0.75rem\"><form method=\"POST\" action=\"/folders/mkdir\" class=\"row\" style=\"flex:1;min-width:260px\"><input type=\"hidden\" name=\"browse\" value=\"{br_esc}\"><input type=\"hidden\" name=\"sub\" value=\"{rel_esc}\"><div style=\"flex:1\"><input type=\"text\" name=\"dirname\" required placeholder=\"Subfolder Name\"></div><button type=\"submit\" class=\"btn-secondary\">+ Subfolder</button></form><form method=\"POST\" action=\"/files/create\" class=\"row\" style=\"flex:2;min-width:340px\"><input type=\"hidden\" name=\"browse\" value=\"{br_esc}\"><input type=\"hidden\" name=\"sub\" value=\"{rel_esc}\"><div style=\"width:170px\"><input type=\"text\" name=\"filename\" required placeholder=\"File (e.g. note.md)\"></div><div style=\"flex:1\"><input type=\"text\" name=\"content\" placeholder=\"File contents\"></div><button type=\"submit\">Save File</button></form></div></div>")
}

#[rustfmt::skip]
pub fn render_admin_pane(tab: u8, store: &MeshStore, snap: &FleetSnapshot) -> String {
    if tab == 3 {
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
        return format!("<div class=\"card\"><h2>3. Access Control — Top-Level Shared Folders (Outside &lt;user&gt;)</h2><div class=\"grid\">{sh_rows}</div><form method=\"POST\" action=\"/admin/shared/create\" style=\"margin-top:1rem;padding-top:1rem;border-top:1px solid var(--border)\"><div class=\"row\"><div style=\"flex:1;min-width:240px\"><input type=\"text\" name=\"folder\" required placeholder=\"New Top-Level Shared Folder Name (e.g. Engineering)\"></div><button type=\"submit\">Create Shared Folder</button></div></form></div>");
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
        all_rows.push_str("<tr><td colspan=\"4\" style=\"color:var(--muted)\">0 enrolled spokes.</td></tr>");
    }
    format!("<div class=\"card\"><div class=\"node-header\"><h2>4. Mesh Hubs</h2><form method=\"POST\" action=\"/reconcile\"><button type=\"submit\" class=\"btn-secondary\">Reconcile</button></form></div><div class=\"grid\" style=\"margin-bottom:1rem\">{srv_cards}</div><form method=\"POST\" action=\"/admin/servers/add\" style=\"margin:1rem 0;padding:0.9rem 0;border-top:1px solid var(--border);border-bottom:1px solid var(--border)\"><div class=\"row\"><div style=\"flex:1;min-width:180px\"><input type=\"text\" name=\"name\" required placeholder=\"Hub Name (e.g. hyperion)\"></div><div style=\"flex:1;min-width:220px\"><input type=\"text\" name=\"host\" required placeholder=\"Tailscale IP / Host (e.g. 100.x.y.z)\"></div><button type=\"submit\">Add Mesh Hub</button></div></form><table><thead><tr><th>Principal</th><th>Spoke Name</th><th>Spoke Device ID</th><th>Scope</th></tr></thead><tbody>{all_rows}</tbody></table></div>")
}
