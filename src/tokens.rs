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
}
