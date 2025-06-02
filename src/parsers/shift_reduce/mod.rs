use std::rc::Rc;

use crate::ast::{ApplyOp, CombOp, Comp};
use crate::errors;
use crate::tokens::Token;
use crate::{ast::Expr, span::TokenSpan};

use super::{IParserFactory, IParser};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenOrExpr {
    Expr(Expr),
    Token(TokenSpan),
}

impl Default for TokenOrExpr {
    fn default() -> Self {
        Self::Token(TokenSpan::empty())
    }
}

impl TokenOrExpr {
    #[track_caller]
    pub fn assert_expr(self) -> Expr {
        match self {
            TokenOrExpr::Expr(v) => v,
            TokenOrExpr::Token(_) => unreachable!("token in asserted expr"),
        }
    }
}

inventory::submit! { crate::parsers::Parser::new::<ShiftReduceFactory>("shift_reduce") }

struct ShiftReduceFactory;

impl IParserFactory for ShiftReduceFactory {
    fn init() -> Box<dyn IParserFactory> where Self: Sized {
        Box::new(Self)
    }

    fn new(&self, mut tokenizer: Box<dyn crate::tokenizers::ITokenizer>) -> errors::Result<Box<dyn super::IParser>> {
        Ok(Box::new(ShiftReduce::new(tokenizer.token_spans()?)))
    }
}

struct ShiftReduce {
    input: Vec<TokenSpan>,
    position: usize,
    look_ahead: TokenSpan,
    stack: Vec<TokenOrExpr>,
}

impl IParser for ShiftReduce {
    fn produce_tree(&mut self) -> errors::Result<Expr> {
        if let Some(next_look_ahead) = self.next_input() {
            self.look_ahead = next_look_ahead;
        } else {
            panic!("invalid reduce pre-pump");
        }
        loop {
            if self.shift().is_none() {
                if self.reduce().is_none() {
                    break
                }
            }
            while self.reduce().is_some() {
                // reduce more
            }
        }
        assert!(self.stack.len() == 1, "input left over in parser: {:#?}", self.stack);
        Ok(self.stack.pop().unwrap().assert_expr())
    }
    fn produce_token_sequence(&mut self) -> errors::Result<Vec<TokenSpan>> {
        Ok(self.input.clone())
    }
}

impl ShiftReduce {
    pub fn new(input: Vec<TokenSpan>) -> Self {
        Self {
            input,
            position: 0,
            look_ahead: TokenSpan::new(Rc::from(""), 0..0, Token::NONE),
            stack: Vec::new(),
        }
    }

    fn next_input(&mut self) -> Option<TokenSpan> {
        self.position = self.position.checked_add(1).unwrap();
        self.input.get(self.position).cloned()
    }

    /// Returns an expression only stack or panics
    #[track_caller]
    fn expr_stack(&self) -> Vec<Expr> {
        self.stack.iter().filter_map(|x| {
            match x {
                TokenOrExpr::Token(TokenSpan {
                    token: Token::EOI, ..
                }) => None,
                v => Some(v.clone().assert_expr()),
            }
        }).collect()
    }

    /// Shift the next look-ahead into the parser
    ///
    /// Returns false if no shift was possible
    fn shift(&mut self) -> Option<()> {
        //println!("shift  state: {:?}", self.stack);
        let expr = match self.look_ahead.token() {
            Token::TAG => TokenOrExpr::Expr(Expr::Tag(self.look_ahead.str().to_string())),
            Token::EOI => return None,
            _ => TokenOrExpr::Token(self.look_ahead.clone()),
        };
        self.stack.push(expr);
        self.look_ahead = self.next_input()?;
        Some(())
    }

