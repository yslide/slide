mod types;
pub use types::*;
use crate::scanner::{Token, TokenType};


pub struct Parser<'a> {
    input: &'a Vec<Token>,
    index: usize,
    cur_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(input: &Vec<Token>) -> Parser {
        Parser {
            input: input.into(),
            index: 1,
            cur_token: input[0].clone().into(),
        }
    }

    fn get_token(&mut self) -> Token {
        self.index = self.index + 1;
        return self.input[self.index - 1].clone();
    }

    pub fn expr(&mut self) -> Box<Expr> {
        return self.add_sub_term();
    }

    fn add_sub_term(&mut self) -> Box<Expr> {
        let mut node = self.mul_divide_mod_term();
        if self.cur_token.token_type == TokenType::Plus
            || self.cur_token.token_type == TokenType::Minus
        {
            let t = self.cur_token.clone();
            self.cur_token = self.get_token();
            node = Box::new(Expr::BinOp(BinOp {
                item: t,
                lhs: node,
                rhs: self.mul_divide_mod_term(),
            }));
        }
        return node;
    }

    fn mul_divide_mod_term(&mut self) -> Box<Expr> {
        let mut node = self.num_term();
        if self.cur_token.token_type == TokenType::Mult
            || self.cur_token.token_type == TokenType::Div || self.cur_token.token_type == TokenType::Mod
        {
            let t = self.cur_token.clone();
            self.cur_token = self.get_token();
            node = Box::new(Expr::BinOp(BinOp {
                item: t,
                lhs: node,
                rhs: self.num_term(),
            }));
        }
        return node;
    }

    fn num_term(&mut self) -> Box<Expr> {
        // 5.0 is a placeholder. mem::discriminant only compares variant types and ignores data
        // this is pretty fucking cool rust has it
        // this value should never be returned
        let node: Box<Expr>;
        match self.cur_token.token_type {
            TokenType::Float(f) => {
                node = Box::new(Expr::Float(f));
                if self.index < self.input.len() {
                    self.cur_token = self.get_token();
                }
                return node;
            },
            TokenType::Int(i) => {
                node = Box::new(Expr::Int(i));
                if self.index < self.input.len() {
                    self.cur_token = self.get_token();
                }
                return node;
            },
            // @todo check for paren errors
            TokenType::OpenParen => {
                // eat left paren
                self.cur_token = self.get_token();
                node = self.expr();
                // eat right paren
                self.cur_token = self.get_token();
                return node;
            },
            TokenType::OpenBracket => {
                // eat left brac\ket{
                self.cur_token = self.get_token();
                node = self.expr();
                self.cur_token = self.get_token();
                return node;
            }
            _ => {
            // this should never be reached
            panic!("Invalid input");
            },
        }
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
            addition: "2 + 2", "(+ 2 2)"
            subtraction: "2 - 2", "(- 2 2)"
            multiplication: "2 * 2", "(* 2 2)"
            division: "2 / 2", "(/ 2 2)"
            modulo:  "2 % 5", "(% 2 5)"
        }
    }
}
