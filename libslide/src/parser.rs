#[macro_use]
pub mod test_utils; // this **must** be first since macro import order matters!

mod expression_parser;
mod expression_pattern_parser;

pub use expression_parser::parse as parse_expression;
pub use expression_pattern_parser::parse as parse_expression_pattern;

use crate::grammar::*;
use crate::scanner::types::{Token, TokenType};
use crate::utils::PeekIter;

use core::convert::TryFrom;
use core::fmt::Display;
use std::rc::Rc;

macro_rules! binary_expr_parser {
    ($self:ident $($name:ident: lhs=$lhs_term:ident, rhs=$rhs_term:ident, op=[$($matching_op:tt)+])*) => {
        $(
        fn $name(&mut $self) -> Rc<Self::Expr> {
            use BinaryOperator::*;

            let mut lhs = $self.$lhs_term();
            while let Ok(op) = $self
                .input()
                .peek()
                .map_or_else(|| Err(()), BinaryOperator::try_from)
            {
                match op {
                    $($matching_op)+ => {
                        $self.input().next();
                        let bin_expr = BinaryExpr{
                            op,
                            lhs,
                            rhs: $self.$rhs_term(),
                        }.into();
                        lhs = $self.finish_expr(bin_expr);
                    }
                    _ => break,
                }
            }
            lhs
        }
        )*
    };
}

