//! The Operations section of the Admin tab.
//!
//! These controls change how the daemon behaves rather than how it looks:
//! pausing the reconciler, changing its cadence, rotating a peer credential,
//! and rotating the session secret. Every change is written to the audit log.

use crate::admin_console::settings::{PortalSettings, MAX_INTERVAL, MIN_INTERVAL};
use crate::audit::AuditEntry;
use crate::fleet_config::FleetConfig;
use crate::render_portal::escape_html;

/// Show only the tail of a secret, enough to match it against a peer config.
fn key_hint(key: &str) -> String {
    let n = key.chars().count();
    if n < 8 {
        return "not configured".to_string();
    }
    let tail: String = key.chars().skip(n - 4).collect();
    format!("&bull;&bull;&bull;&bull; {tail}")
}

/// One row per hub offering a credential rotation.
fn key_rows(cfg: &FleetConfig) -> String {
    let mut out = String::new();
    for s in &cfg.servers {
        let (name, hint) = (escape_html(&s.name), key_hint(&s.syncthing_api_key));
        out.push_str(&format!(
            "<tr class=\"tree-child\" data-parent=\"admin:ops:keys\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:3.0rem\"><form method=\"POST\" action=\"/admin/ops/rotate-key\" style=\"display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap\"><input type=\"hidden\" name=\"server\" value=\"{name}\"><span style=\"font-family:var(--mono);font-size:0.78rem;min-width:7rem\">{name}</span><span style=\"font-family:var(--mono);font-size:0.74rem;color:var(--muted)\">current {hint}</span><input type=\"password\" name=\"api_key\" placeholder=\"New API key\" required autocomplete=\"off\" style=\"flex:1;min-width:12rem\"><button type=\"submit\" class=\"btn-secondary\" style=\"padding:0.22rem 0.6rem;font-size:0.74rem\">Rotate</button></form></td></tr>"
        ));
    }
    out
}

/// Render the recent administrative actions.
fn audit_rows(entries: &[AuditEntry], now: u64) -> String {
    if entries.is_empty() {
        return "<tr class=\"tree-child\" data-parent=\"admin:ops:audit\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:1.75rem;color:var(--muted);font-size:0.8rem\"><span class=\"tree-branch\">&#9492;&#9472;</span>No administrative actions recorded yet.</td></tr>".to_string();
    }
    let mut out = String::new();
    for e in entries.iter().rev() {
        let (actor, action) = (escape_html(&e.actor), escape_html(&e.action));
        let (target, outcome) = (escape_html(&e.target), escape_html(&e.outcome));
        let ok = outcome == "ok";
        out.push_str(&format!(
            "<tr class=\"tree-child\" data-parent=\"admin:ops:audit\" style=\"display:none\"><td style=\"padding-left:1.75rem;font-family:var(--mono);font-size:0.76rem\">{actor}</td><td><code>{action}</code></td><td><code style=\"color:var(--muted)\">{target}</code></td><td style=\"text-align:right\"><span class=\"badge {cls}\">{outcome}</span> <span style=\"font-family:var(--mono);font-size:0.72rem;color:var(--muted)\">{age}</span></td></tr>",
            cls = if ok { "badge-online" } else { "badge-warn" },
            age = crate::audit::age(e.ts, now)
        ));
    }
    out
}

