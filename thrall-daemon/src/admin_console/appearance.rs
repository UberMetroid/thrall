//! The Portal Appearance section of the Admin tab.
//!
//! Every control here changes one field of [`PortalSettings`]. The form posts
//! the whole appearance at once, because the settings are a single record and
//! a partially applied theme is never what an administrator intended.

use crate::admin_console::palette::{BgMode, ACCENTS};
use crate::admin_console::settings::{PortalSettings, BG_COUNT};
use crate::render_portal::escape_html;

/// One on/off control.
fn toggle(name: &str, label: &str, hint: &str, checked: bool) -> String {
    let chk = if checked { " checked" } else { "" };
    format!(
        "<label class=\"user-chip\" style=\"justify-content:space-between;cursor:pointer\"><span>{label}</span><span style=\"display:inline-flex;align-items:center;gap:0.4rem;font-size:0.7rem;color:var(--muted)\">{hint}<input type=\"checkbox\" name=\"{name}\" value=\"1\"{chk}></span></label>"
    )
}

/// The background mode chooser for one surface.
fn bg_mode_select(name: &str, mode: BgMode) -> String {
    let opt = |v: &str, label: &str, sel: bool| {
        let s = if sel { " selected" } else { "" };
        format!("<option value=\"{v}\"{s}>{label}</option>")
    };
    format!(
        "<select name=\"{name}\">{}{}{}</select>",
        opt("off", "Off", mode == BgMode::Off),
        opt("random", "Random", mode == BgMode::Random),
        opt("pin", "Pinned", mode == BgMode::Pinned)
    )
}

/// The background image chooser for one surface.
fn bg_index_select(name: &str, index: u8) -> String {
    let mut opts = String::new();
    for i in 1..=BG_COUNT {
        let s = if i == index { " selected" } else { "" };
        opts.push_str(&format!("<option value=\"{i}\"{s}>Image {i}</option>"));
    }
    format!("<select name=\"{name}\">{opts}</select>")
}

/// The accent chooser.
fn accent_select(current: &str) -> String {
    let mut opts = String::new();
    for a in ACCENTS.iter() {
        let s = if a.name.eq_ignore_ascii_case(current) {
            " selected"
        } else {
            ""
        };
        opts.push_str(&format!(
            "<option value=\"{}\"{s} style=\"color:{}\">{}</option>",
            escape_html(a.name),
            a.hex,
            escape_html(a.name)
        ));
    }
    format!("<select name=\"accent\">{opts}</select>")
}

/// Render the Portal Appearance section of the Admin table.
#[rustfmt::skip]
pub fn render_section(settings: &PortalSettings) -> String {
    let tbtn = "<button type=\"button\" class=\"tree-toggle\" data-target=\"admin:appearance\" aria-expanded=\"false\" aria-label=\"Expand Portal Appearance\">+</button>";
    let head = format!("<tr><td>{tbtn}<strong>&#127912; Portal Appearance</strong></td><td><code style=\"color:var(--muted)\">Footer &bull; Theme &bull; Background &bull; Motion</code></td><td><span class=\"badge badge-online\">{accent}</span></td><td style=\"text-align:right\"><span class=\"badge {badge}\">{footer}</span></td></tr>",
        accent = escape_html(&settings.accent),
        badge = if settings.all_footer_on() { "badge-online" } else { "badge-warn" },
        footer = if settings.all_footer_on() { "5/5 boxes" } else { "partial" });
    let boxes = [
        ("footer_upload", "Browser &lt;-&gt; Mesh Data Flow", settings.footer_upload),
        ("footer_hubs", "Regional Hub Signal &amp; Ping", settings.footer_hubs),
        ("footer_mesh", "Mesh Replication Traffic", settings.footer_mesh),
        ("footer_vault", "Mesh Vault Mass &amp; Links", settings.footer_vault),
        ("footer_clock", "Mesh Uptime", settings.footer_clock),
    ]
    .iter()
    .map(|(n, l, c)| toggle(n, l, "", *c))
    .collect::<Vec<_>>()
    .join("");
    let form = format!(
        "<form method=\"POST\" action=\"/admin/ui/appearance\"><div style=\"display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:0.75rem\"><div style=\"display:flex;flex-direction:column;gap:0.35rem\"><span class=\"field-label\">Footer data boxes</span><div class=\"acl-user-grid\">{boxes}</div><div style=\"display:flex;gap:0.4rem;align-self:flex-start\"><button type=\"submit\" name=\"footer_all\" value=\"on\" class=\"btn-secondary\" style=\"padding:0.22rem 0.6rem;font-size:0.74rem\">All on</button><button type=\"submit\" name=\"footer_all\" value=\"off\" class=\"btn-secondary\" style=\"padding:0.22rem 0.6rem;font-size:0.74rem\">All off</button></div></div><div style=\"display:flex;flex-direction:column;gap:0.35rem\"><span class=\"field-label\">Theme</span><div class=\"acl-user-grid\">{palette}{mono}</div><label class=\"field-label\" style=\"margin-top:0.3rem\">Default accent</label>{accent_sel}</div><div style=\"display:flex;flex-direction:column;gap:0.35rem\"><span class=\"field-label\">Background</span><label class=\"field-label\">Sign-in page</label><div style=\"display:flex;gap:0.4rem\">{login_mode}{login_idx}</div><label class=\"field-label\" style=\"margin-top:0.3rem\">Dashboard</label><div style=\"display:flex;gap:0.4rem\">{dash_mode}{dash_idx}</div><div class=\"acl-user-grid\" style=\"margin-top:0.3rem\">{spot}{motion}</div></div></div><div style=\"display:flex;justify-content:flex-end;margin-top:0.7rem\"><button type=\"submit\" class=\"btn-secondary\" style=\"padding:0.3rem 0.8rem;font-size:0.78rem\">Save Appearance</button></div></form>",
        palette = toggle("palette", "Palette picker visible", "", settings.palette),
        mono = toggle("monochrome", "Monochrome", "", settings.monochrome),
        accent_sel = accent_select(&settings.accent),
        login_mode = bg_mode_select("bg_login", settings.bg_login),
        login_idx = bg_index_select("bg_login_index", settings.bg_login_index),
        dash_mode = bg_mode_select("bg_dashboard", settings.bg_dashboard),
        dash_idx = bg_index_select("bg_dashboard_index", settings.bg_dashboard_index),
        spot = toggle("spotlight", "Sign-in spotlight canvas", "", settings.spotlight),
        motion = toggle("reduce_motion", "Reduce motion", "", settings.reduce_motion));
    format!("{head}<tr class=\"tree-child\" data-parent=\"admin:appearance\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:1.75rem\">{form}</td></tr>")
}
