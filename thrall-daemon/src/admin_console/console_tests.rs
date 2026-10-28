//! Behaviour tests for the admin console: what the appearance form changes,
//! what the operational controls change, and what the portal renders as a
//! result. These are the claims the Admin tab makes, checked.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use crate::admin_console::actions::apply_admin_console_post;
use crate::admin_console::palette::BgMode;
use crate::admin_console::settings::{PortalSettings, MAX_INTERVAL};
use crate::audit;
use crate::device_store::MeshStore;
use crate::fleet_config::{parse_fleet_yaml, FleetConfig, DEFAULT_FLEET_YAML};
use crate::reconcile_mesh::FleetSnapshot;
use crate::render_portal::about_ldap::render_footer_dock;
use crate::render_portal::explorer_view::render_admin_pane;

/// A scratch directory for one test.
fn temp_dir(name: &str) -> String {
    let d = std::env::temp_dir().join(format!("thrall_console_{name}"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.to_string_lossy().to_string()
}

/// A configuration pointed at a scratch directory, with no peers.
fn test_cfg(dir: &str) -> FleetConfig {
    let mut cfg = parse_fleet_yaml(DEFAULT_FLEET_YAML).unwrap();
    cfg.portal.database_path = format!("{dir}/roster.db");
    cfg.portal.session_secret = "0123456789abcdef0123456789abcdef".to_string();
    cfg.servers.clear();
    cfg
}

/// A form body as the parser sees it.
fn form(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[test]
fn test_appearance_form_changes_and_persists_the_settings() {
    let dir = temp_dir("appearance");
    let mut cfg = test_cfg(&dir);
    let mut store = MeshStore::default();
    let mut snap = FleetSnapshot::default();
    let pairs = [
        ("footer_upload", "1"),
        ("footer_hubs", "1"),
        ("palette", "1"),
        ("accent", "Coral"),
        ("bg_login", "pin"),
        ("bg_login_index", "3"),
        ("bg_dashboard", "off"),
    ];
    let body = pairs
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    let out = apply_admin_console_post(
        "/admin/ui/appearance",
        &body,
        &form(&pairs),
        "tester",
        &mut cfg,
        &mut store,
        &mut snap,
        &format!("{dir}/fleet.yaml"),
        &format!("{dir}/audit.log"),
    )
    .unwrap();
    assert!(out.ok);
    assert!(store.settings.footer_upload);
    assert!(!store.settings.footer_mesh, "an absent checkbox is off");
    assert_eq!(store.settings.accent, "Coral");
    assert_eq!(store.settings.bg_login, BgMode::Pinned);
    assert_eq!(store.settings.bg_login_index, 3);
    assert_eq!(store.settings.bg_dashboard, BgMode::Off);

    let reloaded = MeshStore::load(&cfg.portal.database_path);
    assert_eq!(
        reloaded.settings, store.settings,
        "settings survive a reload"
    );

    let entries = audit::recent(&format!("{dir}/audit.log"), 5);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].action, "ui.appearance");
    assert_eq!(entries[0].actor, "tester");
}

#[test]
fn test_footer_drops_a_disabled_box_and_a_locked_palette() {
    let s = PortalSettings {
        footer_mesh: false,
        palette: false,
        ..Default::default()
    };
    let html = render_footer_dock("", &s);
    assert!(!html.contains("footer-mesh-graph"));
    assert!(html.contains("footer-upload-graph"));
    assert!(!html.contains("footer-accent-picker"));
}

#[test]
fn test_admin_pane_renders_the_new_sections() {
    let dir = temp_dir("pane");
    let mut cfg = test_cfg(&dir);
    cfg.servers.push(crate::fleet_config::ServerNodeConfig {
        name: "hub-a".to_string(),
        local: false,
        syncthing_api_url: "http://hub-a:8384".to_string(),
        syncthing_api_key: "0123456789abcdef".to_string(),
        syncthing_wan_addr: "tcp://hub-a:22000".to_string(),
    });
    let store = MeshStore::default();
    let snap = FleetSnapshot::default();
    let html = render_admin_pane(&cfg, &store, &snap);
    for needle in [
        "Portal Appearance",
        "/admin/ui/appearance",
        "/admin/ops/maintenance",
        "/admin/ops/interval",
        "/admin/ops/rotate-key",
        "/admin/ops/rotate-secret",
        "Audit log",
    ] {
        assert!(html.contains(needle), "the admin pane is missing {needle}");
    }
}

#[test]
fn test_maintenance_mode_and_the_interval_are_applied() {
    let dir = temp_dir("ops");
    let mut cfg = test_cfg(&dir);
    let mut store = MeshStore::default();
    let mut snap = FleetSnapshot::default();
    let args = (&dir, &mut cfg, &mut store, &mut snap);

    let out = apply_admin_console_post(
        "/admin/ops/maintenance",
        "mode=on",
        &form(&[("mode", "on")]),
        "tester",
        args.1,
        args.2,
        args.3,
        &format!("{}/fleet.yaml", args.0),
        &format!("{}/audit.log", args.0),
    )
    .unwrap();
    assert!(out.ok);
    assert!(store.settings.maintenance);

    apply_admin_console_post(
        "/admin/ops/interval",
        "interval=99999",
        &form(&[("interval", "99999")]),
        "tester",
        &mut cfg,
        &mut store,
        &mut snap,
        &format!("{dir}/fleet.yaml"),
        &format!("{dir}/audit.log"),
    )
    .unwrap();
    assert_eq!(store.settings.interval, MAX_INTERVAL);
}

#[test]
fn test_rotating_a_peer_key_refuses_an_invalid_key() {
    let dir = temp_dir("rotate");
    let mut cfg = test_cfg(&dir);
    let mut store = MeshStore::default();
    let mut snap = FleetSnapshot::default();
    let out = apply_admin_console_post(
        "/admin/ops/rotate-key",
        "server=nope&api_key=short",
        &form(&[("server", "nope"), ("api_key", "short")]),
        "tester",
        &mut cfg,
        &mut store,
        &mut snap,
        &format!("{dir}/fleet.yaml"),
        &format!("{dir}/audit.log"),
    )
    .unwrap();
    assert!(!out.ok);
    let entries = audit::recent(&format!("{dir}/audit.log"), 5);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].outcome, "denied");
}

#[test]
fn test_an_unknown_path_is_not_ours() {
    let dir = temp_dir("unknown");
    let mut cfg = test_cfg(&dir);
    let mut store = MeshStore::default();
    let mut snap = FleetSnapshot::default();
    let out = apply_admin_console_post(
        "/some/other/path",
        "",
        &form(&[]),
        "tester",
        &mut cfg,
        &mut store,
        &mut snap,
        &format!("{dir}/fleet.yaml"),
        &format!("{dir}/audit.log"),
    );
    assert!(out.is_none());
}
