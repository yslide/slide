//! Parses slide programs into the slide [`grammar`][crate::grammar] IR and produces semantic
//! diagnostics.

#[macro_use]
pub mod test_utils; // this **must** be first since macro import order matters!

mod expression_parser;
mod expression_pattern_parser;

pub use expression_parser::parse as parse_expression;
pub use expression_pattern_parser::parse as parse_expression_pattern;

use crate::common::Span;
use crate::diagnostics::Diagnostic;
use crate::grammar::*;
use crate::scanner::types::{Token, TokenType as TT};
use crate::utils::PeekIter;

use core::convert::TryFrom;
use std::rc::Rc;

macro_rules! binary_expr_parser {
    ($self:ident $($name:ident: lhs=$lhs_term:ident, rhs=$rhs_term:ident, op=[$($matching_op:tt)+])*) => {
        $(
        fn $name(&mut $self) -> Rc<Self::Expr> {
            use BinaryOperator::*;

            let mut lhs = $self.$lhs_term();
            while let Ok(op) = BinaryOperator::try_from($self.peek())
            {
                match op {
                    $($matching_op)+ => {
                        $self.next();
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

/// Returns a diagnostic for an unclosed delimiter.
fn unclosed_delimiter(open: Token, expected: TT, found: Token) -> Diagnostic {
    let mut found_str = found.to_string();
    if !matches!(found.ty, TT::EOF) {
        found_str = format!("`{}`", found_str);
    }
    Diagnostic::span_err(
        found.span,
        format!("Expected `{}`, found {}", expected, found_str),
        format!("expected closing `{}`", expected),
    )
    .with_help_note(open.span, format!("opening `{}` here", open))
}

/// Returns a diagnostic for extra tokens following a primary item.
/// `extra_tokens` will be consumed in the construction of the diagnostic.
fn extra_tokens_diag(extra_tokens: &mut PeekIter<Token>) -> Diagnostic {
    let Span { lo, mut hi } = extra_tokens.peek().unwrap().span;
    while let Some(tok) = extra_tokens.next() {
        if extra_tokens.peek().unwrap().ty == TT::EOF {
            hi = tok.span.hi;
            break;
        }
    }
    Diagnostic::span_err(
        lo..hi,
        "Unexpected extra tokens",
        "not connected to a primary expression".to_string(),
    )
}

trait Parser<T>
where
    T: Grammar,
    Self::Expr: Expression + Clone,
{
    type Expr;

    fn new(input: Vec<Token>) -> Self;
    fn input(&mut self) -> &mut PeekIter<Token>;
    fn parse(&mut self) -> T;
    fn parse_float(&mut self, f: f64, span: Span) -> Self::Expr;
    fn parse_variable(&mut self, name: String, span: Span) -> Self::Expr;
    fn parse_var_pattern(&mut self, name: String, span: Span) -> Self::Expr;
    fn parse_const_pattern(&mut self, name: String, span: Span) -> Self::Expr;
    fn parse_any_pattern(&mut self, name: String, span: Span) -> Self::Expr;
    fn parse_open_paren(&mut self, open: Token) -> Self::Expr {
        let expr = Self::Expr::paren(self.expr());
        let closing_tok = self.next();
        if !matches!(closing_tok.ty, TT::CloseParen) {
            self.push_diag(unclosed_delimiter(open, TT::CloseParen, closing_tok));
        }
        expr
    }
    fn parse_open_bracket(&mut self, open: Token) -> Self::Expr {
        let expr = Self::Expr::bracket(self.expr());
        let closing_tok = self.next();
        if !matches!(closing_tok.ty, TT::CloseBracket) {
            self.push_diag(unclosed_delimiter(open, TT::CloseBracket, closing_tok));
        }
        expr
    }
    fn finish_expr(&mut self, expr: Self::Expr) -> Rc<Self::Expr>;
    fn push_diag(&mut self, diagnostic: Diagnostic);

    // Default parsing implementations.
    // TODO: increase modularity of this parser

    #[inline]
    fn done(&mut self) -> bool {
        self.input().peek().map(|t| &t.ty) == Some(&TT::EOF)
    }

    #[inline]
    fn expr(&mut self) -> Rc<Self::Expr> {
        self.add_sub_term()
    }

    #[inline]
    fn peek(&mut self) -> &Token {
        self.input().peek().unwrap()
    }

    #[inline]
    fn next(&mut self) -> Token {
        if self.done() {
            self.peek().clone()
        } else {
            self.input().next().unwrap()
        }
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
        let tok = self.next();
        if matches!(tok.ty, TT::EOF) {
            self.push_diag(Diagnostic::span_err(
                tok.span,
                "Expected an expression, found end of file",
                Some("expected an expression".into()),
            ));
            return self.finish_expr(Self::Expr::empty());
        }

        let node = if let Ok(op) = UnaryOperator::try_from(&tok) {
            UnaryExpr {
                op,
                rhs: self.exp_term(),
            }
            .into()
        } else {
            match tok.ty {
                TT::Float(f) => self.parse_float(f, tok.span),
                TT::Variable(name) => self.parse_variable(name, tok.span),
                TT::VariablePattern(name) => self.parse_var_pattern(name, tok.span),
                TT::ConstPattern(name) => self.parse_const_pattern(name, tok.span),
                TT::AnyPattern(name) => self.parse_any_pattern(name, tok.span),
                TT::OpenParen => self.parse_open_paren(tok),
                TT::OpenBracket => self.parse_open_bracket(tok),
                _ => {
                    self.push_diag(Diagnostic::span_err(
                        tok.span,
                        format!("Expected an expression, found `{}`", tok.to_string()),
                        Some("expected an expression".into()),
                    ));
                    return self.finish_expr(Self::Expr::empty());
                }
            }
        };

        match self.peek().ty {
            // <node>(<other>) => <node> * (<other>)
            TT::OpenParen | TT::OpenBracket => {
                // TODO: mark this node as synthetic
                self.input().push_front(Token::new(TT::Mult, (0, 0)));
            }
            // <num><var> => <num> * <var>
            TT::Variable(_) | TT::VariablePattern(_) | TT::ConstPattern(_) | TT::AnyPattern(_)
                if node.is_const() =>
            {
                // TODO: mark this node as synthetic
                self.input().push_front(Token::new(TT::Mult, (0, 0)))
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
