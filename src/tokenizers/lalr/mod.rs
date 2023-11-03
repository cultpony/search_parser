use lalrpop_util::lalrpop_mod;

use crate::{ast::Expr, ITokenizer};

//lalrpop_mod!(pub syn, "/tokenizers/lalr/syn.rs");

pub fn parse<'a>(s: &'a str) -> Expr {
    todo!()
    /*let parser = syn::ExprParser::new();
    parser.parse(s).unwrap()*/
}