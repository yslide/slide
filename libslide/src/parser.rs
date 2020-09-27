//! Parses slide programs into the slide [`grammar`][crate::grammar] IR and produces semantic
//! diagnostics.

#[macro_use]
pub mod test_utils; // this **must** be first since macro import order matters!

#[macro_use]
mod errors;
pub use errors::ParseErrors;
use errors::*;

mod expression_pattern_parser;
mod statement_parser;

pub use expression_pattern_parser::parse as parse_expression_pattern;
pub use statement_parser::parse as parse_statement;

use crate::common::Span;
use crate::diagnostics::{Diagnostic, DiagnosticRecord};
use crate::grammar::*;
use crate::scanner::types::{Token, TokenType as TT};
use crate::utils::PeekIter;

use core::convert::TryFrom;
use rug::{Float, Rational};

macro_rules! binary_expr_parser {
    ($self:ident $($name:ident: lhs=$lhs_term:ident, rhs=$rhs_term:ident, op=[$($matching_op:tt)+])*) => {
        $(
        fn $name(&mut $self) -> Self::Expr {
            use BinaryOperator::*;

            let mut lhs = $self.$lhs_term();
            while let Ok(op) = BinaryOperator::try_from($self.peek())
            {
                match op {
                    $($matching_op)+ => {
                        $self.next();
                        let rhs = $self.$rhs_term();
                        let span = lhs.span().to(rhs.span());
                        lhs = Self::Expr::binary(
                            BinaryExpr { op, lhs, rhs, },
                            span,
                        );
                    }
                    _ => break,
                }
            }
            lhs
        }
        )*
    };
}

/// Parses a string into a [Rational][rug::Rational].
fn str2rat(s: &str, prec: u32) -> Rational {
    if s.contains('.') {
        Float::parse(s)
            .ok()
            .map(|f| Float::with_val(prec, f))
            .and_then(|f| f.to_rational())
            .unwrap()
    } else {
        Rational::parse(s).map(Rational::from).unwrap()
    }
}

/// Returns a diagnostic for an unclosed delimiter.
fn unclosed_delimiter(opener: Token, expected_closer: TT, found_closer: Token) -> Diagnostic {
    let mut found_str = found_closer.to_string();
    if !matches!(found_closer.ty, TT::EOF) {
        found_str = format!("`{}`", found_str);
    }
    MismatchedClosingDelimiter!(expected expected_closer, at found_closer.span,
                                due to opener, at opener.span;
                                found found_str)
}

trait Parser<T>
where
    T: Grammar,
    Self::Expr: RcExpression,
{
    type Expr;

    // fn new(input: Vec<Token>) -> Self;
    fn input(&mut self) -> &mut PeekIter<Token>;
    fn parse(&mut self) -> T;
    fn parse_num(&mut self, num: String, span: Span) -> Self::Expr;
    fn parse_variable(&mut self, name: String, span: Span) -> Self::Expr;
    fn parse_var_pattern(&mut self, name: String, span: Span) -> Self::Expr;
    fn parse_const_pattern(&mut self, name: String, span: Span) -> Self::Expr;
    fn parse_any_pattern(&mut self, name: String, span: Span) -> Self::Expr;
    fn parse_open_paren(&mut self, open: Token) -> Self::Expr {
        let inner = self.expr();
        let closing_tok = self.next();
        let sp = open.span.to(closing_tok.span);
        if !matches!(closing_tok.ty, TT::CloseParen) {
            self.push_diag(unclosed_delimiter(open, TT::CloseParen, closing_tok));
        }
        Self::Expr::paren(inner, sp)
    }
    fn parse_open_bracket(&mut self, open: Token) -> Self::Expr {
        let inner = self.expr();
        let closing_tok = self.next();
        let sp = open.span.to(closing_tok.span);
        if !matches!(closing_tok.ty, TT::CloseBracket) {
            self.push_diag(unclosed_delimiter(open, TT::CloseBracket, closing_tok));
        }
        Self::Expr::bracket(inner, sp)
    }
    fn push_diag(&mut self, diagnostic: Diagnostic);

    fn has_stmt_break(&mut self) -> bool;

    #[inline]
    fn done(&mut self) -> bool {
        self.input().peek().map(|t| &t.ty) == Some(&TT::EOF)
    }

    #[inline]
    fn expr(&mut self) -> Self::Expr {
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

    fn num_term(&mut self) -> Self::Expr {
        let tok = self.next();
        let tok_span = tok.span;
        if matches!(tok.ty, TT::EOF) {
            self.push_diag(ExpectedExpr!(tok.span, "end of file"));
            return Self::Expr::empty(tok.span);
        }

        let node = if let Ok(op) = UnaryOperator::try_from(&tok) {
            let rhs = self.exp_term();
            let span = tok.span.to(rhs.span());
            Self::Expr::unary(UnaryExpr { op, rhs }, span)
        } else {
            match tok.ty {
                TT::Num(n) => self.parse_num(n, tok.span),
                TT::Variable(name) => self.parse_variable(name, tok.span),
                TT::VariablePattern(name) => self.parse_var_pattern(name, tok.span),
                TT::ConstPattern(name) => self.parse_const_pattern(name, tok.span),
                TT::AnyPattern(name) => self.parse_any_pattern(name, tok.span),
                TT::OpenParen => self.parse_open_paren(tok),
                TT::OpenBracket => self.parse_open_bracket(tok),
                _ => {
                    self.push_diag(if matches!(tok.ty, TT::CloseParen | TT::CloseBracket) {
                        UnmatchedClosingDelimiter!(tok.span, tok.ty)
                    } else {
                        ExpectedExpr!(tok.span, tok.to_string())
                    });
                    Self::Expr::empty(tok.span)
                }
            }
        };

        let insert_synthetic_mult = match self.peek().ty {
            // <node>(<other>) => <node> * (<other>)
            TT::OpenParen | TT::OpenBracket => true,
            // <num><var> => <num> * <var>
            TT::Variable(_) | TT::VariablePattern(_) | TT::ConstPattern(_) | TT::AnyPattern(_)
                if node.is_const() =>
            {
                true
            }
            _ => false,
        } && !self.has_stmt_break();
        if insert_synthetic_mult {
            let next_span = self.peek().span;
            let bw_cur_and_next = (tok_span.hi, next_span.lo);
            self.input()
                .push_front(Token::new(TT::Mult, bw_cur_and_next, bw_cur_and_next));
        }

        node
    }

    /// Creates diagnostics for extra tokens following a primary item.
    /// All remaining tokens will be consumed in the construction of the diagnostic.
    ///
    /// `additional_diags` applies additional diagnostic messages to the extra tokens diagnostic,
    /// if one is produced.
    fn extra_tokens_diag(&mut self, additional_diags: impl Fn(Diagnostic, Span) -> Diagnostic) {
        while matches!(self.peek().ty, TT::CloseParen | TT::CloseBracket) {
            let tok = self.next();
            self.push_diag(UnmatchedClosingDelimiter!(tok.span, tok.ty));
        }

        if self.done() {
            return;
        }

        let first_tok_span = self.peek().span;
        let Span { lo, mut hi } = first_tok_span;
        while self.peek().ty != TT::EOF {
            hi = self.next().span.hi;
        }
        let diag = additional_diags(ExtraTokens!(lo..hi), first_tok_span);
        self.push_diag(diag);
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
