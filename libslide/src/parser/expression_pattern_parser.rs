use super::{errors::*, str2rat, Parser};
use crate::diagnostics::{Diagnostic, DiagnosticRecord};
use crate::grammar::*;
use crate::scanner::types::Token;
use crate::utils::PeekIter;
use crate::{ProgramContext, Span};

/// Parses a tokenized slide expression pattern, emitting the result and any diagnostics.
pub fn parse(input: Vec<Token>, program_context: &ProgramContext) -> (RcExprPat, Vec<Diagnostic>) {
    let mut parser = ExpressionPatternParser::new(input, program_context);
    (parser.parse(), parser.diagnostics)
}

pub struct ExpressionPatternParser<'a> {
    _input: PeekIter<Token>,
    diagnostics: Vec<Diagnostic>,
    context: &'a ProgramContext,
}

impl<'a> ExpressionPatternParser<'a> {
    fn new(input: Vec<Token>, context: &'a ProgramContext) -> Self {
        Self {
            _input: PeekIter::new(input.into_iter()),
            diagnostics: vec![],
            context,
        }
    }
}

impl<'a> Parser<RcExprPat> for ExpressionPatternParser<'a> {
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

    fn parse_num(&mut self, num: String, span: Span) -> Self::Expr {
        let num = str2rat(&num, self.context.prec);
        rc_expr_pat!(ExprPat::Const(num), span)
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
