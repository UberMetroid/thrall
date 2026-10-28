//! Syncthing 56-character Device ID normalization and Base32 Luhn mod 32
//! check-digit verification.
//!
//! A Syncthing device ID is 56 Base32 characters, grouped as four blocks of
//! fourteen: thirteen data characters followed by one Luhn mod 32 check
//! character. Thrall accepts an ID only when every check digit verifies, so a
//! mistyped device ID is rejected at enrollment rather than silently producing
//! a device that no peer will trust.

const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

/// Compute the Luhn mod 32 check character for a 13-character Base32 block.
fn luhn_base32_check_char(chunk13: &[u8]) -> Option<u8> {
    if chunk13.len() != 13 {
        return None;
    }
    let (mut factor, mut sum) = (1u32, 0u32);
    for &b in chunk13 {
        let v = BASE32_ALPHABET.iter().position(|&c| c == b)? as u32;
        let addend = factor * v;
        factor = if factor == 2 { 1 } else { 2 };
        sum += (addend / 32) + (addend % 32);
    }
    BASE32_ALPHABET
        .get(((32 - (sum % 32)) % 32) as usize)
        .copied()
}

/// Normalize a device ID and verify its four Luhn check digits.
///
/// The input may carry whitespace, separators, or lower-case letters; the
/// result is the canonical upper-case, dash-grouped form.
pub fn normalize_and_verify_device_id(raw: &str) -> Result<String, String> {
    let cleaned: Vec<u8> = raw
        .bytes()
        .map(|b| b.to_ascii_uppercase())
        .filter(|b| b.is_ascii_alphanumeric())
        .collect();
    if cleaned.len() < 56 {
        return Err("Device ID must contain 56 Base32 chars".to_string());
    }
    for win in cleaned.windows(56) {
        let valid = (0..4).all(|g| {
            let grp = &win[g * 14..(g + 1) * 14];
            luhn_base32_check_char(&grp[0..13]) == Some(grp[13])
        });
        if valid {
            let grouped: Vec<String> = win
                .chunks(7)
                .map(|c| String::from_utf8_lossy(c).into())
                .collect();
            return Ok(grouped.join("-"));
        }
    }
    Err("Invalid Syncthing Device ID or Luhn check digit".to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_normalizes_and_verifies_a_valid_device_id() {
        let messy = "  device_id: aaaaaaa aaaaaaa\u{200b}-bbbbbbb_bbbbbbn ccccccc-cccccc2-ddddddd-ddddddh  ";
        assert_eq!(
            normalize_and_verify_device_id(messy).unwrap(),
            "AAAAAAA-AAAAAAA-BBBBBBB-BBBBBBN-CCCCCCC-CCCCCC2-DDDDDDD-DDDDDDH"
        );
    }

    #[test]
    fn test_rejects_short_and_mischecked_ids() {
        assert!(normalize_and_verify_device_id("too-short").is_err());
        let bad = "AAAAAAA-AAAAAAA-BBBBBBB-BBBBBBN-CCCCCCC-CCCCCC2-DDDDDDD-DDDDDDI";
        assert!(normalize_and_verify_device_id(bad).is_err());
    }
}
