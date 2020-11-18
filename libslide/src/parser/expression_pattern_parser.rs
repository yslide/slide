use super::{errors::*, ParseResult, Parser};
use crate::common::Span;
use crate::diagnostics::Diagnostic;
use crate::grammar::*;
use crate::scanner::types::Token;
use crate::utils::PeekIter;

/// Parses a tokenized slide expression pattern, emitting the result and any diagnostics.
pub fn parse(input: Vec<Token>) -> ParseResult<RcExprPat> {
    let mut parser = ExpressionPatternParser::new(input);
    let program = parser.parse();
    let diagnostics = parser.diagnostics;
    ParseResult {
        program,
        diagnostics,
    }
}

pub struct ExpressionPatternParser {
    _input: PeekIter<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl ExpressionPatternParser {
    fn new(input: Vec<Token>) -> Self {
        Self {
            _input: PeekIter::new(input.into_iter()),
            diagnostics: vec![],
        }
    }
}

impl Parser<RcExprPat> for ExpressionPatternParser {
    type Expr = RcExprPat;

    fn input(&mut self) -> &mut PeekIter<Token> {
        &mut self._input
    }

    fn push_diag(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn parse(&mut self) -> RcExprPat {
        let parsed = self.expr();
        if !self.done() {
            self.extra_tokens_diag(|d, _| d);
        }
        parsed
    }

    fn parse_float(&mut self, f: f64, span: Span) -> Self::Expr {
        rc_expr_pat!(ExprPat::Const(f), span)
    }

    fn parse_variable(&mut self, name: String, span: Span) -> Self::Expr {
        self.push_diag(IllegalVariable!(span, name));
        rc_expr_pat!(ExprPat::VarPat(name), span)
    }

    fn parse_var_pattern(&mut self, name: String, span: Span) -> Self::Expr {
        rc_expr_pat!(ExprPat::VarPat(name), span)
    }

    fn parse_const_pattern(&mut self, name: String, span: Span) -> Self::Expr {
        rc_expr_pat!(ExprPat::ConstPat(name), span)
    }

    fn parse_any_pattern(&mut self, name: String, span: Span) -> Self::Expr {
        rc_expr_pat!(ExprPat::AnyPat(name), span)
    }

    fn has_stmt_break(&mut self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    parser_tests! {
        expr_pat

        pattern:                 "$a"
        pattern_in_op_left:      "$a + 1"
        pattern_in_op_right:     "1 + $a"
    }
}
