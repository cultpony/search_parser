#[cfg(test)]
mod tests;

use crate::errors::{Error, Result};
use crate::ITokenizer;
use crate::{
    span::{Span, TokenSpan},
    tokens::Token,
};
use bumpalo::collections::CollectIn;
use bumpalo::collections::vec::Vec;
use tracing::{debug, trace};

// Estimates memory consumption if the output is maximally pessimistic
pub fn maximum_memory_estimate(input: &str) -> usize {
    input.len() * (
        std::mem::size_of::<usize>() * 16
    )
    // allowance for scratch spaces
    + 8000
}

#[derive(Debug, Clone)]
pub struct Scanner<'a> {
    input: &'a str,
    position: usize,
    peek_position: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Scanner<'a> {
        Scanner {
            input,
            position: 0,
            peek_position: 0,
        }
    }

    fn mk_span(&self) -> Span<'a> {
        let span = Span::new(self.input, self.position..self.peek_position);
        trace!("Generating new span {:?}", span);
        span
    }

    // Peeks ahead in the stream and returns the new peek position
    // or none if the maximum peek (end of input) is achieved
    fn st_peek(&mut self, peek: usize) -> Option<usize> {
        trace!("Trying to peek ahead {peek} characters");
        let new_peek;
        if self.peek_position + peek > self.input.len() {
            trace!("peek overflow, trimming peek");
            self.peek_position = self.input.len();
            None
        } else {
            new_peek = self.peek_position + peek;
            self.peek_position = new_peek;
            Some(self.peek_position - self.position)
        }
    }

    // Returns the number of whitespace characters that can be skipped
    fn scan_past_ws(&mut self) -> Span<'a> {
        assert_eq!(
            self.peek_position, self.position,
            "cannot seek past WS unless peek position is committed"
        );
        let mut last_good_peek = None;
        for _ in 0..self.peek_rem_chars() {
            last_good_peek = Some(self.peek_chars());
            self.st_peek(1);
            let span = self.mk_span();
            if span.is_empty() {
                // we must be at EOI
                return span;
            }
            // we now know we have atleast 1 character
            let char = span.str().chars().last().unwrap();
            if !char.is_whitespace() {
                break;
            }
        }
        self.reset_peek();
        if let Some(last_good_peek) = last_good_peek {
            self.st_peek(last_good_peek);
        }
        let span = self.mk_span();
        trace!("got whitespace span: {span:?}");
        self.reset_peek();
        span
    }

    pub fn skip_ws(&mut self) {
        let ws = self.scan_past_ws();
        trace!("skipping {} characters of whitespace", ws.len());
        if !ws.is_empty() {
            self.st_peek(ws.len());
        }
        self.commit_peek();
    }

    // peeks at minimum 1 character to check if given literal matches
    // the input. If yes, check full match and return true if there is still a match
    fn lookahead_peek(&mut self, m: &str) -> Option<Span<'a>> {
        self.reset_peek();
        assert!(!m.is_empty(), "LAP str must be atleast 1 character");
        if self.st_peek(1).is_none() {
            trace!("peek at EOI");
            // EOI, return from scan
            None
        } else if self.mk_span() == m[0..1] {
            let st_peek = self.st_peek(m.len() - 1);
            if st_peek == Some(m.len()) {
                trace!("matched on {m:?}");
                let span = self.mk_span();
                if span == m {
                    Some(span)
                } else {
                    None
                }
            } else {
                trace!(
                    "late mismatch on {m:?}; {:?} != {:?}",
                    st_peek,
                    Some(m.len())
                );
                None
            }
        } else {
            trace!("early mismatch on {m:?}");
            None
        }
    }

    //#[instrument]
    fn commit_peek(&mut self) {
        self.position = self.peek_position;
    }

    fn reset_peek(&mut self) {
        self.peek_position = self.position;
    }

    //#[instrument]
    pub fn scan_match_lit(&mut self, token: Token) -> Option<TokenSpan<'a>> {
        debug!(
            "attempting to find {token:?} in input from {}",
            self.position
        );
        if self.position == self.peek_position && self.position == self.input.len() {
            return Some(TokenSpan::from_span(Token::EOI, self.mk_span()));
        }
        if let Some(substitutes) = token.token_descend() {
            trace!("token {token:?} is replaced by {substitutes:?}");
            for substitute in substitutes {
                if let Some(substitute) = self.scan_match_lit(*substitute) {
                    let real_span = substitute.clone().with_token(token);
                    trace!("substitute {substitute:?} => real {real_span:?}");
                    return Some(substitute);
                }
            }
        }
        if let Some(matches) = token.token_lit() {
            for m in matches {
                if let Some(span) = self.lookahead_peek(m) {
                    self.commit_peek();
                    trace!("matched {token:?}{:?}", span);
                    return Some(TokenSpan::from_span(token, span));
                }
            }
            // no match in stream for given token matches
            return None;
        }
        if let Some(regex) = token.token_regex() {
            if let Some(caps) = regex.captures(&self.input[self.position..]) {
                let capt = caps.get(1).unwrap();
                self.peek_position = capt.end() + self.position;
                assert_eq!(capt.start(), 0, "capture not at start, invariant violation");
                let span = self.mk_span();
                trace!("regex captured span {span:?}");
                return Some(TokenSpan::from_span(token, span));
            } else {
                return None;
                // no match in regex
            }
        }

        trace!("token still not matched, must be another token");
        None
    }

    fn peek_rem_chars(&self) -> usize {
        self.input.len() - self.peek_position
    }

    fn peek_chars(&self) -> usize {
        self.peek_position - self.position
    }

    pub fn root_span(&self) -> TokenSpan<'a> {
        TokenSpan::new(self.input, 0..self.input.len(), Token::ROOT)
    }
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'a, 'bump>
where
    'a: 'bump,
{
    token_spans: Vec<'bump, TokenSpan<'a>>,
    scanner: Scanner<'a>,
}

