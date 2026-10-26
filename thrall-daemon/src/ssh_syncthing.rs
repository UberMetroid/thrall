//! Zero-shell SSH transport for remote directory provisioning and tunneled Syncthing REST API calls.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::fleet_config::ServerNodeConfig;

fn split_ssh_host_port(ssh_host: &str) -> (&str, &str) {
    if let Some((h, p)) = ssh_host.rsplit_once(':') {
        if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) {
            return (h, p);
        }
    }
    (ssh_host, "22")
}

fn base_ssh_command(node: &ServerNodeConfig) -> Command {
    let (host, port) = split_ssh_host_port(&node.ssh_host);
    let mut cmd = Command::new("ssh");
    cmd.args([
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=5",
        "-o",
        "StrictHostKeyChecking=accept-new",
        "-o",
        "UserKnownHostsFile=/config/known_hosts",
        "-o",
        "ControlMaster=auto",
        "-o",
        "ControlPath=/tmp/st-%r@%h:%p",
        "-o",
        "ControlPersist=600",
        "-p",
        port,
    ]);
    if !node.ssh_key_path.is_empty() && Path::new(&node.ssh_key_path).exists() {
        cmd.args(["-i", &node.ssh_key_path]);
    }
    cmd.arg(format!("{}@{}", node.ssh_user, host));
    cmd
}

pub fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    let val = xml[start..end].trim();
    (!val.is_empty()).then(|| val.to_string())
}

pub fn parse_json_string_field(json: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{field}\"");
    let idx = json.find(&pattern)? + pattern.len();
    let after_colon = json[idx..].find(':')? + idx + 1;
    let rest = json[after_colon..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let inner = &rest[1..];
    let end_quote = inner.find('"')?;
    Some(inner[..end_quote].to_string())
}

pub fn parse_json_u64_field(json: &str, field: &str) -> Option<u64> {
    let pattern = format!("\"{field}\"");
    let idx = json.find(&pattern)? + pattern.len();
    let after_colon = json[idx..].find(':')? + idx + 1;
    let rest = json[after_colon..].trim_start();
    let digits: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let int_part = digits.split('.').next().unwrap_or("");
    int_part.parse::<u64>().ok()
}

pub fn parse_json_bool_field(json: &str, field: &str) -> Option<bool> {
    let pattern = format!("\"{field}\"");
    let idx = json.find(&pattern)? + pattern.len();
    let after_colon = json[idx..].find(':')? + idx + 1;
    let rest = json[after_colon..].trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

pub fn ssh_mkdir_many(node: &ServerNodeConfig, remote_dirs: &[String]) -> Result<(), String> {
    for d in remote_dirs {
        if !d.starts_with('/')
            || d.contains("..")
            || !d
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '-' | '_'))
        {
            return Err(format!("Refusing unsafe remote path: {d}"));
        }
    }
    let mut cmd = base_ssh_command(node);
    cmd.args(["mkdir", "-p"]).args(remote_dirs);
    let out = cmd
        .output()
        .map_err(|e| format!("[{}] ssh mkdir failed: {e}", node.name))?;
    if !out.status.success() {
        return Err(format!(
            "[{}] mkdir -p failed: {}",
            node.name,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

const THRALL_NFT_RULES: &str = "add table inet thrall_mesh\nflush table inet thrall_mesh\ntable inet thrall_mesh {\n  chain input {\n    type filter hook input priority filter - 5; policy accept;\n    iifname \"tailscale0\" accept\n    ip saddr 100.64.0.0/10 iifname \"tailscale0\" accept\n    ip saddr { 192.168.0.0/16, 10.88.0.0/16, 10.89.0.0/16 } accept\n  }\n  chain forward {\n    type filter hook forward priority filter - 5; policy accept;\n    iifname \"tailscale0\" accept\n    oifname \"tailscale0\" accept\n    ip saddr { 192.168.0.0/16, 10.88.0.0/16, 10.89.0.0/16 } accept\n    ip daddr { 192.168.0.0/16, 10.88.0.0/16, 10.89.0.0/16 } accept\n  }\n}\n";

pub fn ssh_ensure_trusted_network(node: &ServerNodeConfig) -> Result<(), String> {
    let mut cmd = base_ssh_command(node);
    cmd.args(["sudo", "-n", "nft", "-f", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("[{}] ssh nft spawn failed: {e}", node.name))?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(THRALL_NFT_RULES.as_bytes());
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("[{}] ssh nft wait failed: {e}", node.name))?;
    if !out.status.success() {
        return Err(format!(
            "[{}] nft reconcile failed: {}",
            node.name,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(())
}

pub fn discover_remote_api_key(node: &ServerNodeConfig) -> Result<String, String> {
    if !node.syncthing_api_key.trim().is_empty() {
        return Ok(node.syncthing_api_key.trim().to_string());
    }
    let parent = Path::new(&node.shared_base_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("/srv/thrall-exp");
    let c1 = format!("{parent}/syncthing/config/config.xml");
    let candidates = [
        c1.as_str(),
        "/config/config.xml",
        "/srv/thrall-exp/syncthing/config/config.xml",
        "/var/syncthing/config/config.xml",
    ];
    for path in candidates {
        let mut cmd = base_ssh_command(node);
        cmd.args(["cat", path]);
        if let Some(out) = cmd.output().ok().filter(|o| o.status.success()) {
            let xml = String::from_utf8_lossy(&out.stdout);
            if let Some(key) = extract_xml_tag(&xml, "apikey") {
                if key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                {
                    return Ok(key);
                }
            }
        }
    }
    Err(format!("[{}] Missing Syncthing <apikey>", node.name))
}

pub fn ssh_syncthing_rest(
    node: &ServerNodeConfig,
    api_key: &str,
    method: &str,
    endpoint_path: &str,
    json_body: Option<&str>,
) -> Result<String, String> {
    let base = node.syncthing_api_url.trim_end_matches('/');
    let url = format!("{base}{endpoint_path}");
    let header_key = format!("X-API-Key:{api_key}");

    let mut cmd = base_ssh_command(node);
    cmd.args(["curl", "-fsS", "-X", method, "-H", &header_key]);
    if json_body.is_some() {
        cmd.args(["-H", "Content-Type:application/json", "--data-binary", "@-"]);
    }
    cmd.arg(&url);

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

    #[test]
    fn test_xml_and_json_extractors() {
        let xml = "<configuration><gui><apikey>abc123xyz</apikey></gui></configuration>";
        assert_eq!(extract_xml_tag(xml, "apikey").as_deref(), Some("abc123xyz"));

        let status_json = r#"{"myID": "T4U7YLJ-I6PFVWY", "uptime": 42, "connected": true}"#;
        assert_eq!(
            parse_json_string_field(status_json, "myID").as_deref(),
            Some("T4U7YLJ-I6PFVWY")
        );
        assert_eq!(parse_json_u64_field(status_json, "uptime"), Some(42));
        assert_eq!(parse_json_bool_field(status_json, "connected"), Some(true));
    }
}
