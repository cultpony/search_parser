use ip_network::IpNetwork;
use time::{Duration, OffsetDateTime};

use crate::{tokens::Token, span::TokenSpan};


pub type Field = String;
pub type Tag = String;
pub type TagList = Vec<Tag>;
pub type ExprList = Vec<Expr>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A property field
    Field(Field),
    /// A single tag
    Tag(Tag),
    /// A list of tags (optimization of single tag field)
    Tags(TagList),
    /// A single-argument operation (such as NOT)
    Apply(ApplyOp, Box<Expr>),
    /// A comparison operation
    Comparison(Field, Comp, Value),
    /// A logical combination of all given expression
    Combine(CombOp, Vec<Expr>),
    /// A group of expressions, must be optimized out
    /// by the end, this mostly serves for the SR parsers
    Group(Vec<Expr>),
    /// An empty, false-y expression
    Empty,
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Group(Vec::new())
    }
}

#[derive(Debug, Clone, PartialEq, Copy, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Value {
    Integer(i128),
    Float(f64),
    Bool(bool),
    IP(IpNetwork),
    RelativeDate(Duration),
    AbsoluteDate(OffsetDateTime),
    Undefined,
}

impl<'a> From<TokenSpan<'a>> for Value {
    fn from(value: TokenSpan<'a>) -> Self {
        match value.token() {
            Token::FLOAT => Self::Float(value.str().parse().unwrap()),
            Token::INTEGER => Self::Integer(value.str().parse().unwrap()),
            Token::BOOLEAN => Self::Bool(match value.str() {
                "true" | "yes" => true,
                "false" | "no" => false,
                _ => unreachable!(),
            }),
            Token::IP_CIDR => todo!(),
            Token::ABSOLUTE_DATE => todo!(),
            Token::RELATIVE_DATE => todo!(),
            _ => unreachable!("lexer fucked up the lexem checking")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOp {
    Not,
    Fuzz,
    Boost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comp {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombOp {
    And,
    Or,
}

impl From<Token> for CombOp {
    fn from(value: Token) -> Self {
        match value {
            Token::AND => Self::And,
            Token::OR => Self::Or,
            _ => unreachable!(),
        }
    }
}

impl PartialEq<Token> for CombOp {
    fn eq(&self, other: &Token) -> bool {
        let other: CombOp = (*other).into();
        *self == other
    }
}