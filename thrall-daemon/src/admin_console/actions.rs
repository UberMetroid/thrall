//! POST handlers for the admin console: the appearance form and the
//! operational controls.
//!
//! The logic is kept separate from the HTTP write so it can be exercised
//! directly: `apply_admin_console_post` changes state and records the
//! outcome, and the wrapper turns that into a redirect.

use std::collections::BTreeMap;
use std::net::TcpStream;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::admin_console::palette::{Accent, BgMode, ACCENTS};
use crate::admin_console::settings::PortalSettings;
use crate::audit;
use crate::device_store::MeshStore;
use crate::fleet_config::{save_fleet_config, FleetConfig};
use crate::portal_server::write_redirect;
use crate::reconcile_mesh::{reconcile_fleet, FleetSnapshot};

/// The outcome of one admin console action.
pub struct Outcome {
    pub ok: bool,
    pub message: String,
}

/// True when a named checkbox was submitted. An unchecked box is absent.
fn checked(body: &str, key: &str) -> bool {
    body.split('&').any(|p| {
        p.split_once('=')
            .map(|(k, _)| k.trim() == key)
            .unwrap_or(false)
    })
}

/// The current wall-clock time in seconds since the epoch.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Read an integer form field, falling back to `fallback`.
fn number(form: &BTreeMap<String, String>, key: &str, fallback: u32) -> u32 {
    form.get(key)
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(fallback)
}

/// Apply the submitted appearance form to the stored settings.
fn apply_appearance(body: &str, form: &BTreeMap<String, String>, settings: &mut PortalSettings) {
    match form.get("footer_all").map(String::as_str) {
        Some("on") => settings.set_footer_all(true),
        Some("off") => settings.set_footer_all(false),
        _ => {
            settings.footer_upload = checked(body, "footer_upload");
            settings.footer_hubs = checked(body, "footer_hubs");
            settings.footer_mesh = checked(body, "footer_mesh");
            settings.footer_vault = checked(body, "footer_vault");
            settings.footer_clock = checked(body, "footer_clock");
        }
    }
    settings.palette = checked(body, "palette");
    settings.monochrome = checked(body, "monochrome");
    settings.spotlight = checked(body, "spotlight");
    settings.reduce_motion = checked(body, "reduce_motion");
    if let Some(a) = form.get("accent") {
        if ACCENTS
            .iter()
            .any(|x: &Accent| x.name.eq_ignore_ascii_case(a.trim()))
        {
            settings.accent = a.trim().to_string();
        }
    }
    settings.bg_login = BgMode::parse(form.get("bg_login").map(String::as_str).unwrap_or("random"));
    settings.bg_dashboard = BgMode::parse(
        form.get("bg_dashboard")
            .map(String::as_str)
            .unwrap_or("random"),
    );
    settings.bg_login_index = PortalSettings::clamp_index(number(form, "bg_login_index", 1) as u8);
    settings.bg_dashboard_index =
        PortalSettings::clamp_index(number(form, "bg_dashboard_index", 1) as u8);
}

