mod types;
use crate::scanner::{Token, TokenType};
pub use types::*;

pub struct Parser<'a> {
    input: &'a Vec<Token>,
    index: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &Vec<Token>) -> Parser {
        Parser {
            input: input.into(),
            index: 0,
        }
    }

    fn token(&self) -> &Token {
        &self.input[self.index]
    }

    fn advance(&mut self) -> bool {
        self.index += 1;
        return self.index <= self.input.len();
    }

    pub fn expr(&mut self) -> Box<Expr> {
        return self.add_sub_term();
    }

    fn add_sub_term(&mut self) -> Box<Expr> {
        let lhs = self.mul_divide_mod_term();
        match self.token().token_type {
            TokenType::Plus | TokenType::Minus => {
                let operand = self.token().clone();
                self.advance();
                Box::new(Expr::BinOp(BinOp {
                    op: operand,
                    lhs,
                    rhs: self.mul_divide_mod_term(),
                }))
            }
            TokenType::EOF => lhs,
            _ => unreachable!(),
        }
    }

    fn mul_divide_mod_term(&mut self) -> Box<Expr> {
        let lhs = self.num_term();
        match self.token().token_type {
            TokenType::Mult | TokenType::Div | TokenType::Mod => {
                let operand = self.token().clone();
                self.advance();
                Box::new(Expr::BinOp(BinOp {
                    op: operand,
                    lhs,
                    rhs: self.num_term(),
                }))
            }
            _ => lhs,
        }
    }

    fn num_term(&mut self) -> Box<Expr> {
        let node = match self.token().token_type {
            TokenType::Float(f) => Box::new(Expr::Float(f)),
            TokenType::Int(i) => Box::new(Expr::Int(i)),
            // TODO: check for paren errors
            TokenType::OpenParen => {
                // eat left paren
                self.advance();
                let node = self.expr();
                // eat right paren
                self.advance();
                node
            }
            // TODO: check for bracket errors
            TokenType::OpenBracket => {
                // eat left bracket
                self.advance();
                let node = self.expr();
                // eat right bracket
                self.advance();
                node
            }
            _ => {
                println!("{}", self.token());
                unreachable!()
            }
        };
        self.advance();
        node
    }
}

#[cfg(test)]
mod tests {
    // Tests the Parser's output against a humanized string representation of the expected
    // expressions.
    // See [Expr]'s impl of Display for more details.
    // [Expr]: crate::parser::Expr
    macro_rules! parser_tests {
        ($($name:ident: $program:expr, $format_str:expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::scanner::Scanner;
                use crate::parser::Parser;

                let mut scanner = Scanner::new($program);
                scanner.scan();
                let mut parser = Parser::new(&scanner.output);
                assert_eq!(parser.expr().to_string(), $format_str);
            }
        )*
        }
    }

    mod parse {
        parser_tests! {
            addition:                "2 + 2",     "(+ 2 2)"
            subtraction:             "2 - 2",     "(- 2 2)"
            multiplication:          "2 * 2",     "(* 2 2)"
            division:                "2 / 2",     "(/ 2 2)"
            modulo:                  "2 % 5",     "(% 2 5)"
            precedence_plus_times:   "1 + 2 * 3", "(+ 1 (* 2 3))"
            precedence_times_plus:   "1 * 2 + 3", "(+ (* 1 2) 3)"
            precedence_plus_div:     "1 + 2 / 3", "(+ 1 (/ 2 3))"
            precedence_div_plus:     "1 / 2 + 3", "(+ (/ 1 2) 3)"
            precedence_plus_mod:     "1 + 2 % 3", "(+ 1 (% 2 3))"
            precedence_mod_plus:     "1 % 2 + 3", "(+ (% 1 2) 3)"
            precedence_minus_times:  "1 - 2 * 3", "(- 1 (* 2 3))"
            precedence_times_minus:  "1 * 2 - 3", "(- (* 1 2) 3)"
            precedence_minus_div:    "1 - 2 / 3", "(- 1 (/ 2 3))"
            precedence_div_minus:    "1 / 2 - 3", "(- (/ 1 2) 3)"
            precedence_minus_mod:    "1 - 2 % 3", "(- 1 (% 2 3))"
            precedence_mod_minus:    "1 % 2 - 3", "(- (% 1 2) 3)"
        }
    }
}
