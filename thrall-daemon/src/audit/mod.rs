//! The append-only administrative audit log.
//!
//! Every administrative action is appended to `audit.log`, a file only the
//! owner can read that lives beside the roster. The log is a plain
//! tab-separated file so it can be read with any tool on the host, and it is
//! trimmed to a fixed number of records so it cannot grow without bound.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

/// The number of records kept before the oldest are dropped.
pub const MAX_RECORDS: usize = 500;

/// One recorded administrative action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditEntry {
    pub ts: u64,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub outcome: String,
}

/// Make a field safe to store in a tab-separated record.
fn sanitize(field: &str, max: usize) -> String {
    let cleaned: String = field
        .chars()
        .map(|c| {
            if c == '\t' || c == '\n' || c == '\r' {
                ' '
            } else {
                c
            }
        })
        .collect();
    cleaned.trim().chars().take(max).collect()
}

/// Render one entry as a stored record.
fn encode(e: &AuditEntry) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\n",
        e.ts,
        sanitize(&e.actor, 64),
        sanitize(&e.action, 64),
        sanitize(&e.target, 256),
        sanitize(&e.outcome, 128)
    )
}

/// Parse one stored record.
fn decode(line: &str) -> Option<AuditEntry> {
    let mut parts = line.split('\t');
    Some(AuditEntry {
        ts: parts.next()?.trim().parse().ok()?,
        actor: parts.next()?.to_string(),
        action: parts.next()?.to_string(),
        target: parts.next()?.to_string(),
        outcome: parts.next().unwrap_or("ok").to_string(),
    })
}

/// Append an entry, then trim the log back to [`MAX_RECORDS`].
pub fn append(path: &str, ts: u64, actor: &str, action: &str, target: &str, outcome: &str) {
    let entry = AuditEntry {
        ts,
        actor: actor.to_string(),
        action: action.to_string(),
        target: target.to_string(),
        outcome: outcome.to_string(),
    };
    let Ok(mut file) = OpenOptions::new()
        .append(true)
        .create(true)
        .mode(0o600)
        .open(path)
    else {
        return;
    };
    let _ = file.set_permissions(fs::Permissions::from_mode(0o600));
    let _ = file.write_all(encode(&entry).as_bytes());
    trim(path);
}

/// Drop the oldest records once the log grows past [`MAX_RECORDS`].
fn trim(path: &str) {
    let Ok(text) = fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= MAX_RECORDS {
        return;
    }
    let kept = lines[lines.len() - MAX_RECORDS..].join("\n");
    let _ = crate::fleet_config::secret::write_private_file(path, &format!("{kept}\n"));
}

/// The most recent entries, newest last.
pub fn recent(path: &str, limit: usize) -> Vec<AuditEntry> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut out: Vec<AuditEntry> = text.lines().filter_map(decode).collect();
    if out.len() > limit {
        out.drain(..out.len() - limit);
    }
    out
}

/// The current wall-clock time in seconds since the epoch.
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Where the audit log lives, next to the roster it describes.
pub fn path_for(database_path: &str) -> String {
    match database_path.rfind('/') {
        Some(i) => format!("{}/audit.log", &database_path[..i]),
        None => "audit.log".to_string(),
    }
}

/// Record an administrative action against a specific log.
pub fn record_at(path: &str, actor: &str, action: &str, target: &str, outcome: &str) {
    append(path, now(), actor, action, target, outcome);
}

/// A short human-readable age for a timestamp.
pub fn age(ts: u64, now: u64) -> String {
    let delta = now.saturating_sub(ts);
    match delta {
        0..=59 => format!("{delta}s ago"),
        60..=3599 => format!("{}m ago", delta / 60),
        3600..=86_399 => format!("{}h ago", delta / 3600),
        _ => format!("{}d ago", delta / 86_400),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("thrall_audit_{name}"));
        let _ = fs::create_dir_all(&dir);
        dir.join("audit.log").to_string_lossy().to_string()
    }

    #[test]
    fn test_append_then_read_back() {
        let path = temp_path("basic");
        let _ = fs::remove_file(&path);
        append(&path, 100, "tester", "ui.appearance", "accent=coral", "ok");
        append(&path, 200, "tester", "ops.maintenance", "mode=on", "ok");
        let got = recent(&path, 10);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].action, "ui.appearance");
        assert_eq!(got[1].ts, 200);
        let _ = fs::remove_dir_all(std::path::Path::new(&path).parent().unwrap());
    }

    #[test]
    fn test_fields_cannot_break_the_record_format() {
        let path = temp_path("sanitize");
        let _ = fs::remove_file(&path);
        append(&path, 1, "bad\tactor", "act\nion", "tar\tget", "ok");
        let got = recent(&path, 10);
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].actor, "bad actor");
        assert_eq!(got[0].action, "act ion");
        assert_eq!(got[0].target, "tar get");
        let _ = fs::remove_dir_all(std::path::Path::new(&path).parent().unwrap());
    }

    #[test]
    fn test_the_log_is_owner_only() {
        let path = temp_path("mode");
        let _ = fs::remove_file(&path);
        append(&path, 1, "tester", "act", "target", "ok");
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        let _ = fs::remove_dir_all(std::path::Path::new(&path).parent().unwrap());
    }
}
