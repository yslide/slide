use crate::grammar::*;
use crate::scanner::types::{Token, TokenType};
use core::convert::TryFrom;

pub fn parse(input: Vec<Token>) -> Stmt {
    let mut parser = Parser::new(input);
    *parser.parse()
}

struct Parser {
    input: Vec<Token>,
    index: usize,
}

macro_rules! binary_expr_parser {
    ($self:ident $($name:ident: lhs=$lhs_term:ident, rhs=$rhs_term:ident, op=[$($matching_op:tt)+])*) => {
        $(
        fn $name(&mut $self) -> Box<Expr> {
            use BinaryOperator::*;

            let mut lhs = $self.$lhs_term();
            while let Ok(op) = BinaryOperator::try_from($self.token()) {
                match op {
                    $($matching_op)+ => {
                        $self.advance();
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
        Parser { input, index: 0 }
    }

    fn token(&self) -> &Token {
        &self.input[self.index]
    }

    /// Returns a slice of the next `n` tokens mapped over a function `f`.
    fn peek_map<R, F>(&self, n: usize, f: F) -> Vec<R>
    where
        F: FnMut(&Token) -> R,
    {
        self.input[self.index..].iter().take(n).map(f).collect()
    }

    fn advance(&mut self) {
        self.advance_n(1);
    }

    fn advance_n(&mut self, n: usize) {
        self.index += n;
    }

    fn done(&self) -> bool {
        self.token().ty == TokenType::EOF
    }

    pub fn parse(&mut self) -> Box<Stmt> {
        let next_2 = self.peek_map(2, |t| t.ty.clone());
        let parsed = match &next_2.as_slice() {
            [TokenType::Variable(name), TokenType::Equal] => {
                self.advance_n(2);
                self.assignment(Var { name: name.clone() })
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
        if let Ok(op) = UnaryOperator::try_from(self.token()) {
            self.advance();
            return Box::new(Expr::UnaryExpr(UnaryExpr {
                op,
                rhs: self.exp_term(),
            }));
        }
        let node = match self.token().ty {
            TokenType::Float(f) => Box::new(Expr::Float(f)),
            TokenType::Int(i) => Box::new(Expr::Int(i)),
            TokenType::Variable(ref name) => Box::new(Expr::Var(Var { name: name.clone() })),
            TokenType::OpenParen => {
                self.advance(); // eat left
                Expr::Parend(self.expr()).into()
            }
            TokenType::OpenBracket => {
                self.advance(); // eat left
                Expr::Braced(self.expr()).into()
            }
            _ => unreachable!(),
        };
        self.advance(); // eat rest of created expression
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
                use crate::scanner::scan;
                use crate::parser::parse;

                let tokens = scan($program);
                let parsed = parse(tokens);
                assert_eq!(parsed.to_string(), $format_str);
            }
        )*
        }
    }

    mod parse {
        parser_tests! {
            addition:                "2 + 2",               "(+ 2 2)"
            addition_nested:         "1 + 2 + 3",           "(+ (+ 1 2) 3)"
            subtraction:             "2 - 2",               "(- 2 2)"
            subtraction_nested:      "1 - 2 - 3",           "(- (- 1 2) 3)"
            multiplication:          "2 * 2",               "(* 2 2)"
            multiplication_nested:   "1 * 2 * 3",           "(* (* 1 2) 3)"
            division:                "2 / 2",               "(/ 2 2)"
            division_nested:         "1 / 2 / 3",           "(/ (/ 1 2) 3)"
            modulo:                  "2 % 5",               "(% 2 5)"
            modulo_nested:           "1 % 2 % 3",           "(% (% 1 2) 3)"
            exponent:                "2 ^ 3",               "(^ 2 3)"
            exponent_nested:         "1 ^ 2 ^ 3",           "(^ 1 (^ 2 3))"
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
            variable:                "a",                   "a"
            variable_in_op_left:     "a + 1",               "(+ a 1)"
            variable_in_op_right:    "1 + a",               "(+ 1 a)"
            assignment_op:           "a = 5",               "(= a 5)"
            assignment_op_expr:      "a = 5 + 2 ^ 3",       "(= a (+ 5 (^ 2 3)))"
        }
    }
}
