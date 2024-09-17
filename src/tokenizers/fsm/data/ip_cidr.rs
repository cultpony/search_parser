use tracing::trace;

use crate::tokenizers::fsm::{FSMStateMatcher, FSM};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
struct IPv4Address;

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
        trace!(?first_dot, "found first dot");
        if !is_byte(&inp[0..first_dot]) {
            return None
        }
        let inp: &str = &inp[first_dot+1..];
        let second_dot = inp.find('.')?;
        trace!(?second_dot, "found second dot");
        if !is_byte(&inp[0..second_dot]) {
            return None
        }
        let inp: &str = &inp[second_dot+1..];
        let third_dot = inp.find('.')?;
        trace!(?third_dot, "found third dot");
        if !is_byte(&inp[0..second_dot]) {
            return None
        }
        let inp: &str = &inp[third_dot+1..];
        let inpb = inp.as_bytes();
        let partial_pos = first_dot + 1 + second_dot + 1 + third_dot + 1;
        trace!("looking for last digit on termination");
        for (i, c) in inpb.iter().enumerate().take(3.min(inp.len())) {
            if !(*c as char).is_ascii_digit() {
                trace!(?i, "found non-digit");
                if !is_byte(&inp[0..i]) {
                    return None
                }
                let full_pos = partial_pos + i - 1;
                assert!(full_pos < u8::MAX as usize);
                return Some(full_pos as u8)
            }
        }
        trace!("no terminating digit, whole string is valid");
        let i = 3.min(inp.len());
        if !is_byte(&inp[..i]) {
            return None
        }
        let full_pos = partial_pos + i - 1;
        assert!(full_pos < u8::MAX as usize);
        Some(full_pos as u8)
    }

    fn maximum_bound(self) -> Option<u8> {
        Some("255.255.255.255".len() as u8)
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizers::fsm::FSMStateMatcher;

    #[test]
    #[tracing_test::traced_test]
    pub fn test_ipv4_matcher() {
        use super::IPv4Address as a;
        assert_eq!(Some(8), a::matches(a, "127.0.0.1"));
        assert_eq!(Some(10), a::matches(a, "127.0.0.127"));
        assert_eq!(Some(14), a::matches(a, "255.255.255.255"));
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
    }
}