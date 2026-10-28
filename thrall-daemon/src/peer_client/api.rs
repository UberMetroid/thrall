//! Authenticated REST transport for a Syncthing peer.
//!
//! The call is made with `curl`, invoked directly with an argument vector.
//! No shell is involved, so nothing needs shell quoting and no value can be
//! reinterpreted as a command. The peer's API key travels as one argument in
//! an `X-API-Key` header.
//!
//! The same transport reaches a peer on this host over the loopback interface
//! and a peer on another host over a private overlay network. Only the
//! address differs; the credential and the call are identical.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::fleet_config::ServerNodeConfig;

use super::url::build_url;

/// Build the command for one REST call against a peer.
pub(super) fn build_command(
    node: &ServerNodeConfig,
    api_key: &str,
    method: &str,
    endpoint_path: &str,
    with_body: bool,
) -> Command {
    let url = build_url(&node.syncthing_api_url, endpoint_path);
    let header = format!("X-API-Key:{api_key}");
    let mut cmd = Command::new("curl");
    cmd.args(["-fsS", "--max-time", "20", "-X", method, "-H", &header]);
    if with_body {
        cmd.args(["-H", "Content-Type:application/json", "--data-binary", "@-"]);
    }
    cmd.arg(url);
    cmd
}

/// Perform one Syncthing REST call against a peer.
pub fn peer_rest(
    node: &ServerNodeConfig,
    api_key: &str,
    method: &str,
    endpoint_path: &str,
    json_body: Option<&str>,
) -> Result<String, String> {
    let mut cmd = build_command(node, api_key, method, endpoint_path, json_body.is_some());
    let Some(body) = json_body else {
        let out = cmd
            .output()
            .map_err(|e| format!("[{}] curl exec failed: {e}", node.name))?;
        return finish(node, out, method, endpoint_path);
    };
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("[{}] curl spawn failed: {e}", node.name))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(body.as_bytes())
            .map_err(|e| format!("[{}] stdin write failed: {e}", node.name))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("[{}] curl wait failed: {e}", node.name))?;
    finish(node, out, method, endpoint_path)
}

fn finish(
    node: &ServerNodeConfig,
    out: std::process::Output,
    method: &str,
    endpoint_path: &str,
) -> Result<String, String> {
    if !out.status.success() {
        return Err(format!(
            "[{}] {method} {endpoint_path} failed: {}",
            node.name,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn node(url: &str) -> ServerNodeConfig {
        ServerNodeConfig {
            name: "hub".into(),
            local: false,
            syncthing_api_url: url.into(),
            syncthing_api_key: "k".into(),
            syncthing_wan_addr: "tcp://100.64.0.1:22000".into(),
        }
    }

    fn args(cmd: &Command) -> Vec<String> {
        cmd.get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn test_command_always_carries_the_url() {
        // A GET must still be given the URL; leaving it off silently sends a
        // request with no target and the peer never sees the call.
        let cmd = build_command(
            &node("http://100.64.0.1:8384"),
            "k",
            "GET",
            "/rest/system/status",
            false,
        );
        let a = args(&cmd);
        assert!(a.contains(&"http://100.64.0.1:8384/rest/system/status".to_string()));
        assert!(a.contains(&"X-API-Key:k".to_string()));
        assert!(a.contains(&"GET".to_string()));
    }

    #[test]
    fn test_command_with_body_reads_stdin() {
        let cmd = build_command(
            &node("http://100.64.0.1:8384"),
            "k",
            "PUT",
            "/rest/config/folders/abc",
            true,
        );
        let a = args(&cmd);
        assert!(a.contains(&"http://100.64.0.1:8384/rest/config/folders/abc".to_string()));
        assert!(a.contains(&"@-".to_string()));
        assert!(a.contains(&"Content-Type:application/json".to_string()));
    }

    #[test]
    fn test_api_key_is_one_argument_and_is_never_quoted() {
        // There is no shell between THRALL and the peer, so the key must be
        // passed through untouched rather than wrapped in quotes.
        let cmd = build_command(
            &node("http://100.64.0.1:8384"),
            "abc123",
            "GET",
            "/rest/system/status",
            false,
        );
        let a = args(&cmd);
        assert!(a.contains(&"X-API-Key:abc123".to_string()));
        assert!(!a.iter().any(|x| x.contains('\'')));
    }
}
