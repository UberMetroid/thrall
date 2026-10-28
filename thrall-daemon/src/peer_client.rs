//! Client for reaching the Syncthing peers that THRALL manages.
//!
//! A peer that shares the host with the daemon is reached directly over its
//! REST API (`direct.rs`), so THRALL never needs SSH access to its own machine.
//! A peer on another host is reached over SSH (`ssh.rs`).
//!
//! Which transport is used is decided per peer by the `local` flag in
//! `fleet.yaml`.

mod direct;
mod parse;
mod shell;
mod ssh;

pub use parse::{
    extract_xml_tag, parse_json_bool_field, parse_json_string_field, parse_json_u64_field,
};

use crate::fleet_config::ServerNodeConfig;

/// Perform a Syncthing REST call against a peer.
pub fn syncthing_rest(
    node: &ServerNodeConfig,
    api_key: &str,
    method: &str,
    endpoint_path: &str,
    json_body: Option<&str>,
) -> Result<String, String> {
    if node.local {
        direct::direct_syncthing_rest(
            &node.syncthing_api_url,
            api_key,
            method,
            endpoint_path,
            json_body,
        )
    } else {
        ssh::ssh_syncthing_rest(node, api_key, method, endpoint_path, json_body)
    }
}

/// Find the API key of a peer.
///
/// A configured key always wins. A local peer must be configured, because
/// reading the peer's config file would mean reading another container's
/// private state.
pub fn discover_api_key(node: &ServerNodeConfig) -> Result<String, String> {
    if !node.syncthing_api_key.trim().is_empty() {
        return Ok(node.syncthing_api_key.trim().to_string());
    }
    if node.local {
        return Err(format!(
            "[{}] local peer has no syncthing_api_key: set it in fleet.yaml",
            node.name
        ));
    }
    ssh::discover_remote_api_key(node)
}

/// Create the given directories on a peer.
pub fn ensure_folders(node: &ServerNodeConfig, dirs: &[String]) -> Result<(), String> {
    if node.local {
        return direct::direct_mkdir_many(dirs);
    }
    ssh::ssh_mkdir_many(node, dirs)
}

/// Open the mesh firewall on a peer.
///
/// The host manages its own firewall, so this is a no-op for a local peer.
pub fn ensure_trusted_network(node: &ServerNodeConfig) -> Result<(), String> {
    if node.local {
        return Ok(());
    }
    ssh::ssh_ensure_trusted_network(node)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::process::Command;

    use super::ssh::build_rest_command;
    use super::{discover_api_key, ensure_folders};
    use crate::fleet_config::ServerNodeConfig;

    fn hub() -> ServerNodeConfig {
        ServerNodeConfig {
            name: "hub".into(),
            local: false,
            ssh_host: "100.64.0.1:22".into(),
            ssh_user: "ubermetroid".into(),
            ssh_key_path: String::new(),
            syncthing_api_url: "http://127.0.0.1:8384".into(),
            syncthing_api_key: String::new(),
            syncthing_wan_addr: "dynamic".into(),
            users_base_path: "/users".into(),
            shared_base_path: "/shared".into(),
        }
    }

    fn remote_command(cmd: &Command) -> Vec<String> {
        cmd.get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn test_rest_command_quotes_user_controlled_values() {
        // A sub path carrying shell metacharacters must not escape its argument.
        let evil = "/rest/db/scan?folder=x&sub=a; touch /tmp/pwned";
        let cmd = build_rest_command(&hub(), "secretkey", "POST", evil, false);
        let args = remote_command(&cmd);
        let url_arg = args.last().unwrap().clone();
        assert_eq!(
            url_arg,
            "'http://127.0.0.1:8384/rest/db/scan?folder=x&sub=a; touch /tmp/pwned'"
        );
        // The remote command is the args joined by spaces; the URL must be one
        // single-quoted word so the remote shell cannot see a command separator.
        let remote = args.join(" ");
        assert!(remote.contains(&url_arg));
        assert!(remote.contains("'X-API-Key:secretkey'"));
    }

    #[test]
    fn test_rest_command_quotes_the_api_key_header() {
        let cmd = build_rest_command(&hub(), "key; rm -rf /", "GET", "/rest/system/status", false);
        let args = remote_command(&cmd);
        assert!(args.contains(&"'X-API-Key:key; rm -rf /'".to_string()));
    }

    #[test]
    fn test_local_peer_never_falls_back_to_ssh() {
        let mut node = hub();
        node.local = true;

        // A configured key is used as-is.
        node.syncthing_api_key = "configured-key".into();
        assert_eq!(discover_api_key(&node).unwrap(), "configured-key");

        // Without one it must fail rather than reach for SSH, because the local
        // peer has no config file THRALL is allowed to read.
        node.syncthing_api_key = String::new();
        assert!(discover_api_key(&node)
            .unwrap_err()
            .contains("syncthing_api_key"));

        // Folder creation for a local peer must touch the local filesystem.
        let dir = std::env::temp_dir().join("thrall_local_peer_dirs");
        let _ = std::fs::remove_dir_all(&dir);
        ensure_folders(&node, &[dir.to_string_lossy().into_owned()]).unwrap();
        assert!(dir.is_dir());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
