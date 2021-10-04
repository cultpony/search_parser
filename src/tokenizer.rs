use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{anychar, none_of, space0, space1},
    combinator::{eof, map_res, opt, peek, recognize},
    error::{Error, ErrorKind, ParseError},
    multi::{count, many_till},
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

#[derive(Clone, Debug)]
pub enum SearchToken<'a> {
    /// `AND`, `&&`, `,`
    And,
    /// `OR`, `||`
    Or,
    /// `NOT`, `!`, `-`
    Not,
    /// Left parenthesis (`(`)
    Lparen,
    /// Right parenthesis (`)`)
    Rparen,
    /// Caret (`^`)
    Boost,
    /// Tilde (`~`)
    Fuzz,
    /// Quotation mark (`"`)
    Quote,
    /// Real number literal (`1234.5678`)
    Float,
    /// Decimal literal (`1234`)
    Integer,
    /// Boolean literal (`true`, `false`)
    Boolean,
    /// IP address in optional CIDR notation
    IpCidr,
    /// 4-digit decimal year
    AbsoluteDate4Digit,
    /// 2-digit decimal day, month, hour, minute, or second
    AbsoluteDate2Digit,
    /// Hyphen (`-`)
    AbsoluteDateHyphen,
    /// RFC3339 date/time separator (`T`, space ` `)
    AbsoluteDateTimeSep,
    /// Colon (`:`)
    AbsoluteDateColon,
    /// Plus or minus (`+`, `-`)
    AbsoluteDateOffsetDirection,
    /// Z (`Z`)
    AbsoluteDateZulu,
    /// `seconds`, `minutes`, `hours`, `days`, `weeks`, `months`, `years`
    /// `second`, `minute`, `hour`, `day`, `week`, `month`, `year,`
    RelativeDateMultiplier,
    /// `ago`, `from now`
    RelativeDateDirection,
    /// A field name
    Field(&'a str),
    /// `.lte:`
    RangeLte,
    /// `.lt:`
    RangeLt,
    /// `.gte:`
    RangeGte,
    /// `.gt:
    RangeGt,
    /// `:`
    RangeEq,
    /// Empty string
    Eof,
    /// An unquoted term (see [term])
    Term,
    /// A quoted term (see [quoted_term])
    QuotedTerm,
}

/// Returns whether the character is a decimal digit (0123456789).
fn is_dec_digit(c: char) -> bool {
    ('0'..='9').contains(&c)
}

/// Returns whether the character is a hexadecimal digit
/// (0123456789abcdefABCDEF).
fn is_hex_digit(c: char) -> bool {
    ('0'..='9').contains(&c) || ('a'..='f').contains(&c) || ('A'..='F').contains(&c)
}

/// Returns a parser which recognizes an integer having at least `min`
/// characters and at most `max` characters.
fn limited_integer(min: usize, max: usize) -> impl Fn(&str) -> IResult<&str, &str> {
    move |input| recognize(take_while_m_n(min, max, is_dec_digit))(input)
}

/// Recognizes a signed decimal integer with at most 32 digits.
fn decimal_integer(input: &str) -> IResult<&str, &str> {
    recognize(preceded(
        opt(alt((tag("+"), tag("-")))),
        limited_integer(1, 32),
    ))(input)
}

/// Recognizes a signed real number with at most 64 digits of precision.
fn float(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        decimal_integer,
        opt(preceded(tag("."), opt(limited_integer(1, 32)))),
    ))(input)
}

/// Recognizes an IPv4 address, like `1.2.3.4`.
fn ipv4_addr(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        limited_integer(1, 3),
        tag("."),
        limited_integer(1, 3),
        tag("."),
        limited_integer(1, 3),
        tag("."),
        limited_integer(1, 3),
    )))(input)
}

/// Recognizes an IPv4 CIDR prefix, like `/12`.
fn ipv4_prefix(input: &str) -> IResult<&str, &str> {
    recognize(pair(tag("/"), limited_integer(1, 2)))(input)
}

/// Recognizes a 2-byte segment of an IPv6 address, like `abcd`.
fn ipv6_hexadectet(input: &str) -> IResult<&str, &str> {
    recognize(take_while_m_n(1, 4, is_hex_digit))(input)
}

