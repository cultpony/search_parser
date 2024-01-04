use std::{ops::Range, rc::Rc};

use crate::tokens::Token;

/// A section of a string delimited by start and end position
#[derive(Clone, Eq)]
pub struct Span {
    internal_string: Rc<str>,
    range: (usize, usize),
}

impl Span {
    pub fn new(s: Rc<str>, range: Range<usize>) -> Span {
        Span {
            internal_string: s,
            range: (range.start, range.end),
        }
    }
    pub fn str<'b>(&'b self) -> &'b str {
        &self.internal_string[self.range.0..self.range.1]
    }
    pub fn len(&self) -> usize {
        (self.range.0..self.range.1).len()
    }
    pub fn is_empty(&self) -> bool {
        (self.range.0..self.range.1).is_empty()
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "@{}..{}:{:?}",
            self.range.0,
            self.range.1,
            self.internal_string
                .get(self.range.0..self.range.1)
                .unwrap_or_default()
        ))
    }
}

impl<'a> PartialEq<str> for Span {
    fn eq(&self, other: &str) -> bool {
        self.str() == other
    }
}

impl<'a> PartialEq<&str> for Span  {
    fn eq(&self, other: &&str) -> bool {
        self.str() == *other
    }
}

impl<'a, 'b> PartialEq<Span> for Span  {
    fn eq(&self, other: &Span) -> bool {
        self.internal_string == other.internal_string && self.range == other.range
    }
}

#[derive(Clone, Eq)]
pub struct TokenSpan {
    pub token: Token,
    span: Span,
}

impl TokenSpan {
    pub fn new(s: Rc<str>, range: Range<usize>, token: Token) -> TokenSpan {
        TokenSpan {
            token,
            span: Span::new(s, range),
        }
    }
    pub fn empty() -> TokenSpan {
        TokenSpan {
            token: Token::NONE,
            span: Span::new(Rc::from(""), 0..0),
        }
    }
    pub fn from_span(token: Token, span: Span) -> TokenSpan {
        TokenSpan { token, span }
    }
    pub fn with_token(self, token: Token) -> TokenSpan {
        TokenSpan {
            token,
            span: self.span,
        }
    }
    pub fn with_str(self, s: &str) -> TokenSpan {
        assert!(s.len() >= self.span.range.0);
        assert!(s.len() >= self.span.range.1);
        TokenSpan {
            token: self.token,
            span: Span {
                internal_string: Rc::from(s),
                range: self.span.range,
            },
        }
    }
    pub fn trim_end_whitespace(&mut self) {
        self.span.range.1 = self.span.str()
            .trim_end().len() + self.span.range.0
    }
    pub fn str(&self) -> &str {
        self.span.str()
    }
    pub fn len(&self) -> usize {
        self.span.len()
    }
    pub fn is_empty(&self) -> bool {
        self.span.is_empty()
    }
    pub fn token(&self) -> Token {
        self.token
    }
}

impl PartialEq<str> for TokenSpan {
    fn eq(&self, other: &str) -> bool {
        self.str() == other
    }
}

impl PartialEq<&str> for TokenSpan {
    fn eq(&self, other: &&str) -> bool {
        self.str() == *other
    }
}

impl PartialEq<TokenSpan> for TokenSpan {
    fn eq(&self, other: &TokenSpan) -> bool {
        self.token == other.token && self.span == other.span
    }
}

impl<'a> std::fmt::Debug for TokenSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}{:?}", self.token, self.span))
    }
}