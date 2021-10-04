#[cfg(test)]
use crate::tokenizer::*;

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
    assert_eq!(and("&& "), Ok(("", "&&")));
    assert_eq!(and(" &&"), Ok(("", "&&")));
    assert_eq!(and("&&"), Ok(("", "&&")));
    assert_eq!(and(" AND "), Ok(("", "AND")));
    assert!(and(" AND").is_err());
    assert!(and("AND").is_err());
    assert!(and("AND ").is_err());
}

#[test]
fn test_or() {
    assert_eq!(or(" OR "), Ok(("", "OR")));
    assert_eq!(or(" || "), Ok(("", "||")));
    assert_eq!(or("||"), Ok(("", "||")));
    assert_eq!(or("|| "), Ok(("", "||")));
    assert_eq!(or(" ||"), Ok(("", "||")));
    assert_eq!(or("\n"), Ok(("", "\n")));
    assert_eq!(or("\r\n"), Ok(("", "\r\n")));
    assert!(or(" OR").is_err());
    assert!(or("OR").is_err());
    assert!(or("OR ").is_err());
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
        (
            "2020-01-01T00:00:00",
            vec![
                "2020", "-", "01", "-", "01", "T", "00", ":", "00", ":", "00",
            ],
        ),
        (
            "2020-01-01T00:00",
            vec!["2020", "-", "01", "-", "01", "T", "00", ":", "00"],
        ),
        (
            "2020-01-01T00",
            vec!["2020", "-", "01", "-", "01", "T", "00"],
        ),
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
                SearchToken::AbsoluteDate2Digit,
            ],
        );

        assert_eq!(&tokens, expected);
    }
}

#[test]
fn test_remainder_of_line() {
    assert_eq!(remainder_of_line("hello world 123\nnot part of it"), Ok(("not part of it", "hello world 123\n")));
    assert_eq!(remainder_of_line("hello world 123\r\nnot part of it"), Ok(("not part of it", "hello world 123\r\n")));
}

#[test]
fn test_comments() {
    assert_eq!(comment("# hello world"), Ok(("", "# hello world")));
    assert_eq!(comment(" # hello world"), Ok(("", " # hello world")));
    assert_eq!(comment("# hello world\ntest"), Ok(("test", "# hello world\n")));
    assert_eq!(comment("# hello world\r\ntest"), Ok(("test", "# hello world\r\n")));
}