/// Recognizes the least significant 32 bits of an IPv6 address.
///
/// This could be two hexadectets, like `abcd:dcba`, or an IPv4 address,
/// like `170.170.170.170`.
fn ipv6_ls32(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(tuple((ipv6_hexadectet, tag(":"), ipv6_hexadectet))),
        ipv4_addr,
    ))(input)
}

/// Recognizes a "fragment" of an IPv6 address, e.g. a hexadectet followed
/// by a colon (`aaaa:`).
fn ipv6_fragment(input: &str) -> IResult<&str, &str> {
    recognize(pair(ipv6_hexadectet, tag(":")))(input)
}

/// Recognizes an IPv6 address, in expanded or shortened form.
fn ipv6_addr(input: &str) -> IResult<&str, &str> {
    // Many cases, ordered by decreasing number of sections after :: shortening
    alt((
        alt((
            // 8 sections
            recognize(tuple((count(ipv6_fragment, 6), ipv6_ls32))),
            // 7 sections
            recognize(tuple((tag("::"), count(ipv6_fragment, 5), ipv6_ls32))),
            recognize(tuple((
                ipv6_hexadectet,
                tag("::"),
                count(ipv6_fragment, 4),
                ipv6_ls32,
            ))),
            // 6 sections
            recognize(tuple((tag("::"), count(ipv6_fragment, 4), ipv6_ls32))),
            recognize(tuple((
                count(ipv6_fragment, 1),
                ipv6_hexadectet,
                tag("::"),
                count(ipv6_fragment, 3),
                ipv6_ls32,
            ))),
            recognize(tuple((
                ipv6_hexadectet,
                tag("::"),
                count(ipv6_fragment, 3),
                ipv6_ls32,
            ))),
            // 5 sections
            recognize(tuple((tag("::"), count(ipv6_fragment, 3), ipv6_ls32))),
            recognize(tuple((
                count(ipv6_fragment, 2),
                ipv6_hexadectet,
                tag("::"),
                count(ipv6_fragment, 2),
                ipv6_ls32,
            ))),
            recognize(tuple((
                count(ipv6_fragment, 1),
                ipv6_hexadectet,
                tag("::"),
                count(ipv6_fragment, 2),
                ipv6_ls32,
            ))),
            recognize(tuple((
                ipv6_hexadectet,
                tag("::"),
                count(ipv6_fragment, 2),
                ipv6_ls32,
            ))),
            // 4 sections
            recognize(tuple((tag("::"), count(ipv6_fragment, 2), ipv6_ls32))),
            recognize(tuple((
                count(ipv6_fragment, 3),
                ipv6_hexadectet,
                tag("::"),
                ipv6_fragment,
                ipv6_ls32,
            ))),
            recognize(tuple((
                count(ipv6_fragment, 2),
                ipv6_hexadectet,
                tag("::"),
                ipv6_fragment,
                ipv6_ls32,
            ))),
            recognize(tuple((
                count(ipv6_fragment, 1),
                ipv6_hexadectet,
                tag("::"),
                ipv6_fragment,
                ipv6_ls32,
            ))),
            recognize(tuple((
                ipv6_hexadectet,
                tag("::"),
                ipv6_fragment,
                ipv6_ls32,
            ))),
        )),
        alt((
            // 3 sections
            recognize(tuple((tag("::"), ipv6_fragment, ipv6_ls32))),
            recognize(tuple((
                count(ipv6_fragment, 4),
                ipv6_hexadectet,
                tag("::"),
                ipv6_ls32,
            ))),
            recognize(tuple((
                count(ipv6_fragment, 3),
                ipv6_hexadectet,
                tag("::"),
                ipv6_ls32,
            ))),
            recognize(tuple((
                count(ipv6_fragment, 2),
                ipv6_hexadectet,
                tag("::"),
                ipv6_ls32,
            ))),
            recognize(tuple((
                count(ipv6_fragment, 1),
                ipv6_hexadectet,
                tag("::"),
                ipv6_ls32,
            ))),
            recognize(tuple((ipv6_hexadectet, tag("::"), ipv6_ls32))),
            // 2 sections
            recognize(tuple((tag("::"), ipv6_ls32))),
            recognize(tuple((
                count(ipv6_fragment, 5),
                ipv6_hexadectet,
                tag("::"),
                ipv6_hexadectet,
            ))),
            recognize(tuple((
                count(ipv6_fragment, 4),
                ipv6_hexadectet,
                tag("::"),
                ipv6_hexadectet,
            ))),
            recognize(tuple((
                count(ipv6_fragment, 3),
                ipv6_hexadectet,
                tag("::"),
                ipv6_hexadectet,
            ))),
            recognize(tuple((
                count(ipv6_fragment, 2),
                ipv6_hexadectet,
                tag("::"),
                ipv6_hexadectet,
            ))),
            recognize(tuple((
                count(ipv6_fragment, 1),
                ipv6_hexadectet,
                tag("::"),
                ipv6_hexadectet,
            ))),
            recognize(tuple((ipv6_hexadectet, tag("::"), ipv6_hexadectet))),
            // 1 section
            recognize(tuple((tag("::"), ipv6_hexadectet))),
            recognize(tuple((count(ipv6_fragment, 6), ipv6_hexadectet, tag("::")))),
            recognize(tuple((count(ipv6_fragment, 5), ipv6_hexadectet, tag("::")))),
            recognize(tuple((count(ipv6_fragment, 4), ipv6_hexadectet, tag("::")))),
            recognize(tuple((count(ipv6_fragment, 3), ipv6_hexadectet, tag("::")))),
            recognize(tuple((count(ipv6_fragment, 2), ipv6_hexadectet, tag("::")))),
            recognize(tuple((count(ipv6_fragment, 1), ipv6_hexadectet, tag("::")))),
            recognize(tuple((ipv6_hexadectet, tag("::")))),
        )),
        // 0 sections
        tag("::"),
    ))(input)
}

