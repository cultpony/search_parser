use std::rc::Rc;

use crate::{span::TokenSpan, tokens::Token, errors};


mod fsm;

fn tokenspan_to_token(
    token_spans: &Vec<TokenSpan>,
) -> Vec<Token>
{
    token_spans.iter().map(|x| x.token()).collect()
}

pub trait ITokenizer: std::fmt::Debug {
    fn new(input: Rc<str>) -> Box<dyn ITokenizer> where Self: Sized;
    fn token_spans(
        &mut self,
    ) -> Result<Vec<TokenSpan>, crate::errors::Error>;
    fn tokens(&mut self) -> Result<Vec<Token>, crate::errors::Error> {
        let token_spans = self.token_spans()?;
        Ok(tokenspan_to_token(&token_spans))
    }
}

pub trait ITokenizerFactory: std::fmt::Debug {
    fn init() -> Box<dyn ITokenizerFactory> where Self: Sized;
    fn new(&self, input: Rc<str>) -> Box<dyn ITokenizer>;
}

pub struct Tokenizer {
    pub name: &'static str,
    pub imp: &'static (dyn Send + Sync + Fn() -> Box<dyn ITokenizerFactory>),
}

impl Tokenizer {
    pub const fn new<I: ITokenizerFactory>(name: &'static str) -> Self {
        Self {
            name,
            imp: &|| I::init(),
        }
    }
}

inventory::collect!(Tokenizer);

pub fn tokenizers() -> Vec<String> {
    inventory::iter::<Tokenizer>().map(|x| x.name.to_string()).collect()
}

pub fn tokenizer(name: &str, inp: &str) -> errors::Result<Box<dyn ITokenizer>> {
    for tok in inventory::iter::<Tokenizer> {
        if tok.name == name {
            return Ok((tok.imp)().new(Rc::from(inp)))
        }
    }
    Err(errors::Error::UnknownTokenizer(name.to_string()))
}
