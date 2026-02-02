//! Voucher code generation and validation utilities.
//!
//! Generates human-friendly voucher codes that are easy to type on a phone.
//! Format: HIVE-XXXX-XXXX (uppercase alphanumeric, no ambiguous characters).

use rand::Rng;

/// Characters used in voucher codes.
/// Excludes 0/O, 1/I/L to avoid confusion when typing on a phone.
const CHARSET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// Generate a new voucher code in format HIVE-XXXX-XXXX.
pub fn generate_voucher_code() -> String {
    let mut rng = rand::rng();
    let part1: String = (0..4)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    let part2: String = (0..4)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    format!("HIVE-{}-{}", part1, part2)
}

/// Generate a shorter voucher code (6 chars) for simpler use cases.
pub fn generate_short_code() -> String {
    let mut rng = rand::rng();
    let code: String = (0..6)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    code
}

/// Validate voucher code format (doesn't check database).
pub fn is_valid_format(code: &str) -> bool {
    let code = code.trim().to_uppercase();

    // HIVE-XXXX-XXXX format
    if code.len() == 14 && code.starts_with("HIVE-") {
        let parts: Vec<&str> = code.split('-').collect();
        return parts.len() == 3
            && parts[1].len() == 4
            && parts[2].len() == 4
            && parts[1].chars().all(|c| CHARSET.contains(&(c as u8)))
            && parts[2].chars().all(|c| CHARSET.contains(&(c as u8)));
    }

    // Short 6-char format
    if code.len() == 6 {
        return code.chars().all(|c| CHARSET.contains(&(c as u8)));
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_voucher_code_format() {
        let code = generate_voucher_code();
        assert!(code.starts_with("HIVE-"));
        assert_eq!(code.len(), 14); // HIVE-XXXX-XXXX
        assert!(is_valid_format(&code));
    }

    #[test]
    fn test_generate_short_code() {
        let code = generate_short_code();
        assert_eq!(code.len(), 6);
        assert!(is_valid_format(&code));
    }

    #[test]
    fn test_codes_are_unique() {
        let codes: Vec<String> = (0..100).map(|_| generate_voucher_code()).collect();
        let unique: std::collections::HashSet<&String> = codes.iter().collect();
        // With 30^8 possible codes, collisions in 100 should be essentially impossible
        assert_eq!(codes.len(), unique.len());
    }

    #[test]
    fn test_no_ambiguous_chars() {
        for _ in 0..100 {
            let code = generate_voucher_code();
            let chars: String = code.replace("HIVE-", "").replace('-', "");
            assert!(!chars.contains('0'));
            assert!(!chars.contains('O'));
            assert!(!chars.contains('1'));
            assert!(!chars.contains('I'));
            assert!(!chars.contains('L'));
        }
    }

    #[test]
    fn test_invalid_formats() {
        assert!(!is_valid_format(""));
        assert!(!is_valid_format("abc"));
        assert!(!is_valid_format("HIVE-"));
        assert!(!is_valid_format("HIVE-AAAA"));
        assert!(!is_valid_format("HIVE-0000-AAAA")); // 0 is excluded
    }
}