/// Recognizes an IPv6 CIDR prefix, like `/123`.
fn ipv6_prefix(input: &str) -> IResult<&str, &str> {
    recognize(pair(tag("/"), limited_integer(1, 3)))(input)
}

/// Recognizes a fully specified IP address, like `127.0.0.1` or
/// `2200:dead:beef::cafe`, or an IP prefix specified in CIDR notation,
/// like `1.2.3.4/24` or `2200::dead:beef::/64`.
fn ip_cidr(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(tuple((ipv4_addr, opt(ipv4_prefix)))),
        recognize(tuple((ipv6_addr, opt(ipv6_prefix)))),
    ))(input)
}

/// Recognizes a conjunction operator, like `x AND y`, `x && y`, or `x, y`.
fn and(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(space1, alt((tag("AND"), tag("&&"))), space1),
        delimited(space0, tag(","), space0),
    ))(input)
}

/// Recognizes a disjunction operator, like `x OR y` or `x || y`.
fn or(input: &str) -> IResult<&str, &str> {
    delimited(space1, alt((tag("OR"), tag("||"))), space1)(input)
}

/// Recognizes a negation operator, like `!x`, `-x`, or `NOT x`.
fn not(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(space0, alt((tag("!"), tag("-"))), space0),
        delimited(space0, tag("NOT"), space1),
    ))(input)
}

/// Recognizes the stop words which delimit unquoted terms. These are:
/// - conjunction (`x,`, `x AND `, `x && `)
/// - disjunction (`x || `, `x OR `)
/// - closing parenthesis (`x)`)
/// - end of input
fn stop_words(input: &str) -> IResult<&str, &str> {
    alt((and, or, preceded(space0, tag(")")), eof))(input)
}

/// Recognizes the full sequence of tokens which delimit unquoted terms.
///
/// This includes everything from `stop_words`, as well as an additional rule
/// recognizing caret expressions (`^1234)`). This allows tokenization to avoid
/// consuming a boost rule as part of a term.
fn term_split(input: &str) -> IResult<&str, &str> {
    alt((
        stop_words,
        recognize(delimited(tag("^"), float, stop_words)),
    ))(input)
}

/// Recognizes a character which has been escaped (preceded by a backslash).
///
/// Escaped characters are included verbatim in the term and not considered
/// for further tokenization.
fn escaped_char(input: &str) -> IResult<&str, &str> {
    recognize(pair(tag("\\"), anychar))(input)
}

/// Recognizes a legal subexpression within an unquoted term.
///
/// An unquoted term like `rose (flower)` contains a parenthesized
/// subexpression `(flower)`. No recursion is used to recognize this,
/// only one level of parentheses may be used.
fn subexpression(input: &str) -> IResult<&str, &str> {
    recognize(preceded(
        tag("("),
        many_till(alt((escaped_char, recognize(none_of("()")))), tag(")")),
    ))(input)
}

