//! About view (`/?tab=about`), 8-Bit Footer Dock, and Admin LDAP rows.

use crate::admin_console::palette::ACCENTS;
use crate::admin_console::settings::PortalSettings;
use crate::device_store::MeshStore;
use crate::fleet_config::FleetConfig;
use crate::reconcile_mesh::FleetSnapshot;
use crate::render_portal::escape_html;

/// The eight accent swatches shown in the footer picker.
fn accent_picker(settings: &PortalSettings) -> String {
    let mut out = String::from(
        "<div class=\"footer-accent-picker\" id=\"footer-accent-picker\" role=\"radiogroup\" aria-label=\"8-Bit Rainbow Palette\">",
    );
    for a in ACCENTS.iter() {
        let active = a.name.eq_ignore_ascii_case(&settings.accent);
        out.push_str(&format!(
            "<button type=\"button\" class=\"dock-accent-dot{cls}\" data-accent-name=\"{name}\" style=\"--swatch:{hex};background-color:{hex};\" role=\"radio\" aria-checked=\"{checked}\" tabindex=\"{tab}\" aria-label=\"{name}\"></button>",
            cls = if active { " active" } else { "" },
            name = a.name,
            hex = a.hex,
            checked = active,
            tab = if active { "0" } else { "-1" },
        ));
    }
    out.push_str("</div>");
    out
}

#[rustfmt::skip]
pub fn render_footer_dock(status_html: &str, settings: &PortalSettings) -> String {
    let mut widgets = String::new();
    if settings.footer_upload {
        widgets.push_str("<canvas id=\"footer-upload-graph\" class=\"footer-upload-graph\" width=\"128\" height=\"24\" title=\"Browser &lt;-&gt; Mesh Data Flow\" aria-label=\"Browser &lt;-&gt; Mesh Data Flow\"></canvas>");
    }
    if settings.footer_hubs {
        widgets.push_str("<canvas id=\"footer-hubs-eq\" class=\"footer-pixel-widget\" width=\"84\" height=\"24\" title=\"Regional Hub Signal &amp; Ping (RTT)\" aria-label=\"Regional Hub Signal &amp; Ping (RTT)\"></canvas>");
    }
    if settings.footer_mesh {
        widgets.push_str("<canvas id=\"footer-mesh-graph\" class=\"footer-pixel-widget\" width=\"116\" height=\"24\" title=\"Mesh Hub Replication Traffic (BEP)\" aria-label=\"Mesh Hub Replication Traffic (BEP)\"></canvas>");
    }
    if settings.footer_vault {
        widgets.push_str("<canvas id=\"footer-vault-gauge\" class=\"footer-pixel-widget\" width=\"92\" height=\"24\" title=\"Mesh Vault Mass &amp; Active Links\" aria-label=\"Mesh Vault Mass &amp; Active Links\"></canvas>");
    }
    if settings.footer_clock {
        widgets.push_str("<canvas id=\"footer-uptime-clock\" class=\"footer-pixel-widget\" width=\"76\" height=\"24\" title=\"Mesh Uptime\" aria-label=\"Mesh Uptime\"></canvas>");
    }
    let picker = if settings.palette { accent_picker(settings) } else { String::new() };
    format!("<footer class=\"footer-dock\" id=\"footer-dock\" aria-label=\"Theme Palette\"><div class=\"footer-dock-inner\"><div class=\"footer-left-slot\">{widgets}{status_html}</div><div></div><div class=\"dock-controls\">{picker}</div></div></footer>")
}

#[rustfmt::skip]
pub fn render_about_pane() -> String {
    let topics = [
        ("about:thrall", "What is THRALL?", "Tool for Hierarchical Replication And Identity Linking", "https://github.com/UberMetroid/thrall", "GitHub Repo", "https://ubermetroid.github.io/thrall/", "Landing Page", "THRALL is a zero-trust multi-server Syncthing control plane and self-service web portal. Workstations (Devices) peer exclusively with regional Mesh Hubs over TLS 1.3 and never connect directly to other users' workstations. Supports Personal folders, multi-user Shared folders with Read-Only (RO) / Read-Write (RW) access, per-device Selective Sync, and instant web uploads."),
        ("about:syncthing", "Syncthing & Block Exchange Protocol (BEP)", "TLS 1.3 + Ed25519 Content-Addressed Sync Engine", "https://syncthing.net/", "syncthing.net", "https://docs.syncthing.net/", "Documentation", "Syncthing provides the underlying cryptographic Device IDs (56-character Base32 with Luhn mod 32 check digits) and block-level delta replication. Each workstation runs its own local Syncthing daemon (http://127.0.0.1:8384) while THRALL orchestrates folder topology across regional hubs."),
        ("about:tuition", "Tuition, Not Coffee", "Skip the coffee — help send my daughter to college", "https://github.com/UberMetroid/tuition-not-coffee", "tuition-not-coffee", "https://ubermetroid.github.io/tuition-not-coffee", "College Fund", "Every developer has a 'Buy Me a Coffee' link. We already have plenty of coffee—so 100% of sponsorships for open-source projects in this ecosystem go directly toward a college & future education fund."),
        ("about:arch", "Zero-Dependency Rust & Page Rules", "Pure-std Rust (#![forbid(unsafe_code)]) & 16-256 Line Pages", "https://github.com/UberMetroid/thrall/releases", "Releases", "https://github.com/UberMetroid/thrall/blob/main/AGENTS.md", "Architecture & Rules", "Compiled with zero external crates in [dependencies] for minimal attack surface and instant reproducible builds. Every source page is strictly bounded between 16 and 256 lines, with symlink-jailed path resolution and 8-bit live telemetry."),
    ];
    let mut rows = String::new();
    for (tkey, title, sub, u1, l1, u2, l2, desc) in topics {
        let (t_esc, l1_esc, l2_esc) = (escape_html(title), escape_html(l1), escape_html(l2));
        let tbtn = format!("<button type=\"button\" class=\"tree-toggle\" data-target=\"{tkey}\" aria-expanded=\"false\" aria-label=\"Expand {t_esc}\">+</button>");
        rows.push_str(&format!(
            "<tr><td>{tbtn}<strong>&#128214; {t_esc}</strong></td><td><code style=\"color:var(--muted)\">{sub}</code></td><td><a href=\"{u1}\" target=\"_blank\" rel=\"noopener\" style=\"color:var(--accent);font-family:var(--mono);font-size:0.76rem\">{l1_esc} &nearr;</a></td><td style=\"text-align:right\"><a href=\"{u2}\" target=\"_blank\" rel=\"noopener\" class=\"btn-secondary\" style=\"padding:0.2rem 0.55rem;border-radius:4px;text-decoration:none;font-size:0.76rem\">{l2_esc} &nearr;</a></td></tr>\
             <tr class=\"tree-child\" data-parent=\"{tkey}\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:1.75rem;white-space:normal;color:var(--text-secondary);font-size:0.82rem;line-height:1.55\"><span class=\"tree-branch\">&#9492;&#9472;</span>{desc}</td></tr>"
        ));
    }
    format!("<table class=\"explorer-table\"><thead><tr><th>Topic</th><th>Summary</th><th style=\"width:180px\">Project Link</th><th style=\"width:160px;text-align:right\">Resource</th></tr></thead><tbody>{rows}</tbody></table>")
}

