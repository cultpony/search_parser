use crate::{span::TokenSpan, tokens::Token};

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

#[derive(Debug, thiserror::Error)]
pub enum Error<'a> {
    #[error("Expected {} but could not find these tokens", itertools::join(_0.iter().map(|x| x.name()), ", "))]
    ExpectedTokensNotFound(bumpalo::collections::Vec<'a, Token>),
    #[error("Expected {} but got {}", itertools::join(_0.iter().map(|x| x.name()), ", "), _1.token().name())]
    ExpectedDifferentTokens(bumpalo::collections::Vec<'a, Token>, TokenSpan<'a>),
    #[error("Could not parse integer: {0:?}")]
    ParseIntError(#[from] std::num::ParseIntError),
}
