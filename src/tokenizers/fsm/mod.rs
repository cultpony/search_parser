use std::rc::Rc;

use tracing::trace;

use crate::{span::TokenSpan, tokens::Token};

use self::state_machine::StateMachine;

use super::{ITokenizerFactory, ITokenizer};

pub mod comp;
pub mod data;
pub mod infix;
pub mod prefix;
pub mod state_machine;
pub mod token_and_field;

inventory::submit! { crate::tokenizers::Tokenizer::new::<FSMFactory>("fsm") }

#[derive(Clone, Copy, Debug)]
pub struct FSMFactory;

impl ITokenizerFactory for FSMFactory {
    fn init() -> Box<dyn ITokenizerFactory> {
        Box::new(Self)
    }

    fn new(&self, input: std::rc::Rc<str>) -> Box<dyn ITokenizer> {
        Box::new(Tokenizer::new(input))
    }
}

pub trait FSM: Default + Copy + Clone + PartialEq + Eq + FSMStateMatcher {
    type NextStateType: FSM + Default + Copy + Clone + PartialEq + Eq + std::fmt::Debug;
    fn start() -> Self::NextStateType {
        Self::NextStateType::default()
    }
    fn next_states(self) -> &'static [Self::NextStateType];

    fn to_token(self) -> crate::tokens::Token;
}

pub trait FSMStateMatcher: Copy + Clone + PartialEq + Eq {
    /// Returns the number of characters to consume when it matches
    /// the current input position forward
    ///
    /// If it returns none, input does not match
    fn matches(self, inp: &str) -> Option<u8>;
    /// The maximum number of characters this matcher will consume.
    ///
    /// The input string will be clamped to this number of characters.
    ///
    /// If None is returned, the match is unbounded
    fn maximum_bound(self) -> Option<u8> {
        None
    }
    /// If true is returned, the parser will skip any whitespace after a match
    /// until the first non-whitespace character
    fn trailing_whitespace_trim(self) -> bool {
        true
    }
}

#[derive(Clone, Debug)]
pub struct Tokenizer {
    pub inp: Rc<str>,
    pub position: usize,
    pub state: StateMachine,
}

impl Tokenizer {
    pub fn new(inp: Rc<str>) -> Self {
        Self {
            inp,
            position: 0,
            state: StateMachine::default(),
        }
    }
    /// Tries to get the next token in the input or returns None if no possible
    /// token can match the remainder of the input
    #[tracing::instrument(skip_all)]
    pub fn step(&mut self) -> Option<TokenSpan> {
        // Check if we're at the end and escape hatch out
        if self.state == StateMachine::EndOfInput {
            return None;
        }
        for next in self.state.next_states() {
            trace!("Attempting to transition: {:?} -> {next:?}", self.state);
            if let Some(chars) = next.matches(&self.inp[self.position..]) {
                let mut span = TokenSpan::new(
                    self.inp.clone(),
                    self.position..self.position + (chars as usize),
                    next.to_token(),
                );
                if next.trailing_whitespace_trim() {
                    span.trim_end_whitespace();
                }
                trace!(
                    " -> Transition from {:?} to {next:?} with {chars:?} characters: {:?}",
                    self.state,
                    &self.inp[self.position..self.position + (chars as usize)]
                );
                // move the input stream
                self.position += chars as usize;
                // forward space through input whitespace at end of token
                let fsws = self
                    .inp
                    .get(self.position..)
                    .unwrap_or_default()
                    .chars()
                    .take_while(|x| x.is_whitespace())
                    .count();
                trace!(" <- Forward Space White Space : {fsws}");
                self.position += fsws;
                // update the state machine
                self.state = *next;
                return Some(span);
            } else {
                // no match, try the next one
            }
        }
        trace!("Could not transition to EoI but could not match any transition");
        trace!("Remainder: {:?}", &self.inp[self.position..]);
        None
    }

    #[tracing::instrument(skip(self))]
    pub fn scan_until_none<'bump>(
        &mut self,
    ) -> Vec<TokenSpan>
    {
        let mut out = Vec::new();

        // insert the start state token
        out.push(TokenSpan::new(self.inp.clone(), 0..self.inp.len(), Token::ROOT));

        while let Some(token_span) = self.step() {
            out.push(token_span)
        }

        out
    }
}

