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
    use proptest::prelude::*;

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

        macro_rules! test {
         (exact $ip:literal) => {
             assert_eq!(Some($ip.len() as u8), a::matches(a, $ip))
         };
         (extra $ip:literal $extra:literal) => {
             let ip = format!("{}{}", $ip, $extra);
             assert_eq!(Some($ip.len() as u8), a::matches(a, &ip))
         };
         (bad $ip:literal) => {
             assert_eq!(None, a::matches(a, $ip))
         };
        }
        test!(exact "0000:0000:0000:0000:0000:0000:0000:0000");
        test!(exact "0:0:0:0:0:0:0:0");
        test!(exact "0:0:0:0:0:0::0");
        test!(exact "0::0:0:0:0:0:0");
        test!(exact "0:0:0:0:0::0");
        test!(exact "0::0:0:0:0:0");
        test!(exact "0:0:0:0::0");
        test!(exact "0::0:0:0:0");
        test!(exact "0:0:0::0");
        test!(exact "0::0:0:0");
        test!(exact "0:0::0");
        test!(exact "0::0:0");
        test!(exact "0::0");
        //test!(exact "::0");
        test!(exact "0::");
        //test!(exact "::");
     }

    prop_compose! {
         fn arb_ipv6_segment()(id in any::<u16>()) -> String {
             format!("{id:04x}")
         }
     }

    prop_compose! {
         fn arb_trimmed_ipv6_segment()(id in any::<u16>()) -> String {
             format!("{id:x}")
         }
     }

    prop_compose! {
         fn arb_full_ipv6()(
             seg in proptest::collection::vec(arb_ipv6_segment(),8),
         ) -> String {
             seg.join(":")
         }
     }

    prop_compose! {
         fn arb_simple_trim_ipv6()(
             seg in proptest::collection::vec(arb_trimmed_ipv6_segment(),8),
         ) -> String {
             seg.join(":")
         }
     }
 
     proptest! {
         #[test]
         fn proptest_ipv6_matcher(ip in arb_full_ipv6()) {
             let mat = IPv6Address.matches(&ip).unwrap();
            assert_eq!(mat as usize, ip.len())
         }
 
         #[test]
         fn proptest_trimmed_ipv6_matcher(ip in arb_simple_trim_ipv6()) {
             let mat = IPv6Address.matches(&ip).unwrap();
            assert_eq!(mat as usize, ip.len())
         }
     }
 }
