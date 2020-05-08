use crate::grammar::*;
use crate::scanner::types::{Token, TokenType};
use crate::utils::PeekIter;

use core::convert::TryFrom;
pub use std::vec::IntoIter;

pub fn parse(input: Vec<Token>) -> Stmt {
    let mut parser = Parser::new(input);
    *parser.parse()
}

struct Parser {
    input: PeekIter<Token>,
}

macro_rules! binary_expr_parser {
    ($self:ident $($name:ident: lhs=$lhs_term:ident, rhs=$rhs_term:ident, op=[$($matching_op:tt)+])*) => {
        $(
        fn $name(&mut $self) -> Box<Expr> {
            use BinaryOperator::*;

            let mut lhs = $self.$lhs_term();
            while let Ok(op) = $self
                .input
                .peek()
                .map_or_else(|| Err(()), BinaryOperator::try_from)
            {
                match op {
                    $($matching_op)+ => {
                        $self.input.next();
                        lhs = Expr::BinaryExpr(BinaryExpr{
                            op,
                            lhs,
                            rhs: $self.$rhs_term(),
                        }).into();
                    }
                    _ => break,
                }
            }
            lhs
        }
        )*
    };
}

impl Parser {
    pub fn new(input: Vec<Token>) -> Parser {
        Parser {
            input: PeekIter::new(input.into_iter()),
        }
    }

    fn done(&mut self) -> bool {
        self.input.peek().map(|t| &t.ty) == Some(&TokenType::EOF)
    }

    pub fn parse(&mut self) -> Box<Stmt> {
        eprintln!("here",);
        let mut next_2 = self.input.peek_map_n(2, |tok| tok.ty.clone());
        let parsed = match (next_2.pop_front(), next_2.pop_front()) {
            (Some(TokenType::Variable(name)), Some(TokenType::Equal)) => {
                self.input.next();
                self.input.next();
                self.assignment(Var { name })
            }
            _ => Box::new(Stmt::Expr(*self.expr())),
        };
        assert!(self.done());
        parsed
    }

    fn assignment(&mut self, var: Var) -> Box<Stmt> {
        Box::new(Stmt::Assignment(Assignment {
            var,
            rhs: self.expr(),
        }))
    }

    fn expr(&mut self) -> Box<Expr> {
        self.add_sub_term()
    }

    binary_expr_parser!(
        self

        // Level 1: +, -
        add_sub_term:        lhs = mul_divide_mod_term, rhs = mul_divide_mod_term, op = [Plus | Minus]

        // Level 2: *, /, %
        mul_divide_mod_term: lhs = exp_term,            rhs = exp_term,            op = [Mult | Div | Mod]

        // Level 3: ^                                   right-associativity of ^
        exp_term:            lhs = num_term,            rhs = exp_term,            op = [Exp]
    );

    fn num_term(&mut self) -> Box<Expr> {
        if let Some(Ok(op)) = self.input.peek().map(UnaryOperator::try_from) {
            self.input.next();
            return Box::new(Expr::UnaryExpr(UnaryExpr {
                op,
                rhs: self.exp_term(),
            }));
        }
        let node = match self.input.peek().map(|t| &t.ty) {
            Some(TokenType::Float(f)) => Box::new(Expr::Const(*f)),
            Some(TokenType::Variable(name)) => Box::new(Expr::Var(Var { name: name.clone() })),
            Some(TokenType::OpenParen) => {
                self.input.next(); // eat left
                Expr::Parend(self.expr()).into()
            }
            Some(TokenType::OpenBracket) => {
                self.input.next(); // eat left
                Expr::Braced(self.expr()).into()
            }
            _ => unreachable!(),
        };
        self.input.next(); // eat rest of created expression
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
        ($($name:ident: $program:expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::scanner::{scan, ScannerOptions};
                use crate::parser::parse;

                let tokens = scan($program, ScannerOptions::default());
                let parsed = parse(tokens);
                assert_eq!(parsed.to_string(), $program);
            }
        )*
        }
    }

    mod parse {
        parser_tests! {
            addition:                "2 + 2"
            addition_nested:         "1 + 2 + 3"
            subtraction:             "2 - 2"
            subtraction_nested:      "1 - 2 - 3"
            multiplication:          "2 * 2"
            multiplication_nested:   "1 * 2 * 3"
            division:                "2 / 2"
            division_nested:         "1 / 2 / 3"
            modulo:                  "2 % 5"
            modulo_nested:           "1 % 2 % 3"
            exponent:                "2 ^ 3"
            exponent_nested:         "1 ^ 2 ^ 3"
            precedence_plus_times:   "1 + 2 * 3"
            precedence_times_plus:   "1 * 2 + 3"
            precedence_plus_div:     "1 + 2 / 3"
            precedence_div_plus:     "1 / 2 + 3"
            precedence_plus_mod:     "1 + 2 % 3"
            precedence_mod_plus:     "1 % 2 + 3"
            precedence_minus_times:  "1 - 2 * 3"
            precedence_times_minus:  "1 * 2 - 3"
            precedence_minus_div:    "1 - 2 / 3"
            precedence_div_minus:    "1 / 2 - 3"
            precedence_minus_mod:    "1 - 2 % 3"
            precedence_mod_minus:    "1 % 2 - 3"
            precedence_expo_plus:    "1 + 2 ^ 3"
            precedence_plus_exp:     "1 ^ 2 + 3"
            precedence_expo_times:   "1 * 2 ^ 3"
            precedence_time_expo:    "1 ^ 2 * 3"
            parentheses_plus_times:  "(1 + 2) * 3"
            parentheses_time_plus:   "3 * (1 + 2)"
            parentheses_time_mod:    "3 * (2 % 2)"
            parentheses_mod_time:    "(2 % 2) * 3"
            parentheses_exp_time:    "2 ^ (3 ^ 4 * 5)"
            parentheses_unary:       "-(2 + +-5)"
            nested_parentheses:      "((1 * (2 + 3)) ^ 4)"
            brackets_plus_times:     "[1 + 2] * 3"
            brackets_time_plus:      "3 * [1 + 2]"
            brackets_time_mod:       "3 * [2 % 2]"
            brackets_mod_time:       "[2 % 2] * 3"
            brackets_exp_time:       "2 ^ [3 ^ 4 * 5]"
            brackets_unary:          "-[2 + +-5]"
            nested_brackets:         "[[1 * [2 + 3]] ^ 4]"
            unary_minus:             "-2"
            unary_expo:              "-2 ^ 3"
            unary_quad:              "+-+-2"
            variable:                "a"
            variable_in_op_left:     "a + 1"
            variable_in_op_right:    "1 + a"
            assignment_op:           "a = 5"
            assignment_op_expr:      "a = 5 + 2 ^ 3"
        }
    }
}
