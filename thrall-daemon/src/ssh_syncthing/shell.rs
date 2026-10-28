//! Shell quoting for values that cross an SSH boundary.
//!
//! `ssh` does not execute its trailing arguments directly. It concatenates them
//! into a single string and hands that string to the remote login shell, which
//! then parses it. A value such as `x; touch /tmp/pwned` travelling in one of
//! those arguments therefore becomes two commands on the remote host, even
//! though the local side never invokes a shell.
//!
//! Every untrusted value passed to `ssh` must be quoted with `quote()` first.

/// Quote a value so the remote shell parses it as a single literal argument.
///
/// Embedded single quotes are escaped with the standard `'\''` sequence: close
/// the quoted string, emit an escaped quote, reopen the string.
pub fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_value_is_wrapped() {
        assert_eq!(
            quote("http://127.0.0.1:8384/rest/db/status"),
            "'http://127.0.0.1:8384/rest/db/status'"
        );
    }

    #[test]
    fn test_shell_metacharacters_stay_inside_the_quotes() {
        let quoted = quote("x; touch /tmp/pwned");
        assert_eq!(quoted, "'x; touch /tmp/pwned'");
        assert!(!quoted.starts_with(';'));
    }

    #[test]
    fn test_embedded_single_quote_is_escaped() {
        assert_eq!(quote("a'b"), r"'a'\''b'");
    }

    #[test]
    fn test_dollar_and_backtick_are_neutralised() {
        assert_eq!(quote("$(id)"), "'$(id)'");
        assert_eq!(quote("`id`"), "'`id`'");
    }
}
