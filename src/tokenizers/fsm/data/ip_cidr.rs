use tracing::trace;

use crate::tokenizers::fsm::{FSMStateMatcher, FSM};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct IPAddress;

impl FSMStateMatcher for IPAddress {
    fn matches(self, inp: &str) -> Option<u8> {
        todo!()
    }

    fn maximum_bound(self) -> Option<u8> {
        IPv4Address::maximum_bound(IPv4Address)
            .max(IPv4Network::maximum_bound(IPv4Network))
    }
}


#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
struct IPv4Address;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
struct IPv4Network;

impl FSMStateMatcher for IPv4Network {
    fn matches(self, inp: &str) -> Option<u8> {
        assert!(inp.len() <= u8::MAX as usize);
        trace!("checking if ip address prefix exists");
        let Some(ip_prefix) = IPv4Address.matches(inp) else {
            return None
        };
        trace!("got an IP, checking for network section");
        let network_section: &str = &inp[((ip_prefix + 1) as usize)..];
        trace!("checking if there is enough characters for network section");
        if network_section.len() < 2 {
            // network section is atleast /0, so two characters are needed, if we have less, abort
            return None
        }
        trace!("checking if next digit is network slash /");
        let mut chars = network_section.chars();
        let Some('/') = chars.next() else {
            trace!("no network slash, not a CIDR");
            return None
        };
        trace!("checking if we have another char");
        let Some(digit1) = chars.next() else {
            trace!("not enough characters for network");
            return None
        };
        trace!("checking if it's a digit");
        if !digit1.is_ascii_digit() {
            trace!(?digit1, "not a digit, aborting");
            return None
        }
        trace!("checking for a potential second digit prefix");
        let Some(digit2) = chars.next() else {
            trace!(?digit1, "two digit network prefix");
            return Some(ip_prefix + 2)
        };
        trace!(?digit1, ?digit2, "got a second digit, need more parsing");
        if digit2.is_ascii_digit() {
            match (digit1, digit2) {
                ('3', '0'..='2') => Some(ip_prefix + 3),
                ('0'..='2', '0'..='9') => Some(ip_prefix + 3),
                _ => None
            }
        } else {
            return Some(ip_prefix + 2)
        }
    }

    fn maximum_bound(self) -> Option<u8> {
        Some("255.255.255.255/32".len() as u8)
    }
}

impl FSMStateMatcher for IPv4Address {
    fn matches(self, inp: &str) -> Option<u8> {
        assert!(inp.len() <= u8::MAX as usize);
        trace!(?inp, "checking if string is ipv4");
        let inp_bound = Self.maximum_bound().unwrap().min(inp.len() as u8);
        let inp_bound = inp_bound as usize;

        /// checks if the given string slice is a valid u8
        fn is_byte(s: &str) -> bool {
            trace!(?s, "checking if valid u8 digit");
            if s.len() > 3 {
                false
            } else {
                let Ok(_): Result<u8, _> = s.parse() else {
                    return false
                };
                true
            }
        }

        let inp: &str = &inp[..inp_bound];
        let first_dot = inp.find('.')?;
        trace!(?first_dot, inp=(&inp[0..first_dot]), "found first dot");
        if !is_byte(&inp[0..first_dot]) {
            return None
        }
        let inp: &str = &inp[first_dot+1..];
        let second_dot = inp.find('.')?;
        trace!(?second_dot, inp=(&inp[0..first_dot]), "found second dot");
        if !is_byte(&inp[0..second_dot]) {
            return None
        }
        let inp: &str = &inp[second_dot+1..];
        let third_dot = inp.find('.')?;
        trace!(?third_dot, inp=(&inp[0..first_dot]), "found third dot");
        if !is_byte(&inp[0..third_dot]) {
            return None
        }
        let inp: &str = &inp[third_dot+1..];
        let inpb = inp.as_bytes();
        let partial_pos = first_dot + 1 + second_dot + 1 + third_dot + 1;
        trace!(?inp, "looking for last digit on termination");
        for (i, c) in inpb.iter().enumerate().take(3.min(inp.len())) {
            if !(*c as char).is_ascii_digit() {
                trace!(?i, "found non-digit");
                if !is_byte(&inp[0..i]) {
                    return None
                }
                let full_pos = partial_pos + i - 1;
                assert!(full_pos < u8::MAX as usize);
                trace!(?full_pos, inp=(&inp[..i]), "complete ip located");
                return Some(full_pos as u8)
            }
            trace!("got digit, checking next one")
        }
        trace!("no terminating digit, whole string is valid");
        let i = 3.min(inp.len());
        if !is_byte(&inp[..i]) {
            trace!("terminating digit bad, aborting parse");
            return None
        }
        let full_pos = partial_pos + i - 1;
        trace!(?full_pos, inp=(&inp[..i]), "complete ip located");
        assert!(full_pos < u8::MAX as usize);
        Some(full_pos as u8)
    }