/// Render the Operations section of the Admin table.
#[rustfmt::skip]
pub fn render_section(settings: &PortalSettings, cfg: &FleetConfig, entries: &[AuditEntry], now: u64) -> String {
    let tbtn = "<button type=\"button\" class=\"tree-toggle\" data-target=\"admin:ops\" aria-expanded=\"false\" aria-label=\"Expand Operations\">+</button>";
    let maint_badge = if settings.maintenance { "<span class=\"badge badge-warn\">MAINTENANCE</span>" } else { "<span class=\"badge badge-online\">ACTIVE</span>" };
    let head = format!("<tr><td>{tbtn}<strong>&#9881; Operations</strong></td><td><code style=\"color:var(--muted)\">Reconciler &bull; Credentials &bull; Audit</code></td><td>{maint_badge}</td><td style=\"text-align:right\"><span class=\"badge badge-online\">every {iv}s</span></td></tr>", iv = settings.interval);
    let maint = format!(
        "<div style=\"display:flex;gap:0.4rem;align-items:center;flex-wrap:wrap\"><form method=\"POST\" action=\"/admin/ops/maintenance\" style=\"display:inline\"><input type=\"hidden\" name=\"mode\" value=\"{next}\"><button type=\"submit\" class=\"{cls}\" style=\"padding:0.22rem 0.6rem;font-size:0.74rem\">{label}</button></form><span style=\"font-size:0.72rem;color:var(--muted)\">{hint}</span></div>",
        next = if settings.maintenance { "off" } else { "on" },
        cls = if settings.maintenance { "btn-secondary" } else { "btn-danger" },
        label = if settings.maintenance { "Resume Reconciling" } else { "Pause Reconciling" },
        hint = if settings.maintenance { "Paused: peers are polled but never changed." } else { "Pause to keep peer state frozen during maintenance." });
    let interval = format!(
        "<form method=\"POST\" action=\"/admin/ops/interval\" style=\"display:flex;gap:0.4rem;align-items:center\"><input type=\"number\" name=\"interval\" min=\"{min}\" max=\"{max}\" value=\"{iv}\" required style=\"width:6.5rem\"><span style=\"font-size:0.72rem;color:var(--muted)\">seconds ({min}-{max})</span><button type=\"submit\" class=\"btn-secondary\" style=\"padding:0.22rem 0.6rem;font-size:0.74rem\">Set</button></form>",
        min = MIN_INTERVAL, max = MAX_INTERVAL, iv = settings.interval);
    let rotate_secret = "<form method=\"POST\" action=\"/admin/ops/rotate-secret\"><button type=\"submit\" class=\"btn-danger\" style=\"padding:0.22rem 0.6rem;font-size:0.74rem\">Rotate Session Secret</button></form><span style=\"font-size:0.72rem;color:var(--warn);margin-left:0.45rem\">Signs every administrator out.</span>";
    let keys_toggle = "<button type=\"button\" class=\"tree-toggle\" data-target=\"admin:ops:keys\" aria-expanded=\"false\" aria-label=\"Expand Peer Credentials\">+</button>";
    let audit_toggle = "<button type=\"button\" class=\"tree-toggle\" data-target=\"admin:ops:audit\" aria-expanded=\"false\" aria-label=\"Expand Audit Log\">+</button>";
    format!("{head}<tr class=\"tree-child\" data-parent=\"admin:ops\" style=\"display:none\"><td colspan=\"4\" style=\"padding-left:1.75rem\"><div style=\"display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:0.8rem\"><div style=\"display:flex;flex-direction:column;gap:0.35rem\"><span class=\"field-label\">Maintenance mode</span>{maint}</div><div style=\"display:flex;flex-direction:column;gap:0.35rem\"><span class=\"field-label\">Reconciliation interval</span>{interval}</div><div style=\"display:flex;flex-direction:column;gap:0.35rem\"><span class=\"field-label\">Session secret</span>{rotate_secret}</div></div></td></tr><tr class=\"tree-child\" data-parent=\"admin:ops\" style=\"display:none\"><td style=\"padding-left:1.75rem\">{keys_toggle}<strong>Peer API keys</strong></td><td colspan=\"3\"><code style=\"color:var(--muted)\">One credential per peer, scoped to that peer&apos;s REST surface</code></td></tr>{keys}<tr class=\"tree-child\" data-parent=\"admin:ops\" style=\"display:none\"><td style=\"padding-left:1.75rem\">{audit_toggle}<strong>Audit log</strong></td><td colspan=\"3\"><code style=\"color:var(--muted)\">/config/audit.log &bull; owner-only &bull; last 500 records</code></td></tr>{audit}",
        keys = key_rows(cfg),
        audit = audit_rows(entries, now))
}
