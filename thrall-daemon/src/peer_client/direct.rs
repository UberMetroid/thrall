//! Direct transport for a peer that shares the host with the daemon.
//!
//! The daemon calls the peer's REST API straight over the loopback interface,
//! so it never needs SSH access to its own machine. This is the only transport
//! available when the host does not run an SSH server, which is the common case
//! for a peer that THRALL was installed alongside.

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

/// Perform a Syncthing REST call against a peer on this host.
pub fn direct_syncthing_rest(
    api_url: &str,
    api_key: &str,
    method: &str,
    endpoint_path: &str,
    json_body: Option<&str>,
) -> Result<String, String> {
    let base = api_url.trim_end_matches('/');
    let url = format!("{base}{endpoint_path}");
    let header = format!("X-API-Key:{api_key}");
    let mut cmd = Command::new("curl");
    cmd.args(["-fsS", "--max-time", "20", "-X", method, "-H", &header]);
    let Some(body) = json_body else {
        let out = cmd
            .output()
            .map_err(|e| format!("[local] curl exec failed: {e}"))?;
        return finish(out, method, endpoint_path);
    };
    cmd.args(["-H", "Content-Type:application/json", "--data-binary", "@-"]);
    cmd.arg(&url);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("[local] curl spawn failed: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(body.as_bytes())
            .map_err(|e| format!("[local] stdin write failed: {e}"))?;
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("[local] curl wait failed: {e}"))?;
    finish(out, method, endpoint_path)
}

/// Create directories on this host.
pub fn direct_mkdir_many(dirs: &[String]) -> Result<(), String> {
    for d in dirs {
        fs::create_dir_all(d).map_err(|e| format!("[local] mkdir {d} failed: {e}"))?;
    }
    Ok(())
}

fn finish(out: std::process::Output, method: &str, endpoint_path: &str) -> Result<String, String> {
    if !out.status.success() {
        return Err(format!(
            "[local] {method} {endpoint_path} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}
