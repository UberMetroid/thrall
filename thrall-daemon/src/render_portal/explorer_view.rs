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

#[rustfmt::skip]
fn render_share_cells(user: &str, store: &MeshStore, s_br: &str, f_br: &str, sub: &str, item: &str) -> (String, String) {
    let (sc_br, i_rel) = if s_br.is_empty() { item.split_once('/').unwrap_or((item, "")) } else { (s_br, item.strip_prefix(&format!("{s_br}/")).unwrap_or(item)) };
    let scope = browse_scope(user, store, sc_br);
    let target = if i_rel.is_empty() { scope } else { format!("{scope}/{i_rel}") };
    let tok = compute_share_token(&target);
    let on = store.public_shares.contains_key(&tok);
    let (lbl, cls) = if on { ("Share: ON", "badge-online") } else { ("Share: OFF", "btn-secondary") };
    let link = if on { format!("<a href=\"/s/{tok}\" target=\"_blank\" style=\"color:var(--accent);font-family:var(--mono);font-size:0.76rem\">/s/{tok}</a>") } else { String::new() };
    let (br_e, sub_e, it_e) = (escape_html(f_br), escape_html(sub), escape_html(item));
    let btn = format!("<form method=\"POST\" action=\"/shares/toggle\" style=\"display:inline\"><input type=\"hidden\" name=\"browse\" value=\"{br_e}\"><input type=\"hidden\" name=\"sub\" value=\"{sub_e}\"><input type=\"hidden\" name=\"item\" value=\"{it_e}\"><button type=\"submit\" class=\"{cls}\" style=\"padding:0.18rem 0.5rem;font-size:0.74rem\">{lbl}</button></form>");
    (btn, link)
}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
fn render_dir_rows(user: &str, store: &MeshStore, br_scope: &str, form_br: &str, dir_path: &Path, rel: &str, pkey: &str, depth: usize, out: &mut String) {
    let etab = if br_scope.starts_with("shared:") { 2 } else { 1 };
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
        out.push_str(&format!("<tr{p_attr}><td colspan=\"4\" style=\"padding-left:{pad:.2}rem;color:var(--muted);font-size:0.8rem\">{pfx}{msg}</td></tr>"));
    }
    for (_, name, is_dir, sz, cpath) in entries {
        let n_esc = escape_html(&name);
        let item_rel = if rel.is_empty() { name.clone() } else { format!("{rel}/{name}") };
        let next = escape_html(&item_rel);
        let form_it = if form_br.is_empty() { format!("{br_scope}/{item_rel}") } else { item_rel.clone() };
        let fit_e = escape_html(&form_it);
        let (sh_btn, sh_lnk) = render_share_cells(user, store, br_scope, form_br, &rel_e, &form_it);
        let del_f = format!("<button type=\"button\" class=\"btn-danger\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\" data-del-action=\"/files/delete\" data-del-name=\"{n_esc}\" data-del-browse=\"{fbr_e}\" data-del-sub=\"{rel_e}\" data-del-item=\"{fit_e}\">Delete</button>");
        if is_dir {
            let ckey = format!("{br_e}:{next}");
            let tbtn = if depth < 3 { format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{ckey}\" aria-expanded=\"false\" aria-label=\"Expand {n_esc}\">+</button>") } else { String::new() };
            out.push_str(&format!("<tr{p_attr} data-drop-browse=\"{br_e}\" data-drop-sub=\"{next}\"><td style=\"padding-left:{pad:.2}rem\">{pfx}{tbtn}<a href=\"/?tab={etab}&amp;browse={br_e}&amp;sub={next}\" style=\"color:var(--accent);text-decoration:none;font-weight:600\">&#128193; {n_esc}/</a></td><td>{sh_btn}</td><td>{sh_lnk}</td><td style=\"text-align:right\">{del_f}</td></tr>"));
            if depth < 3 { render_dir_rows(user, store, br_scope, form_br, &cpath, &item_rel, &ckey, depth + 1, out); }
        } else {
            let sz_s = if sz >= 1024 { format!("{:.1} KiB", sz as f64 / 1024.0) } else { format!("{sz} B") };
            out.push_str(&format!("<tr{p_attr} data-drop-browse=\"{br_e}\" data-drop-sub=\"{rel_e}\"><td style=\"padding-left:{pad:.2}rem\">{pfx}<a href=\"/files?browse={br_e}&amp;sub={next}\" style=\"color:var(--text);text-decoration:none\">&#128196; {n_esc}</a> <code style=\"color:var(--muted);font-size:0.74rem;margin-left:0.35rem\">{sz_s}</code></td><td>{sh_btn}</td><td>{sh_lnk}</td><td style=\"text-align:right\"><a href=\"/files?browse={br_e}&amp;sub={next}\" class=\"btn-secondary\" style=\"padding:0.2rem 0.55rem;border-radius:4px;text-decoration:none;font-size:0.78rem\">Download</a> {del_f}</td></tr>"));
        }
    }
    if !pkey.is_empty() {
        let lbl = if rel.is_empty() { br_scope.split_once(':').map(|(_, f)| f).unwrap_or(br_scope) } else { rel };
        let lbl_e = escape_html(lbl);
        out.push_str(&format!("<tr{p_attr} data-drop-browse=\"{br_e}\" data-drop-sub=\"{rel_e}\"><td colspan=\"4\" style=\"padding-left:{pad:.2}rem\"><div class=\"row\" style=\"gap:0.45rem\">{pfx}<button type=\"button\" class=\"btn-secondary\" style=\"padding:0.18rem 0.5rem;font-size:0.74rem\" data-modal-open=\"mkdir\" data-browse=\"{br_e}\" data-sub=\"{rel_e}\" data-ret-br=\"{fbr_e}\" data-label=\"{lbl_e}\">+ Subfolder</button><button type=\"button\" style=\"padding:0.18rem 0.55rem;font-size:0.74rem\" data-modal-open=\"upload-file\" data-browse=\"{br_e}\" data-sub=\"{rel_e}\" data-ret-br=\"{fbr_e}\" data-label=\"{lbl_e}\">+ Upload</button></div></td></tr>"));
    }
}

