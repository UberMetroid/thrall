//! Serialization of the declarative fleet configuration back to YAML.
//!
//! Kept separate from the parser so that `fleet_config.rs` stays focused on
//! reading configuration, and so the admin portal can write a config back to disk
//! without pulling the parser's dependencies along.

use std::fs;

use super::FleetConfig;

/// Render a fleet configuration as YAML.
pub fn serialize_fleet_yaml(cfg: &FleetConfig) -> String {
    let a: Vec<String> = cfg
        .ldap
        .admin_users
        .iter()
        .map(|u| format!("\"{u}\""))
        .collect();
    let (p, l, acsv) = (&cfg.portal, &cfg.ldap, a.join(", "));
    let mut out = format!("portal:\n  listen_addr: \"{}\"\n  database_path: \"{}\"\n  session_secret: \"{}\"\n\nldap:\n  url: \"{}\"\n  base_dn: \"{}\"\n  bind_dn: \"{}\"\n  bind_password: \"{}\"\n  user_filter: \"{}\"\n  admin_users: [{acsv}]\n\nservers:\n", p.listen_addr, p.database_path, p.session_secret, l.url, l.base_dn, l.bind_dn, l.bind_password, l.user_filter);
    for s in &cfg.servers {
        out.push_str(&format!("  - name: \"{}\"\n    local: {}\n    syncthing_api_url: \"{}\"\n    syncthing_api_key: \"{}\"\n    syncthing_wan_addr: \"{}\"\n", s.name, s.local, s.syncthing_api_url, s.syncthing_api_key, s.syncthing_wan_addr));
    }
    out
}

/// Write a fleet configuration to `path` as YAML.
pub fn save_fleet_config(path: &str, cfg: &FleetConfig) -> Result<(), String> {
    fs::write(path, serialize_fleet_yaml(cfg)).map_err(|e| e.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::fleet_config::parse_fleet_yaml;

    #[test]
    fn test_roundtrip_preserves_portal_and_server_names() {
        let cfg = parse_fleet_yaml(crate::fleet_config::DEFAULT_FLEET_YAML).unwrap();
        let yaml = serialize_fleet_yaml(&cfg);
        let back = parse_fleet_yaml(&yaml).unwrap();
        assert_eq!(back.portal.listen_addr, cfg.portal.listen_addr);
        assert_eq!(back.ldap.base_dn, cfg.ldap.base_dn);
        assert_eq!(back.servers.len(), cfg.servers.len());
        assert_eq!(back.servers[0].name, "us-west");
        assert_eq!(back.servers[1].name, "us-east");
        assert!(yaml.contains("session_secret:"));
    }
}
