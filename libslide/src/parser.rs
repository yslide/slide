mod types;
use crate::scanner::{Token, TokenType};
use core::convert::TryFrom;
pub use types::*;

pub struct Parser {
    input: Vec<Token>,
    index: usize,
}

macro_rules! binary_expr_parser {
    ($self:ident $($name:ident: lhs=$lhs_term:ident, rhs=$rhs_term:ident, op=[$($matching_op:tt)+])*) => {
        $(
        fn $name(&mut $self) -> Box<Expr> {
            use BinaryOperator::*;

            let lhs = $self.$lhs_term();
            if let Ok(op) = BinaryOperator::try_from($self.token()) {
                return match op {
                    $($matching_op)+ => {
                        $self.advance();
                        Box::new(Expr::BinaryExpr(BinaryExpr {
                            op,
                            lhs,
                            rhs: $self.$rhs_term(),
                        }))
                    }
                    _ => lhs,
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
            input: input,
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

    pub fn parse(&mut self) -> Box<Stmt> {
        let parsed = match self.token().token_type.clone() {
            TokenType::Variable(name) => self.assignment(name),
            _ => Box::new(Stmt::Expr(*self.expr())),
        };
        assert!(self.done());
        parsed
    }

    fn assignment(&mut self, var: String) -> Box<Stmt> {
        self.advance();
        match self.token().token_type {
            TokenType::Equal => {
                self.advance();
                Box::new(Stmt::Assignment(Assignment {
                    var,
                    rhs: self.expr(),
                }))
            }
            _ => unreachable!(),
        }
    }

    pub fn expr(&mut self) -> Box<Expr> {
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
        if let Ok(op) = UnaryOperator::try_from(self.token()) {
            self.advance();
            return Box::new(Expr::UnaryExpr(UnaryExpr {
                op,
                rhs: self.exp_term(),
            }));
        }

        let node = match self.token().token_type {
            TokenType::Float(f) => Box::new(Expr::Float(f)),
            TokenType::Int(i) => Box::new(Expr::Int(i)),
            TokenType::OpenParen | TokenType::OpenBracket => {
                self.advance(); // eat left
                self.expr()
            }
            _ => unreachable!(),
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
                let mut parser = Parser::new(scanner.output);
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
            brackets_plus_times:     "[1 + 2] * 3",         "(* (+ 1 2) 3)"
            brackets_time_plus:      "3 * [1 + 2]",         "(* 3 (+ 1 2))"
            brackets_time_mod:       "3 * [2 % 2]",         "(* 3 (% 2 2))"
            brackets_mod_time:       "[2 % 2] * 3",         "(* (% 2 2) 3)"
            brackets_exp_time:       "2 ^ [3 ^ 4 * 5]",     "(^ 2 (* (^ 3 4) 5))"
            brackets_unary:          "-[2 + +-5]",          "(- (+ 2 (+ (- 5))))"
            nested_brackets:         "[[1 * [2 + 3]] ^ 4]", "(^ (* 1 (+ 2 3)) 4)"
            unary_minus:             "-2",                  "(- 2)"
            unary_expo:              "-2 ^ 3",              "(- (^ 2 3))"
            unary_quad:              "+-+-2",               "(+ (- (+ (- 2))))"
            assignment_op:           "a = 5",               "(= a 5)"
            assignment_op_expr:      "a = 5 + 2 ^ 3",       "(= a (+ 5 (^ 2 3)))"
        }
    }
}