#[rustfmt::skip]
pub fn render_file_explorer(user: &str, store: &MeshStore, tab: u8, browse: &str, sub: &str) -> String {
    if browse.is_empty() || browse == "personal" || browse == "shared" {
        if tab == 2 || browse == "shared" {
            let mut s_rows = String::new();
            for (fname, _) in store.shared_folders.iter().filter(|(_, a)| a.contains(user)) {
                let (f_esc, br_s) = (escape_html(fname), format!("shared:{fname}"));
                let k_esc = escape_html(&br_s);
                let (sh_btn, sh_lnk) = render_share_cells(user, store, "", "", "", &br_s);
                let tbtn = format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{k_esc}\" aria-expanded=\"false\" aria-label=\"Expand {f_esc}\">+</button>");
                s_rows.push_str(&format!("<tr data-drop-browse=\"{k_esc}\" data-drop-sub=\"\"><td>{tbtn}<a href=\"/?tab=2&amp;browse=shared:{f_esc}\" style=\"color:var(--purple);text-decoration:none;font-weight:700\">&#127760; {f_esc}/</a></td><td>{sh_btn}</td><td>{sh_lnk}</td><td style=\"text-align:right\"><span style=\"color:var(--muted);font-size:0.76rem\">&mdash;</span></td></tr>"));
                let p = if fname == "Shared" { Path::new("/shared").to_path_buf() } else { Path::new("/shared").join(fname) };
                render_dir_rows(user, store, &br_s, "", &p, "", &br_s, 1, &mut s_rows);
            }
            if s_rows.is_empty() {
                s_rows.push_str("<tr><td colspan=\"4\" style=\"color:var(--muted)\">0 shared folders assigned to your account.</td></tr>");
            }
            return format!("<table class=\"explorer-table\"><thead><tr><th>Folder</th><th style=\"width:110px\">Share</th><th style=\"width:200px\">Link</th><th style=\"width:180px;text-align:right\">Action</th></tr></thead><tbody>{s_rows}</tbody></table>");
        }
        let u_list = store.folders_for_user(user);
        let ex_esc = escape_html(&u_list.join(","));
        let mut u_rows = String::new();
        for f in u_list {
            let (f_esc, br_p) = (escape_html(&f), format!("personal:{f}"));
            let k_esc = escape_html(&br_p);
            let (sh_btn, sh_lnk) = render_share_cells(user, store, "", "", "", &br_p);
            let del_f = format!("<button type=\"button\" class=\"btn-danger\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\" data-del-action=\"/user/folders/delete\" data-del-name=\"{f_esc}\" data-del-browse=\"\" data-del-sub=\"\" data-del-item=\"\">Delete</button>");
            let tbtn = format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{k_esc}\" aria-expanded=\"false\" aria-label=\"Expand {f_esc}\">+</button>");
            u_rows.push_str(&format!("<tr data-drop-browse=\"{k_esc}\" data-drop-sub=\"\"><td>{tbtn}<a href=\"/?tab=1&amp;browse=personal:{f_esc}\" style=\"color:var(--accent);text-decoration:none;font-weight:700\">&#128193; {f_esc}/</a></td><td>{sh_btn}</td><td>{sh_lnk}</td><td style=\"text-align:right\">{del_f}</td></tr>"));
            let p = Path::new("/users").join(user).join(&f);
            render_dir_rows(user, store, &br_p, "", &p, "", &br_p, 1, &mut u_rows);
        }
        if u_rows.is_empty() {
            u_rows.push_str("<tr><td colspan=\"4\" style=\"color:var(--muted)\">0 folders accessible yet. Create a personal folder above.</td></tr>");
        }
        return format!("<table class=\"explorer-table\"><thead><tr><th>Folder &nbsp;<button type=\"button\" class=\"btn-secondary\" style=\"padding:0.14rem 0.48rem;font-size:0.72rem;text-transform:none\" data-modal-open=\"create-personal-folder\" data-existing=\"{ex_esc}\">+ New Folder</button></th><th style=\"width:110px\">Share</th><th style=\"width:200px\">Link</th><th style=\"width:180px;text-align:right\">Action</th></tr></thead><tbody>{u_rows}</tbody></table>");
    }
    let Some((label, dir_path, rel)) = resolve_jailed_path(user, store, browse, sub) else { return String::new(); };
    let etab = if browse.starts_with("shared:") { 2 } else { 1 };
    let (br_esc, lbl_esc, rel_esc) = (escape_html(browse), escape_html(&label), escape_html(&rel));
    let mut crumbs = format!("<a href=\"/?tab={etab}\" class=\"btn-secondary\" style=\"padding:0.14rem 0.48rem;border-radius:4px;text-decoration:none;font-size:0.74rem;text-transform:none\">All Folders</a> / <a href=\"/?tab={etab}&amp;browse={br_esc}\" class=\"btn-secondary\" style=\"padding:0.14rem 0.48rem;border-radius:4px;text-decoration:none;font-size:0.74rem;text-transform:none\">&#128194; {lbl_esc}</a>");
    let mut accum = String::new();
    for seg in rel.split('/').filter(|s| !s.is_empty()) {
        if !accum.is_empty() { accum.push('/'); }
        accum.push_str(seg);
        let (a_esc, s_esc) = (escape_html(&accum), escape_html(seg));
        crumbs.push_str(&format!(" / <a href=\"/?tab={etab}&amp;browse={br_esc}&amp;sub={a_esc}\" class=\"btn-secondary\" style=\"padding:0.14rem 0.48rem;border-radius:4px;text-decoration:none;font-size:0.74rem;text-transform:none\">{s_esc}</a>"));
    }
    let (dir_sh_btn, dir_sh_lnk) = render_share_cells(user, store, browse, browse, &rel, &rel);
    let mut rows = String::new();
    render_dir_rows(user, store, browse, browse, &dir_path, &rel, "", 0, &mut rows);
    let act_btns = format!("<button type=\"button\" class=\"btn-secondary\" style=\"padding:0.14rem 0.48rem;font-size:0.72rem;text-transform:none\" data-modal-open=\"mkdir\" data-browse=\"{br_esc}\" data-sub=\"{rel_esc}\" data-ret-br=\"{br_esc}\" data-label=\"{lbl_esc}\">+ Subfolder</button> <button type=\"button\" style=\"padding:0.14rem 0.48rem;font-size:0.72rem;text-transform:none\" data-modal-open=\"upload-file\" data-browse=\"{br_esc}\" data-sub=\"{rel_esc}\" data-ret-br=\"{br_esc}\" data-label=\"{lbl_esc}\">+ Upload</button>");
    format!("<table class=\"explorer-table\" data-drop-browse=\"{br_esc}\" data-drop-sub=\"{rel_esc}\"><thead><tr><th>{crumbs} &nbsp;{act_btns}</th><th style=\"width:110px\">{dir_sh_btn}</th><th style=\"width:200px\">{dir_sh_lnk}</th><th style=\"width:180px;text-align:right\">Action</th></tr></thead><tbody>{rows}</tbody></table>")
}