/// Recognizes any non-empty string.
fn non_empty(input: &str) -> Result<&str, nom::Err<&str>> {
    if input.is_empty() {
        Err(nom::Err::Error(""))
    } else {
        Ok(input)
    }
}

/// Recognizes an unquoted term.
///
/// An unquoted term may be a single word, like `rose`, a word with any
/// number of subexpressions, like `rose (flower)`, or a word with any
/// number of escaped characters, like `rose \(flower\)`.
fn term(input: &str) -> IResult<&str, &str> {
    map_res(
        recognize(many_till(
            alt((subexpression, escaped_char, recognize(anychar))),
            peek(term_split),
        )),
        non_empty,
    )(input)
}

/// Recognizes a quoted term.
///
/// A quoted term consists of any number of escaped or unescaped characters
/// and terminates when an unescaped quote is found, like `"rose (flower)"`.
fn quoted_term(input: &str) -> IResult<&str, &str> {
    map_res(
        recognize(many_till(
            alt((escaped_char, recognize(anychar))),
            peek(tag("\"")),
        )),
        non_empty,
    )(input)
}

/// Try to recognize a token of the given type at the beginning of `input`.
///
/// Returns `Ok((rest, s))` if the token matches, otherwise returns `Err`.
pub fn match_token<'a>(input: &'a str, token: &SearchToken) -> IResult<&'a str, &'a str> {
    match *token {
        SearchToken::And => and(input),
        SearchToken::Or => or(input),
        SearchToken::Not => not(input),
        SearchToken::Lparen => delimited(space0, tag("("), space0)(input),
        SearchToken::Rparen => delimited(space0, tag(")"), space0)(input),
        SearchToken::Boost => delimited(space0, tag("^"), space0)(input),
        SearchToken::Fuzz => delimited(space0, tag("~"), space0)(input),
        SearchToken::Quote => tag("\"")(input),
        SearchToken::Float => float(input),
        SearchToken::Integer => decimal_integer(input),
        SearchToken::Boolean => alt((tag("true"), tag("false")))(input),
        SearchToken::IpCidr => ip_cidr(input),
        SearchToken::AbsoluteDate4Digit => limited_integer(4, 4)(input),
        SearchToken::AbsoluteDate2Digit => limited_integer(2, 2)(input),
        SearchToken::AbsoluteDateHyphen => tag("-")(input),
        SearchToken::AbsoluteDateColon => tag(":")(input),
        SearchToken::AbsoluteDateTimeSep => alt((tag("T"), tag(" ")))(input),
        SearchToken::AbsoluteDateZulu => tag("Z")(input),
        SearchToken::AbsoluteDateOffsetDirection => alt((tag("+"), tag("-")))(input),
        SearchToken::RelativeDateMultiplier => preceded(
            space1,
            alt((
                tag("seconds"),
                tag("minutes"),
                tag("hours"),
                tag("days"),
                tag("weeks"),
                tag("months"),
                tag("years"),
                tag("second"),
                tag("minute"),
                tag("hour"),
                tag("day"),
                tag("week"),
                tag("month"),
                tag("year"),
            )),
        )(input),
        SearchToken::RelativeDateDirection => {
            preceded(space1, alt((tag("ago"), tag("from now"))))(input)
        }
        SearchToken::Field(s) => tag(s)(input),
        SearchToken::RangeLte => tag(".lte:")(input),
        SearchToken::RangeLt => tag(".lt:")(input),
        SearchToken::RangeGte => tag(".gte:")(input),
        SearchToken::RangeGt => tag(".gt:")(input),
        SearchToken::RangeEq => tag(":")(input),
        SearchToken::Eof => eof(input),
        SearchToken::Term => term(input),
        SearchToken::QuotedTerm => quoted_term(input),
    }
}

/// Try to recognize an ordered sequence of `tokens` from `input.
///
/// Like `match_token`, but recognizes more than one token.
/// Returns `Ok((rest, [s]))` if all tokens match, otherwise `Err`.
pub fn match_tokens<'a>(input: &'a str, tokens: &[SearchToken]) -> IResult<&'a str, Vec<&'a str>> {
    tokens.iter().try_fold(
        (input, Vec::with_capacity(tokens.len())),
        |(input, mut v), t| {
            let (input, s) = match_token(input, t)?;
            v.push(s);
            Ok((input, v))
        },
    )
}