#[rustfmt::skip]
pub fn render_ldap_admin_rows(cfg: &FleetConfig, store: &MeshStore, snap: &FleetSnapshot) -> String {
    let l = &cfg.ldap;
    let (url_e, base_e, bind_e, filt_e, adm_e) = (escape_html(&l.url), escape_html(&l.base_dn), escape_html(&l.bind_dn), escape_html(&l.user_filter), escape_html(&l.admin_users.join(", ")));
    let u_cnt = snap.ldap_users.len();
    let badge = if u_cnt > 0 { format!("<span class=\"badge badge-online\">ACTIVE ({u_cnt} principals)</span>") } else { "<span class=\"badge badge-warn\">0 PRINCIPALS</span>".to_string() };
    let t_ldap = "<button type=\"button\" class=\"tree-toggle\" data-target=\"admin:ldap\" aria-expanded=\"false\" aria-label=\"Expand LDAP Directory\">+</button>";
    let test_form = "<form method=\"POST\" action=\"/admin/ldap/test\" style=\"display:inline\"><button type=\"submit\" class=\"btn-secondary\" style=\"padding:0.18rem 0.55rem;font-size:0.74rem\">Validate / Test</button></form>";
    let mut out = format!(
        "<tr><td>{t_ldap}<strong>&#128273; LDAP Directory</strong></td><td><code>{url_e}</code> &bull; <code style=\"color:var(--muted)\">{base_e}</code></td><td>{badge}</td><td style=\"text-align:right\">{test_form}</td></tr>\
         <tr class=\"tree-child\" data-parent=\"admin:ldap\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:1.75rem;white-space:normal\">\
         <form method=\"POST\" action=\"/admin/ldap/update\" style=\"display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:0.65rem;align-items:end;padding:0.35rem 0\">\
         <div><label class=\"field-label\">LDAP URL</label><input type=\"text\" name=\"url\" value=\"{url_e}\" required></div>\
         <div><label class=\"field-label\">Base DN</label><input type=\"text\" name=\"base_dn\" value=\"{base_e}\" required></div>\
         <div><label class=\"field-label\">Bind DN</label><input type=\"text\" name=\"bind_dn\" value=\"{bind_e}\" required></div>\
         <div><label class=\"field-label\">Bind Password</label><input type=\"password\" name=\"bind_password\" placeholder=\"Leave blank to keep current\"></div>\
         <div><label class=\"field-label\">User Filter</label><input type=\"text\" name=\"user_filter\" value=\"{filt_e}\" required></div>\
         <div><label class=\"field-label\">Admin Users (CSV)</label><input type=\"text\" name=\"admin_users\" value=\"{adm_e}\" required></div>\
         <div style=\"display:flex;justify-content:flex-end\"><button type=\"submit\" style=\"padding:0.45rem 0.85rem;font-size:0.78rem\">Save &amp; Validate</button></div>\
         </form></td></tr>"
    );
    for u in &snap.ldap_users {
        let u_esc = escape_html(u);
        let is_adm = l.admin_users.iter().any(|a| a == u);
        let role_b = if is_adm { "<span class=\"badge badge-admin\">ADMIN</span>" } else { "<span class=\"badge badge-online\">USER</span>" };
        let dev_cnt = store.users.get(u).map_or(0, Vec::len);
        let f_cnt = store.folders_for_user(u).len();
        out.push_str(&format!("<tr class=\"tree-child\" data-parent=\"admin:ldap\" style=\"display:none\"><td style=\"padding-left:1.75rem\"><span class=\"tree-branch\">&#9492;&#9472;</span>&#128100; <code>{u_esc}</code></td><td><code>uid={u_esc},{base_e}</code></td><td>{role_b}</td><td style=\"text-align:right\"><code>{dev_cnt} devices &bull; {f_cnt} folders</code></td></tr>"));
    }
    if snap.ldap_users.is_empty() {
        out.push_str("<tr class=\"tree-child\" data-parent=\"admin:ldap\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:1.75rem;color:var(--muted);font-size:0.8rem\"><span class=\"tree-branch\">&#9492;&#9472;</span>0 directory principals discovered.</td></tr>");
    }
    out
}
