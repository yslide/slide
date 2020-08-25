use super::{extra_tokens_diag, Parser};
use crate::common::Span;
use crate::diagnostics::Diagnostic;
use crate::grammar::*;
use crate::scanner::types::{Token, TokenType};
use crate::utils::{PeekIter, StringUtils};

/// Parses a tokenized slide program, emitting the result and any diagnostics.
pub fn parse(input: Vec<Token>) -> (Stmt, Vec<Diagnostic>) {
    let mut parser = ExpressionParser::new(input);
    (parser.parse(), parser.diagnostics)
}

pub struct ExpressionParser {
    _input: PeekIter<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl ExpressionParser {
    fn assignment(&mut self, var: String) -> Stmt {
        Stmt::Assignment(Assignment {
            var,
            rhs: self.expr(),
        })
    }
}

impl ExpressionParser {
    fn parse_pattern(&mut self, name: String, span: Span) -> InternedExpr {
        self.push_diag(
            Diagnostic::span_err(
                span,
                "Patterns cannot be used in an expression",
                Some("unexpected pattern".into()),
            )
            .with_help(format!(
                r#"consider using "{cut_name}" as a variable"#,
                cut_name = name.substring(1, name.len() - 1)
            )),
        );
        intern_expr!(Expr::Var(name))
    }
}

impl Parser<Stmt> for ExpressionParser {
    type Expr = InternedExpr;

    fn new(input: Vec<Token>) -> Self {
        Self {
            _input: PeekIter::new(input.into_iter()),
            diagnostics: vec![],
        }
    }

    fn input(&mut self) -> &mut PeekIter<Token> {
        &mut self._input
    }

    fn push_diag(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn parse(&mut self) -> Stmt {
        let mut next_2 = self.input().peek_map_n(2, |tok| tok.ty.clone());
        let parsed = match (next_2.pop_front(), next_2.pop_front()) {
            (Some(TokenType::Variable(name)), Some(TokenType::Equal)) => {
                self.input().next();
                self.input().next();
                self.assignment(name)
            }
            _ => Stmt::Expr(self.expr()),
        };
        if !self.done() {
            let extra_tokens_diag = extra_tokens_diag(self.input());
            self.push_diag(extra_tokens_diag);
        }
        parsed
    }

    fn parse_float(&mut self, f: f64, _span: Span) -> Self::Expr {
        intern_expr!(Expr::Const(f))
    }

    fn parse_variable(&mut self, name: String, _span: Span) -> Self::Expr {
        intern_expr!(Expr::Var(name))
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
}

#[cfg(test)]
mod tests {
    parser_tests! {
        parse_expression

        variable:                "a"
        variable_in_op_left:     "a + 1"
        variable_in_op_right:    "1 + a"
        assignment_op:           "a = 5"
        assignment_op_expr:      "a = 5 + 2 ^ 3"
    }
}