/// Try to recognize an ordered sequence of `tokens` from `input`,
/// considering alternatives in each list.
///
/// Like `match_tokens`, but considers multiple possible tokens
/// at each step.
pub fn match_alternatives<'a>(
    input: &'a str,
    tokens: &[&[SearchToken]],
) -> IResult<&'a str, Vec<&'a str>> {
    tokens.iter().try_fold(
        (input, Vec::with_capacity(tokens.len())),
        |(input, mut v), tokens| {
            for t in *tokens {
                if let Ok((input, s)) = match_token(input, t) {
                    v.push(s);
                    return Ok((input, v));
                }
            }

            Err(nom::Err::Error(Error::from_error_kind(
                input,
                ErrorKind::Tag,
            )))
        },
    )
}

/// Try to recognize an ordered sequence of `tokens` from `input.
///
/// Like `match_token`, but recognizes more than one token.
/// Returns `(rest, [s])` with the number of tokens that matched.
pub fn match_at_most<'a>(input: &'a str, tokens: &[SearchToken]) -> (&'a str, Vec<&'a str>) {
    let mut input = input;
    let mut output = Vec::with_capacity(tokens.len());

    for t in tokens {
        match match_token(input, t) {
            Ok((new_input, t)) => {
                input = new_input;
                output.push(t);
            }
            Err(_) => break
        }
    }

    (input, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limited_integer() {
        assert_eq!(limited_integer(1, 2)("12"), Ok(("", "12")));
        assert_eq!(limited_integer(1, 2)("123"), Ok(("3", "12")));
        assert!(limited_integer(1, 2)("abc").is_err());
    }

    #[test]
    fn test_decimal_integer() {
        assert_eq!(decimal_integer("-123"), Ok(("", "-123")));
        assert_eq!(decimal_integer("-123.4"), Ok((".4", "-123")));
        assert_eq!(decimal_integer("+123"), Ok(("", "+123")));
        assert_eq!(decimal_integer("+123.4"), Ok((".4", "+123")));
        assert!(decimal_integer("+").is_err());
        assert!(decimal_integer("-").is_err());
        assert!(decimal_integer("x").is_err());
    }

    #[test]
    fn test_float() {
        assert_eq!(float("12"), Ok(("", "12")));
        assert_eq!(float("12."), Ok(("", "12.")));
        assert_eq!(float("12.34"), Ok(("", "12.34")));
        assert_eq!(float("12.3a"), Ok(("a", "12.3")));
        assert!(float("ab.cd").is_err());
    }

    #[test]
    fn test_ipv4_addr() {
        assert_eq!(ipv4_addr("255.255.255.255"), Ok(("", "255.255.255.255")));
        assert_eq!(ipv4_addr("255.25.2.0"), Ok(("", "255.25.2.0")));
        assert_eq!(ipv4_addr("0.0.0.0"), Ok(("", "0.0.0.0")));
        assert!(ipv4_addr("::").is_err())
    }

    #[test]
    fn test_ipv4_prefix() {
        assert_eq!(ipv4_prefix("/128"), Ok(("8", "/12")));
        assert_eq!(ipv4_prefix("/32"), Ok(("", "/32")));
        assert_eq!(ipv4_prefix("/16"), Ok(("", "/16")));
        assert_eq!(ipv4_prefix("/8"), Ok(("", "/8")));
        assert!(ipv4_prefix("/").is_err());
    }

    #[test]
    fn test_ipv6_hexadectet() {
        assert_eq!(ipv6_hexadectet("a"), Ok(("", "a")));
        assert_eq!(ipv6_hexadectet("ab"), Ok(("", "ab")));
        assert_eq!(ipv6_hexadectet("abc"), Ok(("", "abc")));
        assert_eq!(ipv6_hexadectet("abcd"), Ok(("", "abcd")));
        assert_eq!(ipv6_hexadectet("123a"), Ok(("", "123a")));
        assert_eq!(ipv6_hexadectet("abcd:"), Ok((":", "abcd")));
        assert_eq!(ipv6_hexadectet("abcde"), Ok(("e", "abcd")));
        assert!(ipv6_hexadectet("g").is_err());
    }

    #[test]
    fn test_ipv6_ls32() {
        assert_eq!(ipv6_ls32("a:a"), Ok(("", "a:a")));
        assert_eq!(ipv6_ls32("ab:ab"), Ok(("", "ab:ab")));
        assert_eq!(ipv6_ls32("abc:abc"), Ok(("", "abc:abc")));
        assert_eq!(ipv6_ls32("abcd:abcd"), Ok(("", "abcd:abcd")));
        assert_eq!(ipv6_ls32("abcd:0"), Ok(("", "abcd:0")));
        assert_eq!(ipv6_ls32("192.168.1.1"), Ok(("", "192.168.1.1")));
        assert!(ipv6_ls32("g").is_err())
    }

    #[test]
    fn test_ipv6_fragment() {
        assert_eq!(ipv6_fragment("a:"), Ok(("", "a:")));
        assert_eq!(ipv6_fragment("ab:"), Ok(("", "ab:")));
        assert_eq!(ipv6_fragment("abc:"), Ok(("", "abc:")));
        assert_eq!(ipv6_fragment("123a:"), Ok(("", "123a:")));
        assert_eq!(ipv6_fragment("abcd:"), Ok(("", "abcd:")));
        assert!(ipv6_fragment("abcde").is_err());
        assert!(ipv6_fragment("abcd").is_err());
        assert!(ipv6_fragment("g").is_err());
    }

    #[test]
    fn test_ipv6_addr() {
        assert_eq!(ipv6_addr("::"), Ok(("", "::")));
        assert_eq!(ipv6_addr("1::2"), Ok(("", "1::2")));
        assert_eq!(ipv6_addr("1:a:2:b:3:c:4:d"), Ok(("", "1:a:2:b:3:c:4:d")));
        assert_eq!(ipv6_addr("::10.0.0.1"), Ok(("", "::10.0.0.1")));
    }

    #[test]
    fn test_ipv6_prefix() {
        assert_eq!(ipv6_prefix("/1024"), Ok(("4", "/102")));
        assert_eq!(ipv6_prefix("/128"), Ok(("", "/128")));
        assert_eq!(ipv6_prefix("/32"), Ok(("", "/32")));
        assert_eq!(ipv6_prefix("/16"), Ok(("", "/16")));
        assert_eq!(ipv6_prefix("/8"), Ok(("", "/8")));
        assert!(ipv6_prefix("/").is_err());
    }

    #[test]
    fn test_ip_cidr() {
        assert_eq!(ip_cidr("::"), Ok(("", "::")));
        assert_eq!(ip_cidr("::/64"), Ok(("", "::/64")));
        assert_eq!(ip_cidr("::127.0.0.1"), Ok(("", "::127.0.0.1")));
        assert_eq!(ip_cidr("::127.0.0.1/24"), Ok(("", "::127.0.0.1/24")));
        assert_eq!(ip_cidr("127.0.0.1"), Ok(("", "127.0.0.1")));
        assert_eq!(ip_cidr("127.0.0.1/24"), Ok(("", "127.0.0.1/24")));
        assert!(ip_cidr("ab.cd.ef.gh").is_err());
    }

    #[test]
    fn test_and() {
        assert_eq!(and(","), Ok(("", ",")));
        assert_eq!(and(", "), Ok(("", ",")));
        assert_eq!(and(" , "), Ok(("", ",")));
        assert_eq!(and(" ,"), Ok(("", ",")));
        assert_eq!(and(" && "), Ok(("", "&&")));
        assert_eq!(and(" AND "), Ok(("", "AND")));
        assert!(and(" AND").is_err());
        assert!(and("AND").is_err());
        assert!(and("AND ").is_err());
        assert!(and(" &&").is_err());
        assert!(and("&&").is_err());
        assert!(and("&& ").is_err());
    }

    #[test]
    fn test_or() {
        assert_eq!(or(" OR "), Ok(("", "OR")));
        assert_eq!(or(" || "), Ok(("", "||")));
        assert!(or(" OR").is_err());
        assert!(or("OR").is_err());
        assert!(or("OR ").is_err());
        assert!(or(" ||").is_err());
        assert!(or("||").is_err());
        assert!(or("|| ").is_err());
    }

    #[test]
    fn test_not() {
        assert_eq!(not(" !"), Ok(("", "!")));
        assert_eq!(not("!"), Ok(("", "!")));
        assert_eq!(not("! "), Ok(("", "!")));
        assert_eq!(not(" ! "), Ok(("", "!")));
        assert_eq!(not(" -"), Ok(("", "-")));
        assert_eq!(not("-"), Ok(("", "-")));
        assert_eq!(not("- "), Ok(("", "-")));
        assert_eq!(not(" - "), Ok(("", "-")));
        assert_eq!(not("NOT "), Ok(("", "NOT")));
        assert_eq!(not(" NOT "), Ok(("", "NOT")));
        assert!(not(" NOT").is_err());
        assert!(not("NOT").is_err());
    }

    #[test]
    fn test_stop_words() {
        assert_eq!(stop_words(","), Ok(("", ",")));
        assert_eq!(stop_words(" && "), Ok(("", "&&")));
        assert_eq!(stop_words(" AND "), Ok(("", "AND")));
        assert_eq!(stop_words(" || "), Ok(("", "||")));
        assert_eq!(stop_words(" OR "), Ok(("", "OR")));
        assert_eq!(stop_words(") "), Ok((" ", ")")));
        assert_eq!(stop_words(" )"), Ok(("", ")")));
        assert_eq!(stop_words(""), Ok(("", "")));
        assert!(stop_words("abcd").is_err());
    }

    #[test]
    fn test_term_split() {
        assert_eq!(term_split("^123"), Ok(("", "^123")));
        assert!(term_split("abcd").is_err());
    }

    #[test]
    fn test_escaped_char() {
        assert_eq!(escaped_char("\\x"), Ok(("", "\\x")));
        assert!(escaped_char("\\").is_err());
        assert!(escaped_char("x").is_err());
    }

    #[test]
    fn test_subexpression() {
        assert_eq!(subexpression("(flower)"), Ok(("", "(flower)")));
        assert!(subexpression("(flower (flower))").is_err());
        assert!(subexpression("rose (flower)").is_err());
    }

    #[test]
    fn test_non_empty() {
        assert!(non_empty("").is_err());
        assert!(non_empty(" ").is_ok());
    }

    #[test]
    fn test_term() {
        assert_eq!(term("rose"), Ok(("", "rose")));
        assert_eq!(term("rose,"), Ok((",", "rose")));
        assert_eq!(term("rose)"), Ok((")", "rose")));
        assert_eq!(term("rose (flower)"), Ok(("", "rose (flower)")));
        assert_eq!(term("rose (flower),"), Ok((",", "rose (flower)")));
        assert_eq!(term("rose (flower))"), Ok((")", "rose (flower)")));
        assert_eq!(term("rose \\(flower\\)"), Ok(("", "rose \\(flower\\)")));
        assert_eq!(term("rose \\(flower\\),"), Ok((",", "rose \\(flower\\)")));
        assert_eq!(term("rose \\(flower\\))"), Ok((")", "rose \\(flower\\)")));
        assert!(term(")").is_err())
    }

    #[test]
    fn test_quoted_term() {
        assert_eq!(quoted_term("rose\""), Ok(("\"", "rose")));
        assert_eq!(quoted_term("rose,\""), Ok(("\"", "rose,")));
        assert_eq!(quoted_term("rose)\""), Ok(("\"", "rose)")));
        assert_eq!(
            quoted_term("rose \\(flower\\),\""),
            Ok(("\"", "rose \\(flower\\),"))
        );
        assert_eq!(
            quoted_term("rose \\(flower\\))\""),
            Ok(("\"", "rose \\(flower\\))"))
        );
        assert!(quoted_term("\"").is_err());
        assert!(quoted_term("").is_err());
    }

    #[test]
    fn test_match_token() {
        let input = "foo, bar";
        let (input, t1) = match_token(input, &SearchToken::Term).unwrap();
        let (input, t2) = match_token(input, &SearchToken::And).unwrap();
        let (input, t3) = match_token(input, &SearchToken::Term).unwrap();
        let (input, t4) = match_token(input, &SearchToken::Eof).unwrap();

        assert_eq!(t1, "foo");
        assert_eq!(t2, ",");
        assert_eq!(t3, "bar");
        assert_eq!(t4, "");
        assert_eq!(input, "");
    }

    #[test]
    fn test_match_tokens() {
        let input = "1 AND 1 OR 1 NOT ()^~\"1234.true127.0.0.1/322000-00T:+Z seconds agocreated_at.lt:.lte:.gt:.gte::term";
        let (_input, tokens) = match_tokens(
            input,
            &[
                SearchToken::Integer,
                SearchToken::And,
                SearchToken::Integer,
                SearchToken::Or,
                SearchToken::Integer,
                SearchToken::Not,
                SearchToken::Lparen,
                SearchToken::Rparen,
                SearchToken::Boost,
                SearchToken::Fuzz,
                SearchToken::Quote,
                SearchToken::Float,
                SearchToken::Boolean,
                SearchToken::IpCidr,
                SearchToken::AbsoluteDate4Digit,
                SearchToken::AbsoluteDateHyphen,
                SearchToken::AbsoluteDate2Digit,
                SearchToken::AbsoluteDateTimeSep,
                SearchToken::AbsoluteDateColon,
                SearchToken::AbsoluteDateOffsetDirection,
                SearchToken::AbsoluteDateZulu,
                SearchToken::RelativeDateMultiplier,
                SearchToken::RelativeDateDirection,
                SearchToken::Field("created_at"),
                SearchToken::RangeLt,
                SearchToken::RangeLte,
                SearchToken::RangeGt,
                SearchToken::RangeGte,
                SearchToken::RangeEq,
                SearchToken::Term,
            ],
        )
        .unwrap();

        assert_eq!(tokens.len(), 30);

        let input = "\"foo\"";
        let (_, tokens) = match_tokens(
            input,
            &[
                SearchToken::Quote,
                SearchToken::QuotedTerm,
                SearchToken::Quote,
                SearchToken::Eof,
            ],
        )
        .unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens, vec!["\"", "foo", "\"", ""]);

        let input = "foo, bar";
        let result = match_tokens(input, &[SearchToken::Term, SearchToken::Eof]);

        assert!(result.is_err());
    }

    #[test]
    fn test_match_alternatives() {
        let inputs = [
            ("created_at.lt:2020", ".lt:"),
            ("created_at.lte:2020", ".lte:"),
            ("created_at.gt:2020", ".gt:"),
            ("created_at.gte:2020", ".gte:"),
            ("created_at:2020", ":"),
        ];

        for (input, expected) in inputs.iter() {
            let (_, tokens) = match_alternatives(
                input,
                &[
                    &[SearchToken::Field("created_at")],
                    &[
                        SearchToken::RangeLt,
                        SearchToken::RangeLte,
                        SearchToken::RangeGt,
                        SearchToken::RangeGte,
                        SearchToken::RangeEq,
                    ],
                    &[SearchToken::AbsoluteDate4Digit],
                ],
            )
            .unwrap();

            assert_eq!(tokens.len(), 3);
            assert_eq!(tokens[0], "created_at");
            assert_eq!(&tokens[1], expected);
            assert_eq!(tokens[2], "2020");
        }

        assert!(match_alternatives("1234", &[&[SearchToken::Boolean]]).is_err());
    }

    #[test]
    fn test_match_at_most() {
        let inputs = [
            ("2020-01-01T00:00:00", vec!["2020", "-", "01", "-", "01", "T", "00", ":", "00", ":", "00"]),
            ("2020-01-01T00:00", vec!["2020", "-", "01", "-", "01", "T", "00", ":", "00"]),
            ("2020-01-01T00", vec!["2020", "-", "01", "-", "01", "T", "00"]),
            ("2020-01-01T", vec!["2020", "-", "01", "-", "01", "T"]),
            ("2020-01-01", vec!["2020", "-", "01", "-", "01"]),
            ("2020-01", vec!["2020", "-", "01"]),
            ("2020", vec!["2020"]),
            ("2", vec![]),
        ];

        for (input, expected) in inputs.iter() {
            let (_, tokens) = match_at_most(
                input,
                &[
                    SearchToken::AbsoluteDate4Digit,
                    SearchToken::AbsoluteDateHyphen,
                    SearchToken::AbsoluteDate2Digit,
                    SearchToken::AbsoluteDateHyphen,
                    SearchToken::AbsoluteDate2Digit,
                    SearchToken::AbsoluteDateTimeSep,
                    SearchToken::AbsoluteDate2Digit,
                    SearchToken::AbsoluteDateColon,
                    SearchToken::AbsoluteDate2Digit,
                    SearchToken::AbsoluteDateColon,
                    SearchToken::AbsoluteDate2Digit
                ]
            );

            assert_eq!(&tokens, expected);
        }
    }
}