impl ITokenizer for Tokenizer {
    fn new(input: Rc<str>) -> Box<dyn ITokenizer> {
        Box::new(Tokenizer::new(input))
    }

    fn token_spans(
        &mut self,
    ) -> Result<Vec<TokenSpan>, crate::errors::Error> {
        let res = self.scan_until_none();
        if self.state != StateMachine::EndOfInput {
            Err(crate::errors::Error::ExpectedDifferentTokens(
                self.state.next_states().into_iter()
                    .map(|x| x.to_token())
                    .collect(),
                TokenSpan::new(self.inp.clone(), self.position..self.inp.len(), Token::EOI)
            ))
        } else {
            Ok(res)
        }
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::{span::TokenSpan, tokens::Token, tokenizers::tokenspan_to_token};

    use super::Tokenizer;
    use tracing::trace;

    #[test]
    #[tracing_test::traced_test]
    pub fn test_empty() {
        let input: Rc<str> = Rc::from("");
        let token_spans = Tokenizer::new(input.clone()).scan_until_none();
        let tokens = tokenspan_to_token(&token_spans);

        assert_eq!(
            vec![Token::ROOT, Token::EOI],
            tokens,
            "Empty tokens match empty token sequence"
        );

        assert_eq!(
            vec![
                TokenSpan::new(input.clone(), 0..0, Token::ROOT),
                TokenSpan::new(input.clone(), 0..0, Token::EOI),
            ],
            token_spans,
            "Token Span positions correct"
        );
    }

    #[test]
    #[tracing_test::traced_test]
    pub fn test_tag() {
        let input: Rc<str> = Rc::from("hello");
        let token_spans = Tokenizer::new(input.clone()).scan_until_none();
        let tokens = tokenspan_to_token(&token_spans);

        assert_eq!(
            vec![Token::ROOT, Token::TAG, Token::EOI],
            tokens,
            "Empty tokens match empty token sequence"
        );

        assert_eq!(
            vec![
                TokenSpan::new(input.clone(), 0..5, Token::ROOT),
                TokenSpan::new(input.clone(), 0..5, Token::TAG),
                TokenSpan::new(input.clone(), 5..5, Token::EOI),
            ],
            token_spans,
            "Token Span positions correct"
        );
    }

    #[test]
    #[tracing_test::traced_test]
    pub fn test_field() {
        let input: Rc<str> = Rc::from("hello.gte:10");
        let token_spans = Tokenizer::new(input.clone()).scan_until_none();
        let tokens = tokenspan_to_token(&token_spans);

        assert_eq!(
            vec![
              Token::ROOT,
              Token::FIELD,
              Token::RANGE,
              Token::INTEGER,
              Token::EOI
            ],
            tokens,
            "Empty tokens match empty token sequence"
        );

        assert_eq!(
            vec![
                TokenSpan::new(input.clone(), 0..12, Token::ROOT),
                TokenSpan::new(input.clone(), 0..6, Token::FIELD),
                TokenSpan::new(input.clone(), 6..10, Token::RANGE),
                TokenSpan::new(input.clone(), 10..12, Token::INTEGER),
                TokenSpan::new(input.clone(), 12..12, Token::EOI),
            ],
            token_spans,
            "Token Span positions correct"
        );
    }

    #[test]
    #[tracing_test::traced_test]
    pub fn test_complex_expr() {
        let input: Rc<str> = Rc::from("(((field.gte:1000)AND data.neq:20)||bla.gte:100.2,tag),test.lte:-10,tag");
        let token_spans = Tokenizer::new(input.clone()).scan_until_none();
        trace!("{token_spans:#?}");
        let tokens = tokenspan_to_token(&token_spans);

        assert_eq!(
            vec![
              Token::ROOT,
              Token::LPAREN,
              Token::LPAREN,
              Token::LPAREN,
              Token::FIELD,
              Token::RANGE,
              Token::INTEGER,
              Token::RPAREN,
              Token::AND,
              Token::FIELD,
              Token::RANGE,
              Token::INTEGER,
              Token::RPAREN,
              Token::OR,
              Token::FIELD,
              Token::RANGE,
              Token::FLOAT,
              Token::AND,
              Token::TAG,
              Token::RPAREN,
              Token::AND,
              Token::FIELD,
              Token::RANGE,
              Token::INTEGER,
              Token::AND,
              Token::TAG,
              Token::EOI
            ],
            tokens
        );
    }
}
