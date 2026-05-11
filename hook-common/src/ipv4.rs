//! IPv4 dotted-quad parsing.

/// Returns true when `s` is a valid IPv4 address with no
/// leading zeros on multi-digit octets.
pub fn parse_ipv4(s: &[u8]) -> bool {
    let mut count = 0usize;
    for part in s.splitn(5, |&b| b == b'.') {
        if part.is_empty() || part.len() > 3 {
            return false;
        }
        if part.len() > 1 && part[0] == b'0' {
            return false;
        }
        let mut val = 0u16;
        for &b in part {
            if !b.is_ascii_digit() {
                return false;
            }
            val = val * 10 + (b - b'0') as u16;
        }
        if val > 255 {
            return false;
        }
        count += 1;
    }
    count == 4
}

/// Parses "A.B.C.D" into `[A, B, C, D]`.
///
/// The caller must validate with `parse_ipv4` first.
pub fn parse_ipv4_octets(s: &[u8]) -> [u8; 4] {
    debug_assert!(parse_ipv4(s), "parse_ipv4_octets: invalid IPv4");
    let mut octets = [0u8; 4];
    for (idx, part) in s.splitn(4, |&b| b == b'.').enumerate() {
        let mut val = 0u8;
        for &b in part {
            val = val.wrapping_mul(10).wrapping_add(b.wrapping_sub(b'0'));
        }
        octets[idx] = val;
    }
    octets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_valid_addresses() {
        assert!(parse_ipv4(b"0.0.0.0"));
        assert!(parse_ipv4(b"127.0.0.1"));
        assert!(parse_ipv4(b"255.255.255.255"));
        assert!(parse_ipv4(b"192.168.1.10"));
    }

    #[test]
    fn reject_too_many_octets() {
        assert!(!parse_ipv4(b"1.2.3.4.5"));
    }

    #[test]
    fn reject_too_few_octets() {
        assert!(!parse_ipv4(b"1.2.3"));
        assert!(!parse_ipv4(b""));
    }

    #[test]
    fn reject_overflow_octet() {
        assert!(!parse_ipv4(b"256.0.0.0"));
        assert!(!parse_ipv4(b"1.999.0.0"));
    }

    #[test]
    fn reject_leading_zeros() {
        assert!(!parse_ipv4(b"01.0.0.0"));
        assert!(!parse_ipv4(b"0.00.0.0"));
        assert!(!parse_ipv4(b"127.0.0.001"));
    }

    #[test]
    fn reject_empty_octet() {
        assert!(!parse_ipv4(b"1..3.4"));
        assert!(!parse_ipv4(b".1.2.3"));
        assert!(!parse_ipv4(b"1.2.3."));
    }

    #[test]
    fn reject_non_digit() {
        assert!(!parse_ipv4(b"1.2.3.a"));
        assert!(!parse_ipv4(b"1.2.3.4 "));
    }

    #[test]
    fn reject_too_long_octet() {
        assert!(!parse_ipv4(b"1234.5.6.7"));
    }

    #[test]
    fn parse_octets_correctly() {
        assert_eq!(parse_ipv4_octets(b"0.0.0.0"), [0, 0, 0, 0]);
        assert_eq!(parse_ipv4_octets(b"127.0.0.1"), [127, 0, 0, 1]);
        assert_eq!(parse_ipv4_octets(b"255.255.255.255"), [255, 255, 255, 255]);
        assert_eq!(parse_ipv4_octets(b"192.168.1.10"), [192, 168, 1, 10]);
    }
}
