use crate::tokenizers::fsm::FSMStateMatcher;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct IPv6Address;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct IPv6Network;

impl FSMStateMatcher for IPv6Address {
    fn matches(self, inp: &str) -> Option<u8> {
        const char_set: &'static str = "0123456789abcdefABCDEF:";
        const max_ipv6: usize = "2001:0db8:85a3:08d3:1319:8a2e:0370:7344".len();
        const min_ipv6: usize = "::".len();
        if inp.len() < min_ipv6 {
            return None
        }
        for (i, c) in inp.chars().take(max_ipv6).enumerate() {
            if char_set.contains(c) {
                continue
            } else {
                if i < min_ipv6 {
                    return None
                }
                return Some(i as u8 - 1u8)
            }
        }
        Some(inp.len().min(0xFF) as u8)
    }
}

impl FSMStateMatcher for IPv6Network {
    fn matches(self, inp: &str) -> Option<u8> {
        const char_set: &'static str = "0123456789abcdefABCDEF:/";
        const max_ipv6: usize = "2001:0db8:85a3:08d3:1319:8a2e:0370:7344/127".len();
        const min_ipv6: usize = "::/1".len();
        if inp.len() < min_ipv6 {
            return None
        }
        for (i, c) in inp.chars().take(max_ipv6).enumerate() {
            if char_set.contains(c) {
                continue
            } else {
                if i < min_ipv6 {
                    return None
                }
                return Some(i as u8 - 1u8)
            }
        }

        Some(inp.len().min(0xFF) as u8)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    #[tracing_test::traced_test]
    pub fn test_ipv6_cidr_matcher() {
        use super::IPv6Network as a;
        assert_eq!(Some(42), a::matches(a, "2001:0db8:85a3:08d3:1319:8a2e:0370:7344/12"));
        assert_eq!(Some(4), a::matches(a, "::/1"));
        assert_eq!(Some(28), a::matches(a, "2001:0db8:85a3::0370:7344/32"));
        assert_eq!(Some(12), a::matches(a, "0000::0000/0"));
    }

    #[test]
    #[tracing_test::traced_test]
    pub fn test_ipv6_addr_matcher() {
        use super::IPv6Address as a;
        assert_eq!(Some(39), a::matches(a, "2001:0db8:85a3:08d3:1319:8a2e:0370:7344"));
        assert_eq!(Some(2), a::matches(a, "::"));
        assert_eq!(Some(25), a::matches(a, "2001:0db8:85a3::0370:7344"));
        assert_eq!(Some(10), a::matches(a, "0000::0000"));
    }
}