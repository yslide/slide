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

    fn advance(&mut self) {
        self.index += 1;
    }

    fn done(&self) -> bool {
        self.token().token_type == TokenType::EOF
    }

    pub fn parse(&mut self) -> Box<Expr> {
        let parsed = match self.token().token_type {
            TokenType::Variable(_) => self.assignment(),
            _ => self.add_sub_term(),
        };
        assert!(self.done());
        parsed
    }

    fn assignment(&mut self) -> Box<Expr> {
        let lhs = self.token().clone();
        self.advance();
        match self.token().token_type {
            TokenType::Equal => {
                let operand = self.token().clone();
                self.advance();
                Box::new(Expr::Variable(Variable {
                    op: operand,
                    lhs: lhs,
                    rhs: self.expr(),
                }))
            }
            _ => unreachable!(),
        }
    }

    pub fn expr(&mut self) -> Box<Expr> {
        self.add_sub_term()
    }

    fn add_sub_term(&mut self) -> Box<Expr> {
        let lhs = self.mul_divide_mod_term();
        match self.token().token_type {
            TokenType::Plus | TokenType::Minus => {
                let op = self.token().clone();
                self.advance();
                Box::new(Expr::BinOp(BinOp {
                    op,
                    lhs,
                    rhs: self.mul_divide_mod_term(),
                }))
            }
            _ => lhs,
        }
    }

    fn mul_divide_mod_term(&mut self) -> Box<Expr> {
        let lhs = self.exponent_term();
        match self.token().token_type {
            TokenType::Mult | TokenType::Div | TokenType::Mod => {
                let op = self.token().clone();
                self.advance();
                Box::new(Expr::BinOp(BinOp {
                    op,
                    lhs,
                    rhs: self.exponent_term(),
                }))
            }
            _ => lhs,
        }
    }

    fn exponent_term(&mut self) -> Box<Expr> {
        let lhs = self.num_term();
        match self.token().token_type {
            TokenType::Exp => {
                let op = self.token().clone();
                self.advance();
                Box::new(Expr::BinOp(BinOp {
                    op,
                    lhs,
                    rhs: self.exponent_term(),
                }))
            }
            _ => lhs,
        }
    }

    fn num_term(&mut self) -> Box<Expr> {
        match self.token().token_type {
            TokenType::Plus | TokenType::Minus => {
                let op = self.token().clone();
                self.advance();
                Box::new(Expr::UnaryOp(UnaryOp {
                    op,
                    rhs: self.exponent_term(),
                }))
            }
            TokenType::Float(f) => {
                let node = Box::new(Expr::Float(f));
                self.advance();
                node
            }
            TokenType::Int(i) => {
                let node = Box::new(Expr::Int(i));
                self.advance();
                node
            }
            TokenType::OpenParen => {
                // eat left paren
                self.advance();
                let node = self.expr();
                self.advance();
                node
            }
            TokenType::OpenBracket => {
                // eat left bracket
                self.advance();
                let node = self.expr();
                self.advance();
                node
            }
            _ => unreachable!(),
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
                assert_eq!(parser.parse().to_string(), $format_str);
            }
        )*
        }
    }

    mod parse {
        parser_tests! {
            addition:                "2 + 2",               "(+ 2 2)"
            subtraction:             "2 - 2",               "(- 2 2)"
            multiplication:          "2 * 2",               "(* 2 2)"
            division:                "2 / 2",               "(/ 2 2)"
            modulo:                  "2 % 5",               "(% 2 5)"
            exponent:                "2 ^ 3",               "(^ 2 3)"
            precedence_plus_times:   "1 + 2 * 3",           "(+ 1 (* 2 3))"
            precedence_times_plus:   "1 * 2 + 3",           "(+ (* 1 2) 3)"
            precedence_plus_div:     "1 + 2 / 3",           "(+ 1 (/ 2 3))"
            precedence_div_plus:     "1 / 2 + 3",           "(+ (/ 1 2) 3)"
            precedence_plus_mod:     "1 + 2 % 3",           "(+ 1 (% 2 3))"
            precedence_mod_plus:     "1 % 2 + 3",           "(+ (% 1 2) 3)"
            precedence_minus_times:  "1 - 2 * 3",           "(- 1 (* 2 3))"
            precedence_times_minus:  "1 * 2 - 3",           "(- (* 1 2) 3)"
            precedence_minus_div:    "1 - 2 / 3",           "(- 1 (/ 2 3))"
            precedence_div_minus:    "1 / 2 - 3",           "(- (/ 1 2) 3)"
            precedence_minus_mod:    "1 - 2 % 3",           "(- 1 (% 2 3))"
            precedence_mod_minus:    "1 % 2 - 3",           "(- (% 1 2) 3)"
            precedence_expo_plus:    "1 + 2 ^ 3",           "(+ 1 (^ 2 3))"
            precedence_plus_exp:     "1 ^ 2 + 3",           "(+ (^ 1 2) 3)"
            precedence_expo_times:   "1 * 2 ^ 3",           "(* 1 (^ 2 3))"
            precedence_time_expo:    "1 ^ 2 * 3",           "(* (^ 1 2) 3)"
            precedence_expo_exp:     "1 ^ 2 ^ 3",           "(^ 1 (^ 2 3))"
            parentheses_plus_times:  "(1 + 2) * 3",         "(* (+ 1 2) 3)"
            parentheses_time_plus:   "3 * (1 + 2)",         "(* 3 (+ 1 2))"
            parentheses_time_mod:    "3 * (2 % 2)",         "(* 3 (% 2 2))"
            parentheses_mod_time:    "(2 % 2) * 3",         "(* (% 2 2) 3)"
            parentheses_exp_time:    "2 ^ (3 ^ 4 * 5)",     "(^ 2 (* (^ 3 4) 5))"
            parentheses_unary:       "-(2 + +-5)",          "(- (+ 2 (+ (- 5))))"
            nested_parentheses:      "((1 * (2 + 3)) ^ 4)", "(^ (* 1 (+ 2 3)) 4)"
            brackets_plus_times:     "(1 + 2) * 3",         "(* (+ 1 2) 3)"
            brackets_time_plus:      "3 * (1 + 2)",         "(* 3 (+ 1 2))"
            brackets_time_mod:       "3 * (2 % 2)",         "(* 3 (% 2 2))"
            brackets_mod_time:       "(2 % 2) * 3",         "(* (% 2 2) 3)"
            brackets_exp_time:       "2 ^ (3 ^ 4 * 5)",     "(^ 2 (* (^ 3 4) 5))"
            brackets_unary:          "-(2 + +-5)",          "(- (+ 2 (+ (- 5))))"
            nested_brackets:         "((1 * (2 + 3)) ^ 4)", "(^ (* 1 (+ 2 3)) 4)"
            unary_minus:             "-2",                  "(- 2)"
            unary_expo:              "-2 ^ 3",              "(- (^ 2 3))"
            unary_quad:              "+-+-2",               "(+ (- (+ (- 2))))"
            assignment_op:           "a = 5",               "(= a 5)"
            assignment_op_expr:      "a = 5 + 2 ^ 3",       "(= a (+ 5 (^ 2 3)))"
        }
    }
}
