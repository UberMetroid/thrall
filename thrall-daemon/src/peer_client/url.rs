//! Address handling for a peer's REST API.
//!
//! A peer's API base URL is the only thing that differs between a peer on
//! this host and a peer on another host, so the rules about what a usable
//! address looks like live here.

/// Build the full URL for one REST endpoint on a peer.
pub fn build_url(base: &str, endpoint_path: &str) -> String {
    format!("{}{}", base.trim_end_matches('/'), endpoint_path)
}

/// Extract the host part of a URL.
///
/// The authority is stripped of any credentials, port, path, query, and
/// fragment, and the result is lowercased so comparisons are stable.
pub fn host_of(url: &str) -> String {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    let authority = after_scheme.split(['/', '?', '#']).next().unwrap_or("");
    let without_creds = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    let host = if let Some(rest) = without_creds.strip_prefix('[') {
        rest.split(']').next().unwrap_or("")
    } else {
        without_creds.split(':').next().unwrap_or("")
    };
    host.trim_matches(['[', ']']).to_ascii_lowercase()
}

/// True when the URL points at the loopback interface.
///
/// A peer that is not on this host must not be reached over loopback: the
/// only thing listening there is the local peer, so the call would silently
/// land on the wrong machine.
pub fn is_loopback(url: &str) -> bool {
    matches!(
        host_of(url).as_str(),
        "127.0.0.1" | "localhost" | "::1" | "0:0:0:0:0:0:0:1"
    )
}

/// Validate a peer API URL.
///
/// Only `http` and `https` are accepted, and characters that could change
/// the meaning of the request are refused outright.
pub fn validate_api_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("peer API URL is empty".to_string());
    }
    let Some((scheme, _)) = trimmed.split_once("://") else {
        return Err(format!(
            "peer API URL '{trimmed}' has no scheme: use http:// or https://"
        ));
    };
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(format!(
            "peer API URL '{trimmed}' has unsupported scheme '{scheme}': use http or https"
        ));
    }
    if host_of(trimmed).is_empty() {
        return Err(format!("peer API URL '{trimmed}' has no host"));
    }
    if trimmed.chars().any(|c| {
        c.is_whitespace()
            || matches!(
                c,
                '"' | '\'' | '`' | '$' | ';' | '|' | '&' | '<' | '>' | '\\' | '(' | ')' | '{' | '}'
            )
    }) {
        return Err(format!(
            "peer API URL '{trimmed}' contains characters that are not allowed in an address"
        ));
    }
    Ok(())
}

/// Validate a peer API key.
///
/// The key is a credential, so it must be present and free of whitespace.
pub fn validate_api_key(key: &str) -> Result<(), String> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err("peer API key is empty".to_string());
    }
    if trimmed.len() < 16 {
        return Err("peer API key is too short to be a real credential".to_string());
    }
    if trimmed.chars().any(char::is_whitespace) {
        return Err("peer API key contains whitespace".to_string());
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_host_of_strips_port_path_and_case() {
        assert_eq!(host_of("http://100.64.0.1:8384"), "100.64.0.1");
        assert_eq!(
            host_of("http://Hub-1.tailnet.ts.net:8384/rest"),
            "hub-1.tailnet.ts.net"
        );
        assert_eq!(host_of("https://user:pw@hub-2:8384/x"), "hub-2");
        assert_eq!(host_of("http://[::1]:8384"), "::1");
    }

    #[test]
    fn test_loopback_detection() {
        assert!(is_loopback("http://127.0.0.1:8384"));
        assert!(is_loopback("http://localhost:8384"));
        assert!(is_loopback("http://[::1]:8384"));
        assert!(!is_loopback("http://100.64.0.1:8384"));
        assert!(!is_loopback("http://hub.tailnet.ts.net:8384"));
    }

    #[test]
    fn test_url_validation_accepts_plain_addresses() {
        assert!(validate_api_url("http://100.64.0.1:8384").is_ok());
        assert!(validate_api_url("https://hub.tailnet.ts.net:8384").is_ok());
    }

    #[test]
    fn test_url_validation_refuses_anything_else() {
        assert!(validate_api_url("").is_err());
        assert!(validate_api_url("100.64.0.1:8384").is_err());
        assert!(validate_api_url("ftp://100.64.0.1:8384").is_err());
        assert!(validate_api_url("http://").is_err());
        assert!(validate_api_url("http://a b:8384").is_err());
        assert!(validate_api_url("http://a$(id):8384").is_err());
    }

    #[test]
    fn test_api_key_validation() {
        assert!(validate_api_key("0123456789abcdef0123").is_ok());
        assert!(validate_api_key("").is_err());
        assert!(validate_api_key("short").is_err());
        assert!(validate_api_key("0123456789abcde f").is_err());
    }
}