trait Parser<T>
where
    T: Grammar,
    Self::Expr: Expression + Clone,
    Self::Error: Display,
{
    type Expr;
    type Error;

    fn new(input: Vec<Token>) -> Self;
    fn errors(&self) -> &Vec<Self::Error>;
    fn input(&mut self) -> &mut PeekIter<Token>;
    fn parse(&mut self) -> T;
    fn parse_float(&mut self, f: f64) -> Self::Expr;
    fn parse_variable(&mut self, name: String) -> Self::Expr;
    fn parse_var_pattern(&mut self, name: String) -> Self::Expr;
    fn parse_const_pattern(&mut self, name: String) -> Self::Expr;
    fn parse_any_pattern(&mut self, name: String) -> Self::Expr;
    fn parse_open_paren(&mut self) -> Self::Expr;
    fn parse_open_bracket(&mut self) -> Self::Expr;
    fn finish_expr(&mut self, expr: Self::Expr) -> Rc<Self::Expr>;

    // Default parsing implementations.
    // TODO: increase modularity of this parser

    fn done(&mut self) -> bool {
        self.input().peek().map(|t| &t.ty) == Some(&TokenType::EOF)
    }

    fn expr(&mut self) -> Rc<Self::Expr> {
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

    fn num_term(&mut self) -> Rc<Self::Expr> {
        let node = if let Some(Ok(op)) = self.input().peek().map(UnaryOperator::try_from) {
            self.input().next();
            UnaryExpr {
                op,
                rhs: self.exp_term(),
            }
            .into()
        } else {
            let node = match self.input().peek().map(|t| t.ty.clone()) {
                Some(TokenType::Float(f)) => self.parse_float(f),
                Some(TokenType::Variable(name)) => self.parse_variable(name),

                Some(TokenType::VariablePattern(name)) => self.parse_var_pattern(name),
                Some(TokenType::ConstPattern(name)) => self.parse_const_pattern(name),
                Some(TokenType::AnyPattern(name)) => self.parse_any_pattern(name),

                Some(TokenType::OpenParen) => self.parse_open_paren(),
                Some(TokenType::OpenBracket) => self.parse_open_bracket(),
                _ => unreachable!(),
            };
            self.input().next(); // eat rest of created expression
            node
        };

        match self.input().peek().map(|t| &t.ty) {
            // <node>(<other>) => <node> * (<other>)
            Some(TokenType::OpenParen) | Some(TokenType::OpenBracket) => {
                // TODO: mark this node as synthetic
                self.input().push_front(Token::new(TokenType::Mult, (0, 0)));
            }
            // <num><var> => <num> * <var>
            Some(TokenType::Variable(_))
            | Some(TokenType::VariablePattern(_))
            | Some(TokenType::ConstPattern(_))
            | Some(TokenType::AnyPattern(_))
                if node.is_const() =>
            {
                // TODO: mark this node as synthetic
                self.input().push_front(Token::new(TokenType::Mult, (0, 0)))
            }
            _ => {}
        }

        self.finish_expr(node)
    }
}

#[cfg(test)]
mod tests {
    common_parser_tests! {
        addition:                               "2 + 2"
        addition_nested:                        "1 + 2 + 3"
        addition_sub_nested:                    "1 + 2 - 3"
        subtraction:                            "2 - 2"
        subtraction_nested:                     "1 - 2 - 3"
        subtraction_add_nested:                 "1 - 2 + 3"
        multiplication:                         "2 * 2"
        multiplication_nested:                  "1 * 2 * 3"
        division:                               "2 / 2"
        division_nested:                        "1 / 2 / 3"
        modulo:                                 "2 % 5"
        modulo_nested:                          "1 % 2 % 3"
        exponent:                               "2 ^ 3"
        exponent_nested:                        "1 ^ 2 ^ 3"
        precedence_plus_times:                  "1 + 2 * 3"
        precedence_times_plus:                  "1 * 2 + 3"
        precedence_plus_div:                    "1 + 2 / 3"
        precedence_div_plus:                    "1 / 2 + 3"
        precedence_plus_mod:                    "1 + 2 % 3"
        precedence_mod_plus:                    "1 % 2 + 3"
        precedence_minus_times:                 "1 - 2 * 3"
        precedence_times_minus:                 "1 * 2 - 3"
        precedence_minus_div:                   "1 - 2 / 3"
        precedence_div_minus:                   "1 / 2 - 3"
        precedence_minus_mod:                   "1 - 2 % 3"
        precedence_mod_minus:                   "1 % 2 - 3"
        precedence_expo_plus:                   "1 + 2 ^ 3"
        precedence_plus_exp:                    "1 ^ 2 + 3"
        precedence_expo_times:                  "1 * 2 ^ 3"
        precedence_time_expo:                   "1 ^ 2 * 3"
        parentheses_plus_times:                 "(1 + 2) * 3"
        parentheses_time_plus:                  "3 * (1 + 2)"
        parentheses_time_mod:                   "3 * (2 % 2)"
        parentheses_mod_time:                   "(2 % 2) * 3"
        parentheses_exp_time:                   "2 ^ (3 ^ 4 * 5)"
        parentheses_unary:                      "-(2 + +-5)"
        nested_parentheses:                     "((1 * (2 + 3)) ^ 4)"
        brackets_plus_times:                    "[1 + 2] * 3"
        brackets_time_plus:                     "3 * [1 + 2]"
        brackets_time_mod:                      "3 * [2 % 2]"
        brackets_mod_time:                      "[2 % 2] * 3"
        brackets_exp_time:                      "2 ^ [3 ^ 4 * 5]"
        brackets_unary:                         "-[2 + +-5]"
        nested_brackets:                        "[[1 * [2 + 3]] ^ 4]"
        unary_minus:                            "-2"
        unary_quad:                             "+-+-2"
        implicit_mult_num_var:                  "2x => 2 * x"
        implicit_mult_num_var_pat:              "2$x => 2 * $x"
        implicit_mult_num_const_pat:            "2#x => 2 * #x"
        implicit_mult_num_any_pat:              "2_x => 2 * _x"
        implicit_mult_num_paren:                "2(1) => 2 * (1)"
        implicit_mult_num_bracket:              "2[1] => 2 * [1]"
        implicit_mult_var_paren:                "x(1) => x * (1)"
        implicit_mult_var_bracket:              "x[1] => x * [1]"
        implicit_mult_var_pat_paren:            "$x(1) => $x * (1)"
        implicit_mult_var_pat_bracket:          "$x[1] => $x * [1]"
        implicit_mult_const_pat_paren:          "$x(1) => $x * (1)"
        implicit_mult_const_pat_bracket:        "$x[1] => $x * [1]"
        implicit_mult_any_pat_paren:            "_x(1) => _x * (1)"
        implicit_mult_any_pat_bracket:          "_x[1] => _x * [1]"
        implicit_mult_paren_paren:              "(1)(2) => (1) * (2)"
        implicit_mult_paren_bracket:            "(1)[2] => (1) * [2]"
        implicit_mult_bracket_paren:            "[1](2) => [1] * (2)"
        implicit_mult_bracket_bracket:          "[1][2] => [1] * [2]"
        implicit_mult_unary_paren:              "-1(2) => -1 * (2)"
        implicit_mult_unary_bracket:            "-1[2] => -1 * [2]"
        implicit_mult_unary_nested_var:         "-2x => -2 * x"
        implicit_mult_exp:                      "2x^5 => 2 * x ^ 5"
    }
}
