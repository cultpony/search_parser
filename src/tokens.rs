#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum Token {
    /// "("
    LPAREN = 0,
    /// ")"
    RPAREN,
    /// "AND" or "&&" or ","
    AND,
    /// "OR" or "||"
    OR,
    /// "NOT", "!" or "-"
    NOT,
    /// "^"
    BOOST,
    /// "~"
    FUZZ,
    /// "\""
    QUOTE,
    /// 1234.5678
    FLOAT,
    /// 1234
    INTEGER,
    /// "true" or "false"
    BOOLEAN,
    /// IP address in CIDR notation
    IP_CIDR,
    /// 4 digit decimal year
    ABSOLUTE_DATE,
    /// "seconds"/"second", "minutes"/"minute", etc. or "ago" or "from now"
    RELATIVE_DATE,
    /// A field name
    FIELD,
    /// Like field but a Tag explicitly not a field
    TAG,
    /// ".lte:", ".lt:", ".gte:", ".gt:", ".eq", ".neq"
    RANGE,
    /// new line LF or CRLF
    NEWLINE,
    /// End of Input
    EOI,

    /// Term not surrounded with [Token::QUOTE]
    UNQUOTED_TERM,
    /// Term surrounded with [Token::QUOTE]
    QUOTED_TERM,
    /// Either a [Token::UNQUOTED_TERM] or a [Token::QUOTED_TERM]
    TERM,
    /// A [Token::FIELD] followed by a [Token::RANGE] and then a [Token::TERM]
    EXPRESSION,
    /// Two [Token::EXPRESSION] items joined by [Token::COMPARATOR]
    COMBINATOR,
    /// [Token::LPAREN] and [Token::RPAREN] surrounding a [Token::SEARCH_TERM]
    GROUP,
    /// Either [Token::TAG], [Token::EXPRESSION], [Token::GROUP] or [Token::COMBINATOR]
    SEARCH_TERM,
    /// No Token
    NONE,
    /// Root Node of AST
    ROOT,
}

