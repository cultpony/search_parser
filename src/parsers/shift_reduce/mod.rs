use search_parser::ast::{CombOp, Comp};
use search_parser::tokens::Token;
use search_parser::{ast::Expr, span::TokenSpan};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenOrExpr<'a> {
    Expr(Expr),
    Token(TokenSpan<'a>),
}

impl<'a> Default for TokenOrExpr<'a> {
    fn default() -> Self {
        Self::Token(TokenSpan::empty())
    }
}

impl<'a> TokenOrExpr<'a> {
    pub fn assert_expr(self) -> Expr {
        match self {
            TokenOrExpr::Expr(v) => v,
            TokenOrExpr::Token(e) => unreachable!("token in asserted expr"),
        }
    }
}

pub struct ShiftReduce<'a> {
    input: Vec<TokenSpan<'a>>,
    position: usize,
    look_ahead: TokenSpan<'a>,
    stack: Vec<TokenOrExpr<'a>>,
}

impl<'a> ShiftReduce<'a> {
    pub fn new(input: Vec<TokenSpan<'a>>) -> Self {
        Self {
            input,
            position: 0,
            look_ahead: TokenSpan::new("", 0..0, Token::NONE),
            stack: Vec::new(),
        }
    }

    fn next_input(&mut self) -> Option<TokenSpan<'a>> {
        self.position = self.position.checked_add(1).unwrap();
        self.input.get(self.position).copied()
    }

    /// Shift the next look-ahead into the parser
    ///
    /// Returns false if no shift was possible
    fn shift(&mut self) -> Option<()> {
        let expr = match self.look_ahead.token() {
            Token::FIELD => TokenOrExpr::Expr(Expr::Field(self.look_ahead.str().to_string())),
            Token::TAG => TokenOrExpr::Expr(Expr::Tag(self.look_ahead.str().to_string())),
            _ => TokenOrExpr::Token(self.look_ahead),
        };
        self.stack.push(expr);
        self.look_ahead = self.next_input()?;
        Some(())
    }

    /// Reduce the input and lookahead
    ///
    /// Returns None if no reduce was done
    fn reduce(&mut self) -> Option<()> {
        let result = match &self.stack[..] {
            [.., TokenOrExpr::Token(TokenSpan {
                token: Token::LPAREN,
                ..
            }), TokenOrExpr::Token(TokenSpan {
                token: Token::RPAREN,
                ..
            })] => Expr::Empty,

            [TokenOrExpr::Token(TokenSpan {
                token: Token::LPAREN,
                ..
            }), TokenOrExpr::Expr(e), TokenOrExpr::Token(TokenSpan {
                token: Token::RPAREN,
                ..
            })] => e.clone(),

            [TokenOrExpr::Token(TokenSpan {
                token: Token::LPAREN,
                ..
            }), ref n @ .., TokenOrExpr::Token(TokenSpan {
                token: Token::RPAREN,
                ..
            })] => Expr::Group(n.into_iter().map(|x| x.clone().assert_expr()).collect()),

            [.., TokenOrExpr::Expr(a), TokenOrExpr::Token(TokenSpan {
                token: Token::AND, ..
            }), TokenOrExpr::Expr(b)]  if *a == *b => a.clone(),

            [.., TokenOrExpr::Expr(a), TokenOrExpr::Token(TokenSpan {
                token: Token::AND, ..
            }), TokenOrExpr::Expr(b)] => Expr::Combine(CombOp::And, vec![a.clone(), b.clone()]),

            [.., TokenOrExpr::Expr(a), TokenOrExpr::Token(TokenSpan {
                token: Token::OR, ..
            }), TokenOrExpr::Expr(b)] if *a == *b => a.clone(),

            [.., TokenOrExpr::Expr(a), TokenOrExpr::Token(TokenSpan {
                token: Token::OR, ..
            }), TokenOrExpr::Expr(b)] => Expr::Combine(CombOp::Or, vec![a.clone(), b.clone()]),

            [.., TokenOrExpr::Token(
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
            )] => Expr::Comparison(
                f.str().to_owned(),
                str_to_comp(c.str()),
                (*v).into(),
            ),

            _ => return None,
        };
        self.stack = vec![TokenOrExpr::Expr(result)];
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
