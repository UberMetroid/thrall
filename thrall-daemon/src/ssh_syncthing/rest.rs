//! The tunneled Syncthing REST client.
//!
//! Every value that crosses the SSH boundary is shell-quoted here. `ssh` does not
//! run its trailing arguments directly: it joins them into a single string which
//! the remote login shell then parses. A value such as `a; touch /tmp/pwned`
//! travelling in one of those arguments would otherwise become a second command
//! on the remote host.

use std::io::Write;
use std::process::{Command, Stdio};

use super::shell;
use crate::fleet_config::ServerNodeConfig;

/// Build the remote command that performs one Syncthing REST call.
fn build_rest_command(
    node: &ServerNodeConfig,
    api_key: &str,
    method: &str,
    endpoint_path: &str,
    with_body: bool,
) -> Command {
    let base = node.syncthing_api_url.trim_end_matches('/');
    let url = format!("{base}{endpoint_path}");
    let header_key = format!("X-API-Key:{api_key}");
    let mut cmd = super::base_ssh_command(node);
    let q_method = shell::quote(method);
    let q_header = shell::quote(&header_key);
    cmd.args([
        "curl",
        "-fsS",
        "-X",
        q_method.as_str(),
        "-H",
        q_header.as_str(),
    ]);
    if with_body {
        cmd.args(["-H", "Content-Type:application/json", "--data-binary", "@-"]);
    }
    cmd.arg(shell::quote(&url));
    cmd
}

/// Perform one Syncthing REST call on a remote hub over SSH.
pub fn ssh_syncthing_rest(
    node: &ServerNodeConfig,
    api_key: &str,
    method: &str,
    endpoint_path: &str,
    json_body: Option<&str>,
) -> Result<String, String> {
    let mut cmd = build_rest_command(node, api_key, method, endpoint_path, json_body.is_some());

    if let Some(body) = json_body {
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("[{}] ssh curl spawn failed: {e}", node.name))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(body.as_bytes())
                .map_err(|e| format!("[{}] stdin write failed: {e}", node.name))?;
        }
        let out = child
            .wait_with_output()
            .map_err(|e| format!("[{}] ssh curl wait failed: {e}", node.name))?;
        if !out.status.success() {
            return Err(format!(
                "[{}] {method} {endpoint_path} failed: {}",
                node.name,
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        let out = cmd
            .output()
            .map_err(|e| format!("[{}] ssh curl exec failed: {e}", node.name))?;
        if !out.status.success() {
            return Err(format!(
                "[{}] {method} {endpoint_path} failed: {}",
                node.name,
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::fleet_config::ServerNodeConfig;

    fn hub() -> ServerNodeConfig {
        ServerNodeConfig {
            name: "hub".into(),
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
}
