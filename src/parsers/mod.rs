use crate::{errors, tokenizers::ITokenizer, ast::Expr, span::TokenSpan};

mod shift_reduce;
mod recdec;

pub trait IParserFactory {
    fn init() -> Box<dyn IParserFactory> where Self: Sized;
    fn new(&self, tokenizer: Box<dyn ITokenizer>) -> errors::Result<Box<dyn IParser>>;
}

pub trait IParser {
    fn produce_tree(&mut self) -> errors::Result<Expr>;
    fn produce_token_sequence(&mut self) -> errors::Result<Vec<TokenSpan>>;
}

pub struct Parser {
    pub name: &'static str,
    pub imp: &'static (dyn Send + Sync + Fn() -> Box<dyn IParserFactory>),
}

impl Parser {
    pub const fn new<I: IParserFactory>(name: &'static str) -> Self {
        Self {
            name,
            imp: &|| I::init(),
        }
    }
}

inventory::collect!(Parser);

pub fn parsers() -> Vec<String> {
    inventory::iter::<Parser>().map(|x| x.name.to_string()).collect()
}

pub fn parser(name: &str, tok: Box<dyn crate::tokenizers::ITokenizer>) -> errors::Result<Box<dyn IParser>> {
    for par in inventory::iter::<Parser> {
        if par.name == name {
            return Ok((par.imp)().new(tok)?)
        }
    }
    Err(errors::Error::UnknownTokenizer(name.to_string()))
}