    fn maximum_bound(self) -> Option<u8> {
        Some("255.255.255.255".len() as u8)
    }
}

#[cfg(test)]
mod test {
    use proptest::test_runner::TestRunner;

    use crate::tokenizers::fsm::FSMStateMatcher;

    #[test]
    #[tracing_test::traced_test]
    pub fn test_ipv4_matcher() {
        use super::IPv4Address as a;
        assert_eq!(Some(8), a::matches(a, "127.0.0.1"));
        assert_eq!(Some(10), a::matches(a, "127.0.0.127"));
        assert_eq!(Some(14), a::matches(a, "255.255.255.255"));
        assert_eq!(Some(10), a::matches(a, "127.0.0.127a"));
        assert_eq!(Some(9), a::matches(a, "127.0.0.12a"));
        assert_eq!(Some(8), a::matches(a, "127.0.0.1a"));
        assert_eq!(None, a::matches(a, "300.0.0.1"));
        assert_eq!(None, a::matches(a, "256.0.0.1"));
        assert_eq!(None, a::matches(a, "127.256.0.1"));
        assert_eq!(None, a::matches(a, "127.255.256.1"));
        assert_eq!(None, a::matches(a, "127.255.255.256"));
        assert_eq!(None, a::matches(a, "127.256.0.1000"));
        assert_eq!(None, a::matches(a, "127.256.1000.1"));
        assert_eq!(None, a::matches(a, "127.2560.100.1"));
        assert_eq!(None, a::matches(a, "a.2.3.4"));
        assert_eq!(None, a::matches(a, "127.255.a.1"));
        assert_eq!(None, a::matches(a, "test"));
        assert_eq!(Some(6), a::matches(a, "0.0.0.0a0"));
        assert_eq!(Some(8), a::matches(a, "0.100.0.0a0"));
    }

    #[test]
    #[tracing_test::traced_test]
    pub fn test_ipv4_cidr_matcher() {
        use super::IPv4Network as a;
        assert_eq!(Some(11), a::matches(a, "127.0.0.1/12"));
        assert_eq!(Some(12), a::matches(a, "127.0.0.127/1"));
        assert_eq!(Some(17), a::matches(a, "255.255.255.255/32"));
        assert_eq!(Some(8), a::matches(a, "0.0.0.0/0"));
        assert_eq!(Some(9), a::matches(a, "0.0.0.0/00"));
        assert_eq!(None, a::matches(a, "127.255.255.1/99"));
    }

    proptest::proptest! {
        #[test]
        fn proptest_ipv4_matcher(d1 in 0..255u8, d2 in 0..255u8, d3 in 0..255u8, d4 in 0..255u8, g in "[a-zA-Z][0-20]") {
            let ip_as_string = format!("{d1}.{d2}.{d3}.{d4}{g}");
            let ip_as_string_len = ip_as_string.len() - g.len();
            // subtract one, the data we get is an index from where the string ends
            let ip_as_string_len = ip_as_string_len - 1;
            let matcher_len = super::IPv4Address.matches(&ip_as_string).unwrap();
            assert_eq!(ip_as_string_len, matcher_len as usize);
        }

        #[test]
        fn proptest_ipv4_network_matcher(d1 in 0..255u8, d2 in 0..255u8, d3 in 0..255u8, d4 in 0..255u8, n in 0..32u8, g in "[a-zA-Z][0-20]") {
            let ip_as_string = format!("{d1}.{d2}.{d3}.{d4}/{n}{g}");
            let ip_as_string_len = ip_as_string.len() - g.len();
            // subtract one, the data we get is an index from where the string ends
            let ip_as_string_len = ip_as_string_len - 1;
            let matcher_len = super::IPv4Network.matches(&ip_as_string).unwrap();
            assert_eq!(ip_as_string_len, matcher_len as usize);
        }
    }
}