impl Token {
    pub fn name(&self) -> &'static str {
        match self {
            Token::LPAREN => "(",
            Token::RPAREN => ")",
            Token::AND => "AND",
            Token::OR => "OR",
            Token::NOT => "NOT",
            Token::BOOST => "^",
            Token::FUZZ => "~",
            Token::QUOTE => "\"",
            Token::FLOAT => "Decimal Number",
            Token::INTEGER => "Integer",
            Token::BOOLEAN => "Boolean",
            Token::IP_CIDR => "IP Address",
            Token::ABSOLUTE_DATE => "Absolute Date",
            Token::RELATIVE_DATE => "Relative Date",
            Token::FIELD => "Field or Tag",
            Token::TAG => "Tag",
            Token::RANGE => "Range",
            Token::NEWLINE => "New Line",
            Token::EOI => "End of Input",
            Token::UNQUOTED_TERM => "Unquoted Term",
            Token::QUOTED_TERM => "Quoted Term",
            Token::TERM => "Term",
            Token::EXPRESSION => "Expression",
            Token::COMBINATOR => "Combinator",
            Token::GROUP => "Group",
            Token::SEARCH_TERM => "Search Term",
            Token::NONE => "None",
            Token::ROOT => "Start of Input",
        }
    }
    /*pub fn next_sequence(&self) -> &'static [Token] {
        use Token::*;
        const EXPR_TERMINATOR: &[Token] = &[AND, OR, RPAREN];
        const NEW_EXPR: &[Token] = &[LPAREN, NOT, BOOST, FUZZ, FIELD];
        const AFTER_TERM: &[Token] = &[RANGE, AND, OR, RPAREN];
        const AFTER_EXPR: &[Token] = &[AND, OR, RPAREN];
        match self {
            Token::LPAREN => NEW_EXPR,
            Token::RPAREN => EXPR_TERMINATOR,
            Token::AND => NEW_EXPR,
            Token::OR => NEW_EXPR,
            Token::NOT => NEW_EXPR,
            Token::BOOST => NEW_EXPR,
            Token::FUZZ => NEW_EXPR,
            Token::QUOTE => &[RANGE, AND, OR, QUOTED_TERM],
            Token::FLOAT => EXPR_TERMINATOR,
            Token::INTEGER => EXPR_TERMINATOR,
            Token::BOOLEAN => EXPR_TERMINATOR,
            Token::IP_CIDR => EXPR_TERMINATOR,
            Token::ABSOLUTE_DATE => EXPR_TERMINATOR,
            Token::RELATIVE_DATE => EXPR_TERMINATOR,
            Token::FIELD => &[RANGE, AND, OR, RPAREN],
            Token::TAG => &[AND, OR, RPAREN],
            Token::RANGE => &[TERM, QUOTE],
            Token::NEWLINE => NEW_EXPR,
            Token::EOI => &[],
            Token::UNQUOTED_TERM => AFTER_TERM,
            Token::QUOTED_TERM => &[QUOTE, NONE, NONE],
            Token::TERM => AFTER_EXPR,
            Token::EXPRESSION => AFTER_EXPR,
            Token::COMBINATOR => AFTER_EXPR,
            Token::GROUP => AFTER_EXPR,
            Token::SEARCH_TERM => &[EOI, NEWLINE],
            Token::ROOT => NEW_EXPR,
            Token::NONE => &[],
        }
    }

    pub fn token_lit(&self) -> Option<&'static [&'static str]> {
        match self {
            Token::LPAREN => Some(&["("]),
            Token::RPAREN => Some(&[")"]),
            Token::AND => Some(&["AND", "&&", ","]),
            Token::OR => Some(&["OR", "||"]),
            Token::NOT => Some(&["NOT", "!", "-"]),
            Token::BOOST => Some(&["^"]),
            Token::FUZZ => Some(&["~"]),
            Token::QUOTE => Some(&["\""]),
            Token::FLOAT => None,
            Token::INTEGER => None,
            Token::BOOLEAN => Some(&["true", "false", "yes", "no"]),
            Token::IP_CIDR => None,
            Token::ABSOLUTE_DATE => None,
            Token::RELATIVE_DATE => None,
            Token::FIELD => None,
            Token::TAG => None,
            Token::RANGE => Some(&[".gte:", ".gt:", ".lte:", ".lt:", ".eq:", ".neq:"]),
            Token::NEWLINE => Some(&["\n"]),
            Token::EOI => None,
            Token::UNQUOTED_TERM => None,
            Token::QUOTED_TERM => None,
            Token::TERM => None,
            Token::EXPRESSION => None,
            Token::COMBINATOR => None,
            Token::GROUP => None,
            Token::SEARCH_TERM => None,
            Token::NONE => None,
            Token::ROOT => None,
        }
    }*/

    /*pub fn token_regex(self) -> Option<&'static Regex> {
        lazy_static::lazy_static! {
            static ref FLOAT: Regex = Regex::new(r"(?P<float>^[+-]{0,1}\d+\.\d+)").unwrap();
            static ref INTEGER: Regex = Regex::new(r"(?P<int>^[+-]{0,1}\d+)").unwrap();
            static ref FIELD: Regex = Regex::new(r"(?P<field>^[^.\(\),\s]+)(?:\s(AND|OR|\.[gl]te?:|\.n?eq)(\s)){0,1}").unwrap();
            static ref IP_CIDR: Regex = Regex::new(r"(?P<ip>(\b25[0-5]|\b2[0-4][0-9]|\b[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}|(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])))(?P<netmask>/\d+)?").unwrap();
            static ref ABS_DATE: Regex = Regex::new(r"^(?P<year>\d{4}(-(?P<month>\d{2})(-(?P<day>\d{2}))?)?)((T| )(?P<hour>\d{2}(:(?P<minute>\d{2}(:(?P<second>\d{2}))?))?))?(?P<offset_hour>[+-]\d{2}(:(?P<offset_minute>\d{2}))?|(?P<zulu>Z))?").unwrap();
            static ref REL_DATE: Regex = Regex::new(r"((?P<years>\d+ years?)\s+)?((?P<months>\d+ months?)\s+)?((?P<weeks>\d+ weeks?)\s+)?((?P<days>\d+ days?)\s+)?((?P<hours>\d+ hours?)\s+)?((?P<minutes>\d+ minutes?)\s+)?((?P<seconds>\d+ seconds?)\s+)?(ago|from now)").unwrap();
        }

        match self {
            Token::FLOAT => Some(&FLOAT),
            Token::INTEGER => Some(&INTEGER),
            Token::FIELD => Some(&FIELD),
            Token::ABSOLUTE_DATE => Some(&ABS_DATE),
            Token::RELATIVE_DATE => Some(&REL_DATE),
            Token::IP_CIDR => Some(&IP_CIDR),
            _ => None,
        }
    }

    // When checking for a token, use these tokens instead
    pub fn token_descend(self) -> Option<&'static [Token]> {
        match self {
            Token::TERM => Some(&[
                Token::FLOAT,
                Token::INTEGER,
                Token::FIELD,
                Token::BOOLEAN,
                Token::ABSOLUTE_DATE,
                Token::RELATIVE_DATE,
                Token::IP_CIDR,
            ]),
            _ => None,
        }
    }*/
}
