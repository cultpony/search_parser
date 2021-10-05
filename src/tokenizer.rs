use nom::{
    branch::alt,
    bytes::complete::{tag, take_while_m_n},
    character::complete::{anychar, line_ending, none_of, space0, space1},
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
    /// A line ending. LF or CRLF.
    Newline,
}

/// Returns whether the character is a decimal digit (0123456789).
pub fn is_dec_digit(c: char) -> bool {
    ('0'..='9').contains(&c)
}

/// Returns whether the character is a hexadecimal digit
/// (0123456789abcdefABCDEF).
pub fn is_hex_digit(c: char) -> bool {
    ('0'..='9').contains(&c) || ('a'..='f').contains(&c) || ('A'..='F').contains(&c)
}

/// Recognizes all text until newline.
pub fn remainder_of_line(input: &str) -> IResult<&str, &str> {
    recognize(many_till(none_of("\r\n"), alt((line_ending, eof))))(input)
}

/// Recognizes a comment.
pub fn comment(input: &str) -> IResult<&str, &str> {
    recognize(tuple((opt(space0), tag("#"), remainder_of_line)))(input)
}

/// Recognizes an empty line.
pub fn empty_line(input: &str) -> IResult<&str, &str> {
    recognize(tuple((line_ending, space0, line_ending)))(input)
}

/// Returns a parser which recognizes an integer having at least `min`
/// characters and at most `max` characters.
pub fn limited_integer(min: usize, max: usize) -> impl Fn(&str) -> IResult<&str, &str> {
    move |input| recognize(take_while_m_n(min, max, is_dec_digit))(input)
}

/// Recognizes a signed decimal integer with at most 32 digits.
pub fn decimal_integer(input: &str) -> IResult<&str, &str> {
    recognize(preceded(
        opt(alt((tag("+"), tag("-")))),
        limited_integer(1, 32),
    ))(input)
}

/// Recognizes a signed real number with at most 64 digits of precision.
pub fn float(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        decimal_integer,
        opt(preceded(tag("."), opt(limited_integer(1, 32)))),
    ))(input)
}

/// Recognizes an IPv4 address, like `1.2.3.4`.
pub fn ipv4_addr(input: &str) -> IResult<&str, &str> {
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
pub fn ipv4_prefix(input: &str) -> IResult<&str, &str> {
    recognize(pair(tag("/"), limited_integer(1, 2)))(input)
}

/// Recognizes a 2-byte segment of an IPv6 address, like `abcd`.
pub fn ipv6_hexadectet(input: &str) -> IResult<&str, &str> {
    recognize(take_while_m_n(1, 4, is_hex_digit))(input)
}

/// Recognizes the least significant 32 bits of an IPv6 address.
///
/// This could be two hexadectets, like `abcd:dcba`, or an IPv4 address,
/// like `170.170.170.170`.
pub fn ipv6_ls32(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(tuple((ipv6_hexadectet, tag(":"), ipv6_hexadectet))),
        ipv4_addr,
    ))(input)
}

/// Recognizes a "fragment" of an IPv6 address, e.g. a hexadectet followed
/// by a colon (`aaaa:`).
pub fn ipv6_fragment(input: &str) -> IResult<&str, &str> {
    recognize(pair(ipv6_hexadectet, tag(":")))(input)
}

/// Recognizes an IPv6 address, in expanded or shortened form.
pub fn ipv6_addr(input: &str) -> IResult<&str, &str> {
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
pub fn ipv6_prefix(input: &str) -> IResult<&str, &str> {
    recognize(pair(tag("/"), limited_integer(1, 3)))(input)
}

/// Recognizes a fully specified IP address, like `127.0.0.1` or
/// `2200:dead:beef::cafe`, or an IP prefix specified in CIDR notation,
/// like `1.2.3.4/24` or `2200::dead:beef::/64`.
pub fn ip_cidr(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(tuple((ipv4_addr, opt(ipv4_prefix)))),
        recognize(tuple((ipv6_addr, opt(ipv6_prefix)))),
    ))(input)
}

/// Recognizes a conjunction operator, like `x AND y`, `x && y`, or `x, y`.
pub fn and(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(space1, tag("AND"), space1),
        delimited(space0, alt((tag(","), tag("&&"))), space0),
    ))(input)
}