impl<'a, 'bump> Tokenizer<'a, 'bump>
where
    'a: 'bump,
{
    pub fn new(alloc: &'bump bumpalo::Bump, input: &'a str) -> Tokenizer<'a, 'bump> {
        let data: Vec<'bump, TokenSpan<'a>> = Vec::with_capacity_in(input.len(), alloc);
        Tokenizer {
            token_spans: data,
            scanner: Scanner::new(input),
        }
    }
    /// Consume the next token from the stream
    ///
    /// Returns true unless the token was EOI
    ///
    /// Returns Err(()) when no token matched the rest of the string
    pub fn consume_token(&mut self) -> Result<'bump, bool> {
        let last_token = match self.token_spans.last() {
            Some(t) => t,
            None => {
                // no tokens in yet, produce a root span
                let span = self.scanner.root_span();
                self.token_spans.push(span);
                return Ok(true);
            }
        };
        let possible_next_tokens = last_token.token().next_sequence();
        for possible_next_token in possible_next_tokens {
            if let Some(span) = self.scanner.scan_match_lit(*possible_next_token) {
                let token = span.token();
                self.scanner.commit_peek();
                trace!("committing token {span:?}");
                self.token_spans.push(span);
                self.scanner.skip_ws();
                return Ok(token != Token::EOI);
            }
        }
        self.token_spans
            .push(TokenSpan::from_span(Token::EOI, self.scanner.mk_span()));
        Err(Error::ExpectedTokensNotFound(possible_next_tokens.iter().cloned().collect_in(self.token_spans.bump())))
    }

    /// Consume stream until EOI
    pub fn consume_all(&mut self) -> Result<'bump, Vec<'bump, TokenSpan<'bump>>>
    where
        'a: 'bump,
    {
        while match self.consume_token() {
            Ok(v) => v,
            Err(e) => {
                match e {
                    Error::ExpectedTokensNotFound(e) => {
                        return Err(Error::ExpectedDifferentTokens(
                            e,
                            self.token_spans.last().unwrap().clone(),
                        ))
                    }
                    _ => unreachable!("no other valid errors here"),
                };
            }
        } {}
        Ok(self.token_spans.clone())
    }

    pub fn tokens(&self) -> Vec<'bump, TokenSpan<'bump>> {
        self.token_spans.clone()
    }
}

impl<'a, 'bump> ITokenizer<'a, 'bump> for Tokenizer<'a, 'bump> {
    fn new(alloc: &'bump bumpalo::Bump, input: &'a str) -> Self {
        Self::new(alloc, input)
    }

    fn token_spans(
        mut self,
    ) -> std::result::Result<
        bumpalo::collections::Vec<'bump, TokenSpan<'bump>>,
        crate::errors::Error<'bump>,
    > {
        self.consume_all()?;
        Ok(self.token_spans)
    }

    fn tokens(
        mut self,
    ) -> std::result::Result<bumpalo::collections::Vec<'bump, Token>, crate::errors::Error<'bump>>
    {
        self.consume_all()?;
        Ok(crate::tokenspan_to_token(&self.token_spans))
    }
}
