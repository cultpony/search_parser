use crate::{span::TokenSpan, tokens::Token};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Expected {} but could not find these tokens", itertools::join(_0.iter().map(|x| x.name()), ", "))]
    ExpectedTokensNotFound(Vec<Token>),
    #[error("Expected {} but got {}", itertools::join(_0.iter().map(|x| x.name()), ", "), _1.token().name())]
    ExpectedDifferentTokens(Vec<Token>, TokenSpan),
    #[error("Could not parse integer: {0:?}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("unknown tokenizer {0:?}, available tokenizers: {}", crate::tokenizers().join(", "))]
    UnknownTokenizer(String),
    #[error("unknown parser {0:?}, available parsers: {}", crate::parsers().join(", "))]
    UnknownParser(String),
    #[error("unknown transformer {0:?}, available transformers: {}", crate::transformers().join(", "))]
    UnknownTransformer(String),
    #[error("could not transform to json: {0:?}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("io error: {0:?}")]
    IOError(#[from] std::io::Error),
}
