use std::str::FromStr;

use crate::ast::{Expr, CombOp, Value, Comp};
use crate::errors;
use crate::span::TokenSpan;
use crate::tokenizers::ITokenizer;

use super::IParserFactory;

pub type ExprNodeRef = Box<Expr>;

inventory::submit! { super::Parser::new::<ParserFactory>("recdec") }

pub struct ParserFactory;

impl super::IParserFactory for ParserFactory {
    fn init() -> Box<dyn IParserFactory> where Self: Sized {
        Box::new(Self)
    }

    fn new(&self, tokenizer: Box<dyn ITokenizer>) -> errors::Result<Box<dyn super::IParser>> {
        Ok(Box::new(Parser::new(tokenizer)))
    }
}
pub struct Parser
{
    tree: ExprNodeRef,
    tokenizer: Box<dyn ITokenizer>,
}

impl super::IParser for Parser {
    fn produce_tree(&mut self) -> errors::Result<Expr> {
        self.generate_primitive_ast(true)?;
        Ok((*self.tree).clone())
    }
    fn produce_token_sequence(&mut self) -> errors::Result<Vec<TokenSpan>> {
        self.tokenizer.token_spans()
    }
}

impl Parser
{
    pub fn new(tokenizer: Box<dyn ITokenizer>) -> Self {
        Self {
            tree: Box::new(Expr::default()),
            tokenizer,
        }
    }
    /// Parses the token stream into an AST without any optimizations
    ///
    /// If the function is not called, the rest of the parser will operate on
    /// and empty syntax tree.
    #[tracing::instrument(skip(self))]
    pub fn generate_primitive_ast(&mut self, eoi_fold: bool) -> crate::errors::Result<()> {
        let result = self.tokenizer.token_spans()?;
        // when building the AST, we may decend into recursively trying to solve
        // groups or large AND connections. This space allows the parser to save
        // existing progress before diving deeper without having to be actually recursive
        let mut scratch_space: Vec<ExprNodeRef> = Vec::with_capacity(100);
        let mut current: ExprNodeRef = Box::new(Expr::default());
        for (_i, token) in result.iter().enumerate() {
            match token.token() {
                crate::tokens::Token::LPAREN | crate::tokens::Token::ROOT => {
                    scratch_space.push(current);
                    current = Box::new(Expr::Group(Vec::new()));
                }
                crate::tokens::Token::RPAREN => {
                    if let Some(mut up) = scratch_space.pop() {
                        match &mut *up {
                            Expr::Field(_) => todo!(),
                            Expr::Tag(_) => todo!(),
                            Expr::Tags(_) => todo!(),
                            Expr::Apply(_, _) => todo!(),
                            Expr::Comparison(_, _, _) => todo!(),
                            Expr::Combine(_, c) => match *current {
                                Expr::Group(mut j) if j.len() == 1 => c.push(j.pop().unwrap()),
                                Expr::Group(j) if j.is_empty() => (),
                                current => c.push(current),
                            },
                            Expr::Group(g) => match *current {
                                Expr::Group(mut j) if j.len() == 1 => g.push(j.pop().unwrap()),
                                Expr::Group(j) if j.is_empty() => (),
                                Expr::Group(j) if j.is_empty() => {
                                    current = up;
                                    continue
                                },
                                current => g.push(current),
                            },
                            Expr::Empty => unreachable!(),
                        }
                        current = up;
                    } else {
                        panic!("invalid group close");
                    }
                }
                crate::tokens::Token::AND => {
                    match &mut *current {
                        Expr::Field(_) => todo!(),
                        Expr::Tag(_) => todo!(),
                        Expr::Tags(_) => todo!(),
                        Expr::Apply(_, _) => todo!(),
                        Expr::Comparison(_, _, _) => todo!(),
                        Expr::Combine(v, _) if *v != crate::tokens::Token::AND => {
                            scratch_space.push(current);
                            current = Box::new(
                                Expr::Combine(
                                    CombOp::And,
                                    // repop the OR back into the and combinator
                                    vec![*scratch_space.pop().unwrap()],
                                ),
                            );
                        }
                        Expr::Combine(_v, _) => {
                            // skip repated ands
                        }
                        Expr::Group(g) => {
                            if g.len() == 1 {
                                let e = g.pop().unwrap();
                                current = Box::new(
                                    Expr::Combine(
                                        CombOp::And,
                                        vec![e],
                                    ),
                                );
                            } else {
                                current = Box::new(
                                    Expr::Combine(
                                        CombOp::And,
                                        vec![*current],
                                    ),
                                );
                            }
                        },
                        Expr::Empty => unreachable!(),
                    }
                }
                crate::tokens::Token::OR => {
                    match &mut *current {
                        Expr::Field(_) => todo!(),
                        Expr::Tag(_) => todo!(),
                        Expr::Tags(_) => todo!(),
                        Expr::Apply(_, _) => todo!(),
                        Expr::Comparison(_, _, _) => todo!(),
                        Expr::Combine(v, t) if *v != crate::tokens::Token::OR => {
                            let t = t.pop().unwrap();
                            scratch_space.push(current);
                            current = Box::new(
                                Expr::Combine(
                                    CombOp::Or,
                                    // or has lower priority, we do not repopulate, instead grab last one
                                    vec![t],
                                ),
                            );
                        }
                        Expr::Combine(_v, _) => {
                            // skip repated ands
                        }
                        Expr::Group(g) => {
                            if g.len() == 1 {
                                let e = g.pop().unwrap();
                                current = Box::new(
                                    Expr::Combine(
                                        CombOp::Or,
                                        vec![e],
                                    ),
                                );
                            } else {
                                current = Box::new(
                                    Expr::Combine(
                                        CombOp::Or,
                                        vec![*current],
                                    ),
                                );
                            }
                        },
                        Expr::Empty => unreachable!(),
                    }
                }
                crate::tokens::Token::NOT => todo!("not"),
                crate::tokens::Token::BOOST => todo!("boost"),
                crate::tokens::Token::FUZZ => todo!("fuzz"),
                crate::tokens::Token::QUOTE => todo!("quote"),
                crate::tokens::Token::FLOAT => todo!("float"),
                crate::tokens::Token::INTEGER => {
                    match &mut *current {
                        Expr::Field(_) => todo!("integer <- field"),
                        Expr::Tag(_) => todo!("integer <- tag"),
                        Expr::Tags(_) => todo!("integer <- taglist"),
                        Expr::Apply(_, _) => todo!("integer <- apply"),
                        Expr::Comparison(_, _, v) => {
                            *v = Value::Integer(token.str().parse()?);
                            let done_comp = current;
                            current = scratch_space.pop().unwrap();
                            match &mut *current {
                                Expr::Field(_) => todo!(),
                                Expr::Tag(_) => todo!(),
                                Expr::Tags(_) => todo!(),
                                Expr::Apply(_, _) => todo!(),
                                Expr::Comparison(_, _, _) => todo!(),
                                Expr::Combine(_, f) => {
                                    f.push(*done_comp);
                                },
                                Expr::Group(g) => {
                                    g.push(*done_comp)
                                },
                                Expr::Empty => unreachable!(),
                            }
                        },
                        Expr::Combine(_, _) => todo!("integer <- comb"),
                        Expr::Group(_) => todo!("integer <- group"),
                        Expr::Empty => unreachable!(),
                    }
                },
                crate::tokens::Token::BOOLEAN => todo!("boolean"),
                crate::tokens::Token::IP_CIDR => todo!("ip_cidr"),
                crate::tokens::Token::ABSOLUTE_DATE => todo!("abs_date"),
                crate::tokens::Token::RELATIVE_DATE => todo!("rel_date"),
                crate::tokens::Token::TAG => match &mut *current {
                    Expr::Field(_) => todo!(),
                    Expr::Tag(_) => todo!(),
                    Expr::Tags(_) => todo!(),
                    Expr::Apply(_, _) => todo!(),
                    Expr::Comparison(_, _, _) => todo!(),
                    Expr::Combine(_v, f) => {
                        let token_str =
                            String::from_str(token.str()).unwrap();
                        f.push(Expr::Tag(token_str));
                    }
                    Expr::Group(g) => {
                        let token_str =
                            String::from_str(token.str()).unwrap();
                        g.push(Expr::Tag(token_str));
                    },
                    Expr::Empty => unreachable!(),
                },
                crate::tokens::Token::FIELD => match &mut *current {
                    Expr::Field(_) => todo!(),
                    Expr::Tag(_) => todo!(),
                    Expr::Tags(_) => todo!(),
                    Expr::Apply(_, _) => todo!(),
                    Expr::Comparison(_, _, _) => todo!(),
                    Expr::Combine(_v, f) => {
                        let token_str =
                            String::from_str(token.str()).unwrap();
                        f.push(Expr::Field(token_str));
                    }
                    Expr::Group(g) => {
                        let token_str =
                            String::from_str(token.str()).unwrap();
                        g.push(Expr::Field(token_str));
                    },
                    Expr::Empty => unreachable!(),
                },
                crate::tokens::Token::RANGE => {
                    match &mut *current {
                        Expr::Field(f) => {
                            current = Box::new(Expr::Comparison(f.to_owned(), match token.str() {
                                ".gte" | "gte:" => Comp::GreaterThanOrEqual,
                                _ => unreachable!(),
                            }, Value::Undefined));
                        },
                        Expr::Tag(_) => todo!("invalid tag"),
                        Expr::Tags(_) => todo!("invalid taglist"),
                        Expr::Apply(_, _) => todo!("invalid apply"),
                        Expr::Comparison(_, _, _) => todo!("invalid comparison"),
                        Expr::Combine(_, v) => {
                            let last = v.pop().unwrap();
                            let last = match last {
                                Expr::Field(f) => f,
                                Expr::Tag(t) => t,
                                Expr::Tags(_) => todo!("field <- taglist -> combine"),
                                Expr::Apply(_, _) => todo!("field <- apply -> combine"),
                                Expr::Comparison(_, _, _) => todo!("field <- comp -> combine"),
                                Expr::Combine(_, _) => todo!("field <- combine -> combine"),
                                Expr::Group(_) => todo!("field <- group -> combine"),
                                Expr::Empty => unreachable!(),
                            };
                            scratch_space.push(current);
                            current = Box::new(Expr::Comparison(last, match token.str() {
                                ".gte" | "gte:" => Comp::GreaterThanOrEqual,
                                ".lt" | "lt:" => Comp::LessThan,
                                _ => unreachable!(),
                            }, Value::Undefined))
                        },
                        Expr::Group(_) => todo!("invalid group: {:#?}", current),
                        Expr::Empty => unreachable!(),
                    }
                },
                crate::tokens::Token::NEWLINE => todo!("newline"),
                crate::tokens::Token::EOI => {
                    if !eoi_fold {
                        while let Some(mut scratch) = scratch_space.pop() {
                            match &mut *scratch {
                                Expr::Field(_) => todo!("not valid field at SSP clean"),
                                Expr::Tag(_) => todo!("not valid tag at SSP clean"),
                                Expr::Tags(_) => todo!("not valid taglist at SSP clean"),
                                Expr::Apply(_, _) => todo!("invalid apply at SSP clean"),
                                Expr::Comparison(_, _, _) => todo!("invalid comp at SSP clean"),
                                Expr::Combine(_, s) => {
                                    s.push(*current);
                                    current = scratch;
                                },
                                Expr::Group(g) => {
                                    g.push(*current);
                                    current = scratch;
                                },
                                Expr::Empty => unreachable!(),
                            }
                        }
                        // fold down the top level
                        let mut dirty = true;
                        while dirty {
                            dirty = false;
                            match &mut *current {
                                Expr::Group(g) => {
                                    if g.len() == 1 {
                                        current = Box::new(g.pop().unwrap());
                                        dirty = true;
                                    }
                                },
                                _ => (),
                            }
                        }
                    }
                    self.tree = current;
                    return Ok(());
                }
                crate::tokens::Token::UNQUOTED_TERM => todo!("unquoted term"),
                crate::tokens::Token::QUOTED_TERM => todo!("quoted term"),
                crate::tokens::Token::TERM => todo!("term"),
                crate::tokens::Token::EXPRESSION => todo!("expression"),
                crate::tokens::Token::COMBINATOR => todo!("combinator"),
                crate::tokens::Token::GROUP => todo!("group"),
                crate::tokens::Token::SEARCH_TERM => todo!("search term"),
                crate::tokens::Token::NONE => todo!("none"),
            }
        }
        unreachable!("hit end of token stream without reading EOI")
    }
}