/// Apply an admin console action. Returns `None` when the path is not ours.
#[allow(clippy::too_many_arguments)]
pub fn apply_admin_console_post(
    path: &str,
    body: &str,
    form: &BTreeMap<String, String>,
    username: &str,
    cfg: &mut FleetConfig,
    store: &mut MeshStore,
    snap: &mut FleetSnapshot,
    config_path: &str,
    audit_path: &str,
) -> Option<Outcome> {
    let record = |action: &str, target: &str, ok: bool| {
        audit::append(
            audit_path,
            now(),
            username,
            action,
            target,
            if ok { "ok" } else { "denied" },
        );
    };
    if path == "/admin/ui/appearance" {
        let mut settings = store.settings.clone();
        apply_appearance(body, form, &mut settings);
        let summary = format!(
            "accent={} palette={} mono={} bg_login={} bg_dashboard={} motion={}",
            settings.accent,
            settings.palette,
            settings.monochrome,
            settings.bg_login.as_str(),
            settings.bg_dashboard.as_str(),
            settings.reduce_motion
        );
        store.settings = settings;
        let _ = store.save(&cfg.portal.database_path);
        record("ui.appearance", &summary, true);
        return Some(Outcome {
            ok: true,
            message: "Appearance+saved".to_string(),
        });
    }
    if path == "/admin/ops/maintenance" {
        let on = form.get("mode").map(String::as_str).unwrap_or("off") == "on";
        store.settings.maintenance = on;
        let _ = store.save(&cfg.portal.database_path);
        record(
            "ops.maintenance",
            if on { "enabled" } else { "disabled" },
            true,
        );
        let message = if on {
            "Maintenance+mode+enabled+-+reconciler+paused"
        } else {
            "Reconciling+resumed"
        };
        return Some(Outcome {
            ok: true,
            message: message.to_string(),
        });
    }
    if path == "/admin/ops/interval" {
        let seconds =
            PortalSettings::clamp_interval(number(form, "interval", store.settings.interval));
        store.settings.interval = seconds;
        let _ = store.save(&cfg.portal.database_path);
        let target = format!("interval={seconds}s");
        record("ops.interval", &target, true);
        return Some(Outcome {
            ok: true,
            message: "Reconciliation+interval+updated".to_string(),
        });
    }
    if path == "/admin/ops/rotate-key" {
        let server = form
            .get("server")
            .map(String::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let key = form
            .get("api_key")
            .map(String::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if crate::peer_client::validate_api_key(&key).is_err() {
            record("ops.key.rotate", &format!("server={server}"), false);
            return Some(Outcome {
                ok: false,
                message: "Invalid+API+key".to_string(),
            });
        }
        let Some(node) = cfg.servers.iter_mut().find(|s| s.name == server) else {
            record("ops.key.rotate", &format!("server={server}"), false);
            return Some(Outcome {
                ok: false,
                message: "Unknown+hub".to_string(),
            });
        };
        node.syncthing_api_key = key;
        let _ = save_fleet_config(config_path, cfg);
        *snap = reconcile_fleet(cfg, store);
        record("ops.key.rotate", &format!("server={server}"), true);
        return Some(Outcome {
            ok: true,
            message: "API+key+rotated".to_string(),
        });
    }
    if path == "/admin/ops/rotate-secret" {
        return match crate::fleet_config::secret::rotate_session_secret(cfg, config_path) {
            Ok(()) => {
                record("ops.secret.rotate", "session_secret", true);
                Some(Outcome {
                    ok: true,
                    message: "Session+secret+rotated+-+sign+in+again".to_string(),
                })
            }
            Err(_) => {
                record("ops.secret.rotate", "session_secret", false);
                Some(Outcome {
                    ok: false,
                    message: "Secret+rotation+failed".to_string(),
                })
            }
        };
    }
    None
}

/// Handle a POST from the admin console. Returns true when it was consumed.
#[allow(clippy::too_many_arguments)]
pub fn handle_admin_console_post(
    stream: &mut TcpStream,
    path: &str,
    body: &str,
    form: &BTreeMap<String, String>,
    username: &str,
    cfg: &mut FleetConfig,
    store: &mut MeshStore,
    snap: &mut FleetSnapshot,
) -> bool {
    let Some(outcome) = apply_admin_console_post(
        path,
        body,
        form,
        username,
        cfg,
        store,
        snap,
        crate::fleet_config::CONFIG_PATH,
        &audit::path_for(&cfg.portal.database_path),
    ) else {
        return false;
    };
    let loc = if outcome.ok {
        format!("/?tab=admin&ok={}", outcome.message)
    } else {
        format!("/?tab=admin&err={}", outcome.message)
    };
    write_redirect(stream, &loc, "");
    true
}
