//! URL encoding for values placed into Syncthing REST query strings.

/// Percent-encode a value for use as a URL query-string component.
///
/// Everything outside the RFC 3986 unreserved set is escaped. The result can
/// therefore never introduce a query separator (`&`, `#`) into the URL, and it
/// never carries a character that a shell would treat specially.
pub fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for &b in value.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_unreserved_characters_pass_through() {
        assert_eq!(percent_encode("Documents-2"), "Documents-2");
        assert_eq!(percent_encode("a_b.c~d"), "a_b.c~d");
    }

    #[test]
    fn test_query_separators_are_escaped() {
        assert_eq!(percent_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(percent_encode("a#b"), "a%23b");
    }

    #[test]
    fn test_shell_metacharacters_are_escaped() {
        assert_eq!(
            percent_encode("x;touch /tmp/pwned"),
            "x%3Btouch%20%2Ftmp%2Fpwned"
        );
        assert_eq!(percent_encode("$(id)"), "%24%28id%29");
    }

    #[test]
    fn test_spaces_are_escaped() {
        assert_eq!(percent_encode("My Folder"), "My%20Folder");
    }
}
