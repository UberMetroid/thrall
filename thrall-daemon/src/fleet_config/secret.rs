//! Session-secret hardening for the THRALL portal.
//!
//! The portal signs session cookies with `portal.session_secret`. If that value is
//! left at the shipped placeholder, anyone who can read the public repository can
//! forge a valid session cookie and impersonate an administrator. This module makes
//! the daemon fail closed:
//!
//! - the shipped placeholder is a hard startup error,
//! - an empty secret is replaced with a freshly generated random value that is
//!   persisted back to the config file so it survives restarts,
//! - a configured secret that is too short is rejected.

use std::fs;
use std::io::Read;

use super::FleetConfig;

/// The placeholder shipped in `assets/default-fleet.yaml`. Public by definition.
pub const PLACEHOLDER_SESSION_SECRET: &str = "change-me";

/// Minimum length enforced for a configured secret.
pub const MIN_SESSION_SECRET_LEN: usize = 16;

/// Ensure `cfg.portal.session_secret` is safe to use, generating one when absent.
///
/// A configured, non-placeholder secret of sufficient length is accepted as-is. The
/// shipped placeholder is rejected outright. An empty secret is replaced with a
/// generated random value that is written back to `path`.
pub fn ensure_session_secret(cfg: &mut FleetConfig, path: &str) -> Result<(), String> {
    let current = cfg.portal.session_secret.trim().to_string();
    if current == PLACEHOLDER_SESSION_SECRET {
        return Err(
            "portal.session_secret is still the 'change-me' placeholder: set a unique secret \
             (portal.session_secret in fleet.yaml, or the PORTAL_SESSION_SECRET environment variable)"
                .to_string(),
        );
    }
    if !current.is_empty() {
        if current.len() < MIN_SESSION_SECRET_LEN {
            return Err(format!(
                "portal.session_secret must be at least {MIN_SESSION_SECRET_LEN} characters (got {})",
                current.len()
            ));
        }
        return Ok(());
    }
    let generated = generate_secret()?;
    cfg.portal.session_secret = generated.clone();
    persist_secret(path, &generated)
}

/// Generate a 256-bit random secret, hex-encoded.
fn generate_secret() -> Result<String, String> {
    let mut buf = [0u8; 32];
    let mut f =
        fs::File::open("/dev/urandom").map_err(|e| format!("cannot open /dev/urandom: {e}"))?;
    f.read_exact(&mut buf)
        .map_err(|e| format!("cannot read /dev/urandom: {e}"))?;
    Ok(buf.iter().map(|b| format!("{b:02x}")).collect())
}

/// Write `secret` into the `session_secret:` entry of the config file at `path`.
///
/// The file is rewritten line-by-line so operator comments and formatting survive;
/// only the `session_secret:` value is replaced.
fn persist_secret(path: &str, secret: &str) -> Result<(), String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let mut out = String::with_capacity(raw.len() + secret.len());
    let mut replaced = false;
    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("session_secret:") {
            let indent = line.len() - trimmed.len();
            out.push_str(&" ".repeat(indent));
            out.push_str(&format!("session_secret: \"{secret}\"\n"));
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !replaced {
        return Err(format!(
            "no 'session_secret:' entry found in {path}: add one under the 'portal:' section"
        ));
    }
    fs::write(path, out).map_err(|e| format!("cannot write {path}: {e}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::fleet_config::{parse_fleet_yaml, FleetConfig};

    fn cfg_with(secret: &str) -> FleetConfig {
        let raw = format!("portal:\n  session_secret: \"{secret}\"\n");
        parse_fleet_yaml(&raw).unwrap()
    }

    #[test]
    fn test_placeholder_is_rejected() {
        let mut cfg = cfg_with(PLACEHOLDER_SESSION_SECRET);
        let err = ensure_session_secret(&mut cfg, "/nonexistent/fleet.yaml").unwrap_err();
        assert!(err.contains("change-me"));
    }

    #[test]
    fn test_short_secret_is_rejected() {
        let mut cfg = cfg_with("tooshort");
        let err = ensure_session_secret(&mut cfg, "/nonexistent/fleet.yaml").unwrap_err();
        assert!(err.contains("at least"));
    }

    #[test]
    fn test_good_secret_is_accepted_untouched() {
        let mut cfg = cfg_with("a-sufficiently-long-secret-value");
        ensure_session_secret(&mut cfg, "/nonexistent/fleet.yaml").unwrap();
        assert_eq!(
            cfg.portal.session_secret,
            "a-sufficiently-long-secret-value"
        );
    }

    #[test]
    fn test_empty_secret_is_generated_and_persisted() {
        let dir = std::env::temp_dir().join("thrall_secret_guard_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("fleet.yaml");
        let _ = fs::write(
            &path,
            "portal:\n  listen_addr: \"0.0.0.0:8080\"\n  session_secret: \"\"\n",
        );
        let mut cfg = parse_fleet_yaml(&fs::read_to_string(&path).unwrap()).unwrap();
        ensure_session_secret(&mut cfg, path.to_str().unwrap()).unwrap();
        assert_eq!(cfg.portal.session_secret.len(), 64);
        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains(&cfg.portal.session_secret));
        assert!(written.contains("listen_addr"));
        let _ = fs::remove_dir_all(&dir);
    }
}
