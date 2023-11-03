use bumpalo::collections::CollectIn;
use parser::Parser;
use ast::Expr;
use span::TokenSpan;
use tokens::Token;

pub mod errors;
pub mod parser;
pub mod span;
pub mod tokens;
pub mod tokenizers;
pub mod ast;

pub fn tokenspan_to_token<'a, 'bump>(
    token_spans: &bumpalo::collections::Vec<'bump, TokenSpan<'a>>,
) -> bumpalo::collections::Vec<'bump, Token>
where
    'a: 'bump,
{
    let alloc = token_spans.bump();
    token_spans.iter().map(|x| x.token()).collect_in(alloc)
}

pub trait ITokenizer<'a, 'bump>: std::fmt::Debug + Clone {
    fn new(alloc: &'bump bumpalo::Bump, input: &'a str) -> Self;
    fn token_spans(
        self,
    ) -> Result<bumpalo::collections::Vec<'bump, TokenSpan<'bump>>, crate::errors::Error<'bump>>;
    fn tokens(self) -> Result<bumpalo::collections::Vec<'bump, Token>, crate::errors::Error<'bump>> {
        let token_spans = self.token_spans()?;
        Ok(tokenspan_to_token(&token_spans))
    }
}

pub enum Tokenizer {
    FSM,
    DLLN,
}

#[tracing::instrument(skip(alloc))]
pub fn parse_string<'a, 'bump, T: ITokenizer<'a, 'bump> + 'a>(
    alloc: &'bump bumpalo::Bump,
    inp: &'a str,
    eoi_fold: bool,
) -> Expr
where
    'a: 'bump,
{
    let mut parser = Parser::<T>::new(inp, alloc);
    match parser.generate_primitive_ast(eoi_fold) {
        Ok(_) => (),
        Err(e) => panic!("could not parse: {:?}", e),
    }
    parser.into_tree()
}
