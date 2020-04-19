mod types;
use crate::scanner::{Token, TokenType};
pub use types::*;

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
        let mut node = self.mul_divide_term();
        if self.cur_token.token_type == TokenType::Plus
            || self.cur_token.token_type == TokenType::Minus
        {
            let t = self.cur_token.clone();
            self.cur_token = self.get_token();
            node = Box::new(Expr::BinOp(BinOp {
                item: t,
                lhs: node,
                rhs: self.mul_divide_term(),
            }));
        }
        return node;
    }

    fn mul_divide_term(&mut self) -> Box<Expr> {
        let mut node = self.num_term();
        if self.cur_token.token_type == TokenType::Mult
            || self.cur_token.token_type == TokenType::Div
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
        let mut node = Box::new(Expr::Int(1));
        if std::mem::discriminant(&self.cur_token.token_type)
            == std::mem::discriminant(&TokenType::Float(5.0))
        {
            // look into a better way of doing this. only other method I can find is match
            // by this I mean extract the data of variant float
            if let TokenType::Float(f) = self.cur_token.token_type {
                node = Box::new(Expr::Float(f));
            }
            if self.index < self.input.len() {
                self.cur_token = self.get_token();
            }
            return node;
        } else if std::mem::discriminant(&self.cur_token.token_type)
            == std::mem::discriminant(&TokenType::Int(1))
        {
            if let TokenType::Int(i) = self.cur_token.token_type {
                node = Box::new(Expr::Int(i));
            }
            if self.index < self.input.len() {
                self.cur_token = self.get_token();
            }
            return node;
        } else if self.cur_token.token_type == TokenType::OpenParen {
            // eat left paren
            self.cur_token = self.get_token();
            node = self.expr();
            // eat right paren
            self.cur_token = self.get_token();
            return node;
        } else if self.cur_token.token_type == TokenType::OpenBracket {
            // eat left brac\ket{
            self.cur_token = self.get_token();
            node = self.expr();
            self.cur_token = self.get_token();
            return node;
        } else {
            // this should never be reached
            panic!("Invalid input");
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
        }
    }
}
