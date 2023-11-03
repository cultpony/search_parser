use std::ops::Range;

use crate::tokens::Token;

/// A section of a string delimited by start and end position
#[derive(Clone, Eq, Copy)]
pub struct Span<'a> {
    internal_string: &'a str,
    range: (usize, usize),
}

#[derive(Clone)]
pub struct OwnedSpan {
    internal_string: String,
    range: (usize, usize),
}

impl<'a> Span<'a> {
    pub fn new(s: &str, range: Range<usize>) -> Span<'_> {
        Span {
            internal_string: s,
            range: (range.start, range.end),
        }
    }
    pub fn str<'b>(&'b self) -> &'a str {
        &self.internal_string[self.range.0..self.range.1]
    }
    pub fn len(&self) -> usize {
        (self.range.0..self.range.1).len()
    }
    pub fn is_empty(&self) -> bool {
        (self.range.0..self.range.1).is_empty()
    }
    pub fn to_owned_span(&self) -> OwnedSpan {
        OwnedSpan {
            internal_string: self.internal_string.to_owned(),
            range: self.range,
        }
    }
}

impl<'a> std::fmt::Debug for Span<'a> {
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

impl std::fmt::Debug for OwnedSpan {
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

impl<'a> PartialEq<str> for Span<'a> {
    fn eq(&self, other: &str) -> bool {
        self.str() == other
    }
}

impl<'a> PartialEq<&str> for Span<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.str() == *other
    }
}

impl<'a, 'b> PartialEq<Span<'b>> for Span<'a> {
    fn eq(&self, other: &Span<'b>) -> bool {
        self.internal_string == other.internal_string && self.range == other.range
    }
}

#[derive(Clone, Eq, Copy)]
pub struct TokenSpan<'a> {
    pub token: Token,
    span: Span<'a>,
}

pub struct OwnedTokenSpan {
    pub token: Token,
    span: OwnedSpan,
}

impl<'a> From<&TokenSpan<'a>> for OwnedTokenSpan {
    fn from(value: &TokenSpan<'a>) -> Self {
        Self {
            token: value.token,
            span: value.span.to_owned_span(),
        }
    }
}

impl<'a> TokenSpan<'a> {
    pub fn new(s: &str, range: Range<usize>, token: Token) -> TokenSpan<'_> {
        TokenSpan {
            token,
            span: Span::new(s, range),
        }
    }
    pub fn empty() -> TokenSpan<'static> {
        TokenSpan {
            token: Token::NONE,
            span: Span::new("", 0..0),
        }
    }
    pub fn from_span(token: Token, span: Span<'a>) -> TokenSpan<'a> {
        TokenSpan { token, span }
    }
    pub fn with_token(self, token: Token) -> TokenSpan<'a> {
        TokenSpan {
            token,
            span: self.span,
        }
    }
    pub fn with_str(self, s: &str) -> TokenSpan<'_> {
        assert!(s.len() >= self.span.range.0);
        assert!(s.len() >= self.span.range.1);
        TokenSpan {
            token: self.token,
            span: Span {
                internal_string: s,
                range: self.span.range,
            },
        }
    }
    pub fn trim_end_whitespace(&mut self) {
        self.span.range.1 = self.span.str()
            .trim_end().len() + self.span.range.0
    }
    pub fn str<'b>(&'b self) -> &'a str {
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

impl<'a> PartialEq<str> for TokenSpan<'a> {
    fn eq(&self, other: &str) -> bool {
        self.str() == other
    }
}

impl<'a> PartialEq<&str> for TokenSpan<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.str() == *other
    }
}

impl<'a, 'b> PartialEq<TokenSpan<'b>> for TokenSpan<'a> {
    fn eq(&self, other: &TokenSpan<'b>) -> bool {
        self.token == other.token && self.span == other.span
    }
}

impl<'a> std::fmt::Debug for TokenSpan<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}{:?}", self.token, self.span))
    }
}

impl std::fmt::Debug for OwnedTokenSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}{:?}", self.token, self.span))
    }
}