    /// Reduce the input and lookahead
    ///
    /// Returns None if no reduce was done
    fn reduce(&mut self) -> Option<()> {
        //println!("reduce state: {:?}", self.stack);
        let (rest, result) = match &self.stack[..] {
            [rest @ .., TokenOrExpr::Token(TokenSpan {
                token: Token::LPAREN,
                ..
            }), TokenOrExpr::Token(TokenSpan {
                token: Token::RPAREN,
                ..
            })] => (rest, Expr::Empty),

            [rest @ .., TokenOrExpr::Token(TokenSpan {
                token: Token::LPAREN,
                ..
            }), TokenOrExpr::Expr(e), TokenOrExpr::Token(TokenSpan {
                token: Token::RPAREN,
                ..
            })] => (rest, e.clone()),

            [TokenOrExpr::Token(TokenSpan {
                token: Token::LPAREN,
                ..
            }), ref n @ .., TokenOrExpr::Token(TokenSpan {
                token: Token::RPAREN,
                ..
            })] => (self.stack.as_slice(), Expr::Group(n.into_iter().map(|x| x.clone().assert_expr()).collect())),

            [rest @ .., TokenOrExpr::Expr(a), TokenOrExpr::Token(TokenSpan {
                token: Token::AND, ..
            }), TokenOrExpr::Expr(b)]  if *a == *b => (rest, a.clone()),

            [rest @ .., TokenOrExpr::Expr(a), TokenOrExpr::Token(TokenSpan {
                token: Token::AND, ..
            }), TokenOrExpr::Expr(b)] => (rest, Expr::Combine(CombOp::And, vec![a.clone(), b.clone()])),

            [rest @ .., TokenOrExpr::Expr(a), TokenOrExpr::Token(TokenSpan {
                token: Token::OR, ..
            }), TokenOrExpr::Expr(b)] if *a == *b => (rest, a.clone()),

            [rest @ .., TokenOrExpr::Expr(a), TokenOrExpr::Token(TokenSpan {
                token: Token::OR, ..
            }), TokenOrExpr::Expr(b)] => (rest, Expr::Combine(CombOp::Or, vec![a.clone(), b.clone()])),

            [rest @ .., TokenOrExpr::Token(TokenSpan {
                token: Token::NOT, ..
            }), TokenOrExpr::Expr(e) ] => (rest, Expr::Apply(ApplyOp::Not, e.clone().into())),

            [rest@.., TokenOrExpr::Token(
                f @ TokenSpan {
                    token: Token::FIELD,
                    ..
                },
            ), TokenOrExpr::Token(
                c @ TokenSpan {
                    token: Token::RANGE,
                    ..
                },
            ), TokenOrExpr::Token(
                v @ TokenSpan {
                    token:
                        Token::INTEGER
                        | Token::FLOAT
                        | Token::BOOLEAN
                        | Token::IP_CIDR
                        | Token::ABSOLUTE_DATE
                        | Token::RELATIVE_DATE,
                    ..
                },
            )] => (rest, Expr::Comparison(
                    f.str().to_string(),
                    str_to_comp(c.str()),
                    (*v).clone().into(),
                )),

            [.., TokenOrExpr::Token(_c @ TokenSpan {
                token: Token::EOI,
                ..
            })] => {
                if self.stack.len() == 2 {
                    self.stack = vec![self.stack.get(0).unwrap().clone()];
                    return None;
                }
                self.stack = vec![
                    TokenOrExpr::Expr(Expr::Group(self.expr_stack()))
                ];
                return None;
            }

            _ => return None,
        };
        self.stack.truncate(rest.len());
        self.stack.push(TokenOrExpr::Expr(result));
        Some(())
    }
}

fn str_to_comp(a: &str) -> Comp {
    match a {
        "lt:" => Comp::LessThan,
        "lte:" => Comp::LessThanOrEqual,
        "eq:" => Comp::Equal,
        "neq:" => Comp::NotEqual,
        "gt:" => Comp::GreaterThan,
        "gte:" => Comp::GreaterThanOrEqual,
        _ => unreachable!(),
    }
}
