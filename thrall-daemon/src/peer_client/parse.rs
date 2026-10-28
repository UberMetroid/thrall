//! Minimal extractors for the Syncthing REST API responses.
//!
//! Kept separate from the transport so the transport module can focus on
//! making the call and reading the reply.

/// Extract a string field from a JSON object.
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

/// Extract an unsigned integer field from a JSON object.
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

/// Extract a boolean field from a JSON object.
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_json_extractors() {
        let status_json = r#"{"myID": "T4U7YLJ-I6PFVWY", "uptime": 42, "connected": true}"#;
        assert_eq!(
            parse_json_string_field(status_json, "myID").as_deref(),
            Some("T4U7YLJ-I6PFVWY")
        );
        assert_eq!(parse_json_u64_field(status_json, "uptime"), Some(42));
        assert_eq!(parse_json_bool_field(status_json, "connected"), Some(true));
    }
}
