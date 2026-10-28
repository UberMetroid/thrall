//! Client for reaching the Syncthing peers that THRALL manages.
//!
//! Every peer is reached over its own REST API, authenticated with that
//! peer's API key. A peer on the same host is reached over the loopback
//! interface; a peer on another host publishes the same API on a private
//! overlay network. THRALL opens no SSH connection and needs no SSH access
//! to any machine, neither its own nor a peer's.
//!
//! Which address is used is decided per peer by the `local` flag and the
//! `syncthing_api_url` in `fleet.yaml`.

mod api;
mod parse;
mod url;

pub use parse::{parse_json_bool_field, parse_json_string_field, parse_json_u64_field};
pub use url::{host_of, validate_api_key, validate_api_url};

use std::fs;

use crate::fleet_config::ServerNodeConfig;

/// Prepare a peer's folder directory.
///
/// A peer that shares the host also shares its storage, so the daemon creates
/// the directory and its marker itself. A remote peer creates the folder path
/// on its own when the folder configuration is applied, so there is nothing
/// to prepare here and no reason to reach the host at all.
pub fn prepare_folder(srv: &ServerNodeConfig, path: &str) -> Result<(), String> {
    if !srv.local {
        return Ok(());
    }
    fs::create_dir_all(format!("{path}/.stfolder"))
        .map_err(|e| format!("[{}] mkdir {path} failed: {e}", srv.name))
}

/// Perform a Syncthing REST call against a peer.
pub fn syncthing_rest(
    node: &ServerNodeConfig,
    api_key: &str,
    method: &str,
    endpoint_path: &str,
    json_body: Option<&str>,
) -> Result<String, String> {
    api::peer_rest(node, api_key, method, endpoint_path, json_body)
}

/// Find the API key of a peer.
///
/// The key is always configured. It is the credential that authenticates
/// THRALL against the peer's REST API, and there is no other way to read it:
/// the peer's configuration file is private to the peer.
pub fn discover_api_key(node: &ServerNodeConfig) -> Result<String, String> {
    let key = node.syncthing_api_key.trim();
    if key.is_empty() {
        return Err(format!(
            "[{}] no syncthing_api_key configured: set it in fleet.yaml",
            node.name
        ));
    }
    if !node.local && url::is_loopback(&node.syncthing_api_url) {
        return Err(format!(
            "[{}] syncthing_api_url is a loopback address but the peer is not on this host: \
             a remote peer must publish its API on a reachable address",
            node.name
        ));
    }
    Ok(key.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn hub() -> ServerNodeConfig {
        ServerNodeConfig {
            name: "hub".into(),
            local: false,
            syncthing_api_url: "http://100.64.0.1:8384".into(),
            syncthing_api_key: "0123456789abcdef0123".into(),
            syncthing_wan_addr: "tcp://100.64.0.1:22000".into(),
        }
    }

    #[test]
    fn test_configured_api_key_is_used() {
        let node = hub();
        assert_eq!(discover_api_key(&node).unwrap(), "0123456789abcdef0123");
    }

    #[test]
    fn test_missing_api_key_is_refused() {
        let mut node = hub();
        node.syncthing_api_key = String::new();
        assert!(discover_api_key(&node)
            .unwrap_err()
            .contains("syncthing_api_key"));
    }

    #[test]
    fn test_remote_peer_may_not_be_reached_over_loopback() {
        // Reaching a remote peer over loopback would land the call on the
        // local peer instead, silently reconciling the wrong machine.
        let mut node = hub();
        node.syncthing_api_url = "http://127.0.0.1:8384".into();
        assert!(discover_api_key(&node).unwrap_err().contains("loopback"));

        // The same address is correct for a peer that shares the host.
        node.local = true;
        assert!(discover_api_key(&node).is_ok());
    }

    #[test]
    fn test_loopback_peer_is_allowed_when_local() {
        let mut node = hub();
        node.local = true;
        node.syncthing_api_url = "http://127.0.0.1:8384".into();
        assert!(discover_api_key(&node).is_ok());
    }

    #[test]
    fn test_only_a_local_peer_gets_its_folder_prepared_here() {
        let base = std::env::temp_dir().join("thrall_prepare_folder_test");
        let _ = std::fs::remove_dir_all(&base);

        // A peer on this host shares its storage, so the daemon creates the
        // folder directory and its marker.
        let local_path = base
            .join("alice")
            .join("Docs")
            .to_string_lossy()
            .into_owned();
        let mut node = hub();
        node.local = true;
        prepare_folder(&node, &local_path).unwrap();
        assert!(std::path::Path::new(&local_path).join(".stfolder").is_dir());

        // A remote peer creates its own folder path when its folder
        // configuration is applied. Nothing is written on this host.
        let remote_path = base.join("bob").join("Docs").to_string_lossy().into_owned();
        node.local = false;
        prepare_folder(&node, &remote_path).unwrap();
        assert!(!std::path::Path::new(&remote_path).exists());

        let _ = std::fs::remove_dir_all(&base);
    }
}
