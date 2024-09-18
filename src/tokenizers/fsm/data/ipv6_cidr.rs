use nom::AsChar;
use tracing::trace;

use crate::tokenizers::fsm::FSMStateMatcher;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct IPv6Address;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct IPv6Network;

impl FSMStateMatcher for IPv6Address {
    fn matches(self, inp: &str) -> Option<u8> {
        // limit input to IPv6 address maximum length
        let tinp = &inp[..(41.min(inp.len()))];

        // returns number of characters to next color and the remaining string
        fn next_colon(i: &str) -> Option<(u8, &str)> {
            let n = 5.min(i.len());
            trace!(i=?(i[..n]), "checking for colon");
            i[..n]
                .find(':')
                .filter(|v| {
                    trace!(?v, i=?(i[..(n-1)]), "found colon at position, checking if valid");
                    *v <= n
                })
                .filter(|v| {
                    trace!("checking if str up to is hex");
                    i[..*v].bytes().all(|v| (v as char).is_hex_digit())
                })
                .map(|v| v as u8)
                .map(|v| (v + 1, &i[(v as usize + 1)..]))
        }

        /// parses multiple colon sequences (up to 8) and returns the position, sequence count and remaining string
        fn segment_series(mut i: &str) -> Option<(u8, &str)> {
            let mut seg: u8 = 0;
            for j in 0..8 {
                trace!(?seg, ?i, ?j, "checking segment");
                let segn;
                (segn, i) = match next_colon(i) {
                    None => {
                        trace!("found no colon, looking for end of sequence");
                        let mut out = None;
                        if !i.len() == 0 {
                            for (c, p) in i.bytes().take(4).enumerate() {
                                trace!(?c, ?p, "check if digit");
                                if !(p as char).is_hex_digit() {
                                    trace!(?c, ?p, ?i, inp=?(&i[(c as usize)..]), "no hex digit, closing segment early");
                                    out = Some((c as u8, &i[(c as usize)..]));
                                    break;
                                }
                            }
                        }
                        trace!("completed end of sequence trace");
                        match out {
                            None => (4.min(i.len()) as u8, &i[4.min(i.len())..]),
                            Some(v) => v,
                        }
                    }
                    Some(v) => v,
                };
                trace!(?seg, ?segn, ?i, ?j, "found next segment");
                seg += segn;
            }
            if seg == 0 {
                None
            } else {
                Some((seg, i))
            }
        }

        let trim = inp.find("::");
        let lpos;

        match trim {
            None => {
                trace!(
                    ?inp,
                    ?trim,
                    "uncompressed ipv6, using single segment series expansion"
                );
                (lpos, _) = segment_series(inp)?;
            }
            Some(trim) => {
                trace!(
                    ?inp,
                    ?trim,
                    "compressed ipv6, splitting and looking for expansions"
                );
                let (btrim, atrim) = inp.split_at(trim);
                let (bpos, rem) = segment_series(btrim)?;
                if rem.len() != 0 {
                    trace!(?btrim, ?rem, "remainder in before-trim segment, not IPv6");
                    return None;
                }
                let (apos, rem) = segment_series(atrim)?;
                if rem.len() != 0 {
                    trace!(?btrim, ?rem, "remainder in after-trim segment, not IPv6");
                    return None;
                }
                lpos = bpos + apos;
            }
        }

        trace!(?lpos, "segment termination found");
        if lpos == u8::MAX {
            return None;
        }

        let res = lpos - 1;
        trace!(?res, inp=?tinp[..(lpos as usize)], "found ip address");
        Some(res)
    }
}

impl FSMStateMatcher for IPv6Network {
    fn matches(self, inp: &str) -> Option<u8> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use proptest::{prelude::*, prop_compose, proptest};

    use super::*;
    use crate::tokenizers::fsm::FSMStateMatcher;

    #[test]
    #[tracing_test::traced_test]
    fn test_basic_ipv6() {
        use super::IPv6Address as a;
        macro_rules! test {
            (exact $ip:literal) => {
                assert_eq!(Some($ip.len() as u8 - 1), a::matches(a, $ip))
            };
            (extra $ip:literal $extra:literal) => {
                let ip = format!("{}{}", $ip, $extra);
                assert_eq!(Some($ip.len() as u8 - 1), a::matches(a, &ip))
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
        test!(exact "::0");
        test!(exact "0::");
        test!(exact "::");
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
