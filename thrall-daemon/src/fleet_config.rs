//! Declarative `/config/fleet.yaml` configuration parser and environment override merger.

use std::env;
use std::fs;

mod emit;
pub(crate) mod secret;

pub use emit::{save_fleet_config, serialize_fleet_yaml};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortalConfig {
    pub listen_addr: String,
    pub database_path: String,
    pub session_secret: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LdapConfig {
    pub url: String,
    pub base_dn: String,
    pub bind_dn: String,
    pub bind_password: String,
    pub user_filter: String,
    pub admin_users: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerNodeConfig {
    pub name: String,
    /// True when this peer shares the host with the daemon.
    pub local: bool,
    /// Base URL of the peer's REST API.
    pub syncthing_api_url: String,
    /// API key that authenticates the daemon against the peer's REST API.
    pub syncthing_api_key: String,
    /// The peer's Block Exchange Protocol address, advertised to other peers.
    pub syncthing_wan_addr: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FleetConfig {
    pub portal: PortalConfig,
    pub ldap: LdapConfig,
    pub servers: Vec<ServerNodeConfig>,
}

pub const DEFAULT_FLEET_YAML: &str = include_str!("../../assets/default-fleet.yaml");

fn unquote(val: &str) -> String {
    let trimmed = val.trim();
    let without_comment = if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        let quote = trimmed.chars().next().unwrap_or('"');
        if let Some(end_idx) = trimmed[1..].find(quote) {
            return trimmed[1..=end_idx].to_string();
        }
        trimmed
    } else if let Some((before, _)) = trimmed.split_once('#') {
        before.trim()
    } else {
        trimmed
    };
    without_comment
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn parse_inline_list(val: &str) -> Vec<String> {
    let trimmed = val.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(trimmed);
    inner
        .split(',')
        .map(unquote)
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn parse_fleet_yaml(raw: &str) -> Result<FleetConfig, String> {
    let mut cfg = FleetConfig {
        portal: PortalConfig {
            listen_addr: "0.0.0.0:8080".to_string(),
            database_path: "/config/roster.db".to_string(),
            session_secret: String::new(),
        },
        ldap: LdapConfig {
            url: "ldap://127.0.0.1:389".to_string(),
            base_dn: "dc=example,dc=com".to_string(),
            bind_dn: String::new(),
            bind_password: String::new(),
            user_filter: "(&(objectClass=person)(uid=%s))".to_string(),
            admin_users: Vec::new(),
        },
        servers: Vec::new(),
    };

    let mut section = "";
    let mut current_server: Option<ServerNodeConfig> = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') && !line.starts_with('\t') && trimmed.ends_with(':') {
            if let Some(srv) = current_server.take() {
                cfg.servers.push(srv);
            }
            section = trimmed.trim_end_matches(':');
            continue;
        }

        if section == "servers" && trimmed.starts_with("- ") {
            if let Some(srv) = current_server.take() {
                cfg.servers.push(srv);
            }
            let mut srv = ServerNodeConfig {
                name: String::new(),
                local: false,
                syncthing_api_url: "http://127.0.0.1:8384".to_string(),
                syncthing_api_key: String::new(),
                syncthing_wan_addr: "dynamic".to_string(),
            };
            let rest = trimmed.trim_start_matches("- ").trim();
            if let Some((k, v)) = rest.split_once(':') {
                apply_server_field(&mut srv, k.trim(), v);
            }
            current_server = Some(srv);
            continue;
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim();
            match section {
                "portal" => match key {
                    "listen_addr" => cfg.portal.listen_addr = unquote(v),
                    "database_path" => cfg.portal.database_path = unquote(v),
                    "session_secret" => cfg.portal.session_secret = unquote(v),
                    _ => {}
                },
                "ldap" => match key {
                    "url" => cfg.ldap.url = unquote(v),
                    "base_dn" => cfg.ldap.base_dn = unquote(v),
                    "bind_dn" => cfg.ldap.bind_dn = unquote(v),
                    "bind_password" => cfg.ldap.bind_password = unquote(v),
                    "user_filter" => cfg.ldap.user_filter = unquote(v),
                    "admin_users" => cfg.ldap.admin_users = parse_inline_list(v),
                    _ => {}
                },
                "servers" => {
                    if let Some(ref mut srv) = current_server {
                        apply_server_field(srv, key, v);
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(srv) = current_server.take() {
        cfg.servers.push(srv);
    }

    apply_env_overrides(&mut cfg);
    Ok(cfg)
}

fn apply_server_field(srv: &mut ServerNodeConfig, key: &str, raw_val: &str) {
    let val = unquote(raw_val);
    match key {
        "name" => srv.name = val,
        "local" => srv.local = val == "true" || val == "yes" || val == "1",
        "syncthing_api_url" => srv.syncthing_api_url = val,
        "syncthing_api_key" => srv.syncthing_api_key = val,
        "syncthing_wan_addr" => srv.syncthing_wan_addr = val,
        _ => {}
    }
}

fn env_override(var: &str, target: &mut String) {
    if let Ok(v) = env::var(var) {
        if !v.trim().is_empty() {
            *target = v;
        }
    }
}

pub fn apply_env_overrides(cfg: &mut FleetConfig) {
    env_override("PORTAL_LISTEN_ADDR", &mut cfg.portal.listen_addr);
    env_override("PORTAL_DATABASE_PATH", &mut cfg.portal.database_path);
    env_override("PORTAL_SESSION_SECRET", &mut cfg.portal.session_secret);
    env_override("LDAP_URL", &mut cfg.ldap.url);
    env_override("LDAP_BASE_DN", &mut cfg.ldap.base_dn);
    env_override("LDAP_BIND_DN", &mut cfg.ldap.bind_dn);
    env_override("LDAP_BIND_PASSWORD", &mut cfg.ldap.bind_password);
    env_override("LDAP_USER_FILTER", &mut cfg.ldap.user_filter);
    if let Ok(v) = env::var("ADMIN_USERS") {
        let list = parse_inline_list(&v);
        if !list.is_empty() {
            cfg.ldap.admin_users = list;
        }
    }
}

pub fn load_or_create_fleet_config(path: &str) -> Result<FleetConfig, String> {
    let mut cfg = if let Ok(content) = fs::read_to_string(path) {
        parse_fleet_yaml(&content)?
    } else {
        let _ = secret::write_private_file(path, DEFAULT_FLEET_YAML);
        parse_fleet_yaml(DEFAULT_FLEET_YAML)?
    };
    secret::ensure_session_secret(&mut cfg, path)?;
    Ok(cfg)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_fleet_yaml() {
        let cfg = parse_fleet_yaml(DEFAULT_FLEET_YAML).unwrap();
        assert_eq!(cfg.portal.listen_addr, "0.0.0.0:8080");
        assert_eq!(cfg.ldap.base_dn, "dc=example,dc=com");
        assert!(cfg.ldap.admin_users.contains(&"admin".to_string()));
        assert_eq!(cfg.servers.len(), 2);
        assert_eq!(cfg.servers[0].name, "us-west");
        assert_eq!(cfg.servers[1].name, "us-east");
    }
}
