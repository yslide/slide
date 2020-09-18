use super::{errors::*, extra_tokens_diag, Parser};
use crate::common::Span;
use crate::diagnostics::{Diagnostic, DiagnosticRecord};
use crate::grammar::*;
use crate::scanner::types::{Token, TokenType};
use crate::utils::{PeekIter, StringUtils};

/// Parses a tokenized slide program, emitting the result and any diagnostics.
pub fn parse(input: Vec<Token>, program: &str) -> (StmtList, Vec<Diagnostic>) {
    let mut parser = ExpressionParser::new(input, program);
    (parser.parse(), parser.diagnostics)
}

pub struct ExpressionParser<'a> {
    _input: PeekIter<Token>,
    program: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> ExpressionParser<'a> {
    fn new(input: Vec<Token>, program: &'a str) -> Self {
        Self {
            _input: PeekIter::new(input.into_iter()),
            program,
            diagnostics: vec![],
        }
    }

    fn assignment(&mut self, var: String) -> Stmt {
        Stmt::Assignment(Assignment {
            var,
            rhs: self.expr(),
        })
    }

    fn parse_stmt(&mut self) -> Stmt {
        let mut next_2 = self.input().peek_map_n(2, |tok| tok.ty.clone());
        match (next_2.pop_front(), next_2.pop_front()) {
            (Some(TokenType::Variable(name)), Some(TokenType::Equal)) => {
                self.input().next();
                self.input().next();
                self.assignment(name)
            }
            _ => Stmt::Expr(self.expr()),
        }
    }

    fn parse_pattern(&mut self, name: String, span: Span) -> InternedExpr {
        self.push_diag(IllegalPattern!(span, name));
        intern_expr!(Expr::Var(name), span)
    }
}

impl<'a> Parser<StmtList> for ExpressionParser<'a> {
    type Expr = InternedExpr;

    fn input(&mut self) -> &mut PeekIter<Token> {
        &mut self._input
    }

    fn push_diag(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn parse(&mut self) -> StmtList {
        let mut stmts = Vec::new();
        while !self.done() {
            stmts.push(self.parse_stmt());

            if !self.done() && !self.has_stmt_break() {
                let next_span = self.peek().span;
                let extra_tokens_diag = extra_tokens_diag(self.input()).with_spanned_help(
                    next_span,
                    "if you meant to specify another statement, add a newline before this token",
                );
                self.push_diag(extra_tokens_diag);
                break;
            }
        }
        StmtList::new(stmts)
    }

    fn parse_float(&mut self, f: f64, span: Span) -> Self::Expr {
        intern_expr!(Expr::Const(f), span)
    }

    fn parse_variable(&mut self, name: String, span: Span) -> Self::Expr {
        intern_expr!(Expr::Var(name), span)
    }

    fn parse_var_pattern(&mut self, name: String, span: Span) -> Self::Expr {
        self.parse_pattern(name, span)
    }

    fn parse_const_pattern(&mut self, name: String, span: Span) -> Self::Expr {
        self.parse_pattern(name, span)
    }

    fn parse_any_pattern(&mut self, name: String, span: Span) -> Self::Expr {
        self.parse_pattern(name, span)
    }

    /// Do we have another statement (on a newline)?
    fn has_stmt_break(&mut self) -> bool {
        self.peek()
            .full_span
            .clone()
            .over(self.program)
            .contains('\n')
    }
}

#[cfg(test)]
mod tests {
    parser_tests! {
        expr

        variable:                "a"
        variable_in_op_left:     "a + 1"
        variable_in_op_right:    "1 + a"
        assignment_op:           "a = 5"
        assignment_op_expr:      "a = 5 + 2 ^ 3"
    }
}