#[rustfmt::skip]
pub fn render_admin_pane(tab: u8, store: &MeshStore, snap: &FleetSnapshot) -> String {
    if tab == 4 {
        let mut sh_rows = String::new();
        for (fname, allowed) in &store.shared_folders {
            let (f_esc, is_root) = (escape_html(fname), fname == "Shared");
            let fid = escape_html(&syncthing_folder_id(&format!("shared:{}:mesh", fname.to_ascii_lowercase())));
            let sub = if is_root { String::new() } else { format!("/{f_esc}") };
            let tkey = format!("acl:{f_esc}");
            let mut checks = String::new();
            for u in snap.ldap_users.iter().filter(|u| u.as_str() != "root") {
                let (u_esc, chk) = (escape_html(u), if allowed.contains(u) { " checked" } else { "" });
                let acc = snap.shared_user_accepted.get(&(fname.clone(), u.clone())).copied().unwrap_or(false);
                let mark = if acc { " &#10003;" } else { "" };
                checks.push_str(&format!("<label class=\"user-chip\"><input type=\"checkbox\" name=\"users\" value=\"{u_esc}\"{chk}>{u_esc}{mark}</label>"));
            }
            let tbtn = format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{tkey}\" aria-expanded=\"false\" aria-label=\"Expand {f_esc}\">+</button>");
            let u_cnt = allowed.len();
            let del_btn = format!("<button type=\"button\" class=\"btn-danger\" style=\"padding:0.18rem 0.45rem;font-size:0.74rem\" data-del-action=\"/admin/shared/delete\" data-del-name=\"{f_esc}\">Delete</button>");
            sh_rows.push_str(&format!("<tr><td>{tbtn}<strong>&#127760; {f_esc}</strong></td><td><code>{fid}</code> &bull; <code style=\"color:var(--muted)\">/shared{sub}</code></td><td><span class=\"badge badge-online\">{u_cnt} users</span></td><td style=\"text-align:right\">{del_btn}</td></tr><tr class=\"tree-child\" data-parent=\"{tkey}\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:1.75rem\"><form method=\"POST\" action=\"/admin/shared/update\"><input type=\"hidden\" name=\"folder\" value=\"{f_esc}\"><div class=\"row\" style=\"justify-content:space-between;align-items:flex-start;gap:0.75rem\"><div class=\"acl-user-grid\">{checks}</div><button type=\"submit\" class=\"btn-secondary acl-save-btn\" data-default-label=\"Save Access\" style=\"padding:0.25rem 0.65rem;font-size:0.76rem\">Save Access</button></div></form></td></tr>"));
        }
        if sh_rows.is_empty() {
            sh_rows.push_str("<tr><td colspan=\"4\" style=\"color:var(--muted)\">0 shared folders configured.</td></tr>");
        }
        return format!("<table class=\"explorer-table\"><thead><tr><th>Shared Folder &nbsp;<button type=\"button\" class=\"btn-secondary\" style=\"padding:0.14rem 0.48rem;font-size:0.72rem;text-transform:none\" data-modal-open=\"create-shared-folder\">+ Shared Folder</button></th><th>Folder ID</th><th style=\"width:140px\">Users</th><th style=\"width:140px;text-align:right\">Action</th></tr></thead><tbody>{sh_rows}</tbody></table>");
    }
    let mut rows = String::new();
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
        let tkey = format!("hub:{n_esc}");
        let tbtn = format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{tkey}\" aria-expanded=\"false\" aria-label=\"Expand {n_esc}\">+</button>");
        let mut kids = String::new();
        for (u, devs) in &store.users {
            let u_esc = escape_html(u);
            for d in devs {
                let (l_esc, id_esc) = (escape_html(&d.label), escape_html(&d.device_id));
                kids.push_str(&format!("<tr class=\"tree-child\" data-parent=\"{tkey}\" style=\"display:none\"><td style=\"padding-left:1.75rem\"><span class=\"tree-branch\">&#9492;&#9472;</span>&#128187; {l_esc} (<code>{u_esc}</code>)</td><td><span class=\"badge badge-online\">DEVICE</span></td><td><code>{id_esc}</code></td><td style=\"text-align:right\"><code>/users/{u_esc}/*</code></td></tr>"));
            }
        }
        if kids.is_empty() {
            kids.push_str(&format!("<tr class=\"tree-child\" data-parent=\"{tkey}\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:1.75rem;color:var(--muted);font-size:0.8rem\"><span class=\"tree-branch\">&#9492;&#9472;</span>0 enrolled devices across fleet.</td></tr>"));
        }
        rows.push_str(&format!("<tr><td>{tbtn}<strong>{n_esc}</strong></td><td>{badge}</td><td><code>{wan_esc}</code> &bull; <code style=\"color:var(--muted)\">{info_esc}</code></td><td style=\"text-align:right\"><form method=\"POST\" action=\"/admin/servers/toggle\" style=\"display:inline\"><input type=\"hidden\" name=\"server\" value=\"{n_esc}\"><button type=\"submit\" class=\"{btn_cls}\" style=\"padding:0.18rem 0.55rem;font-size:0.74rem\">{btn_lbl}</button></form></td></tr>{kids}"));
    }
    format!("<table class=\"explorer-table\"><thead><tr><th>Hub &nbsp;<button type=\"button\" class=\"btn-secondary\" style=\"padding:0.14rem 0.48rem;font-size:0.72rem;text-transform:none\" data-modal-open=\"add-mesh-hub\">+ Add Hub</button> <form method=\"POST\" action=\"/reconcile\" style=\"display:inline\"><button type=\"submit\" class=\"btn-secondary\" style=\"padding:0.14rem 0.48rem;font-size:0.72rem;text-transform:none\">Reconcile</button></form></th><th style=\"width:130px\">Status</th><th>Address / Device ID</th><th style=\"width:140px;text-align:right\">Action</th></tr></thead><tbody>{rows}</tbody></table>")
}