/// Recognizes a disjunction operator, like `x OR y` or `x || y`.
pub fn or(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(space1, tag("OR"), space1),
        delimited(space0, tag("||"), space0),
    ))(input)
}

// Recognizes a newline.
pub fn newline(input: &str) -> IResult<&str, &str> {
    delimited(space0, line_ending, space0)(input)
}

/// Recognizes a negation operator, like `!x`, `-x`, or `NOT x`.
pub fn not(input: &str) -> IResult<&str, &str> {
    alt((
        delimited(space0, alt((tag("!"), tag("-"))), space0),
        delimited(space0, tag("NOT"), space1),
    ))(input)
}

/// Recognizes the stop words which delimit unquoted terms. These are:
/// - conjunction (`x,`, `x AND `, `x && `)
/// - disjunction (`x || `, `x OR `)
/// - line ending (`\n`, `\r\n`)
/// - comment (`#`)
/// - closing parenthesis (`x)`)
/// - end of input
pub fn stop_words(input: &str) -> IResult<&str, &str> {
    alt((and, or, newline, comment, preceded(space0, tag(")")), eof))(input)
}

/// Recognizes the full sequence of tokens which delimit unquoted terms.
///
/// This includes everything from `stop_words`, as well as an additional rule
/// recognizing caret expressions (`^1234)`). This allows tokenization to avoid
/// consuming a boost rule as part of a term.
pub fn term_split(input: &str) -> IResult<&str, &str> {
    alt((
        stop_words,
        recognize(delimited(tag("^"), float, stop_words)),
    ))(input)
}

/// Recognizes a character which has been escaped (preceded by a backslash).
///
/// Escaped characters are included verbatim in the term and not considered
/// for further tokenization.
pub fn escaped_char(input: &str) -> IResult<&str, &str> {
    recognize(pair(tag("\\"), anychar))(input)
}

/// Recognizes a legal subexpression within an unquoted term.
///
/// An unquoted term like `rose (flower)` contains a parenthesized
/// subexpression `(flower)`. No recursion is used to recognize this,
/// only one level of parentheses may be used.
pub fn subexpression(input: &str) -> IResult<&str, &str> {
    recognize(preceded(
        tag("("),
        many_till(alt((escaped_char, recognize(none_of("()")))), tag(")")),
    ))(input)
}

/// Recognizes any non-empty string.
pub fn non_empty(input: &str) -> Result<&str, nom::Err<&str>> {
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
pub fn term(input: &str) -> IResult<&str, &str> {
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
pub fn quoted_term(input: &str) -> IResult<&str, &str> {
    map_res(
        recognize(many_till(
            alt((escaped_char, recognize(anychar))),
            peek(tag("\"")),
        )),
        non_empty,
    )(input)
}

/// Matches two newlines directly after each other,
/// optionally with spaces in-between them.
fn match_empty_line(input: &str) -> Option<&str> {
    match empty_line(input) {
        Ok((remainder, _)) => Some(remainder),
        _ => None,
    }
}

/// Matches comments.
fn match_comments(input: &str) -> Option<&str> {
    match comment(input) {
        Ok((remainder, _)) => Some(remainder),
        _ => match_empty_line(input),
    }
}

/// Strips the immediate comments and empty lines
/// from the input.
fn strip_comments(input: &str) -> &str {
    let mut buf = input;

    while let Some(remainder) = match_comments(buf) {
        buf = remainder;
    }

    buf
}

/// Try to recognize a token of the given type at the beginning of `input`.
///
/// Returns `Ok((rest, s))` if the token matches, otherwise returns `Err`.
pub fn match_token<'a>(input: &'a str, token: &SearchToken) -> IResult<&'a str, &'a str> {
    let input = strip_comments(input);

    let result = match *token {
        SearchToken::And => and(input),
        SearchToken::Or => or(input),
        SearchToken::Newline => newline(input),
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
    };

    match result {
        Ok((input, term)) => Ok((strip_comments(input), strip_comments(term))),
        _ => result,
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
            Err(_) => break,
        }
    }

    (input, output)
}
