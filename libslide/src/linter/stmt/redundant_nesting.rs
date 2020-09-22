explain_lint! {
    ///The redundant nesting lint detects redundant nesting of expressions in parantheses or
    ///brackets.
    ///
    ///For example, the following nestings are redundant and can be reduced to a single nesting:
    ///
    ///```text
    ///((1))     -> (1)
    ///[[1]]     -> [1]
    ///([[(1)]]) -> (1)
    ///```
    ///
    ///Redundant nestings are difficult to read and may be misleading, as generally a single nesting
    ///is expected to host an expression for precedence or clarity reasons.
    L0001: RedundantNestingLinter
}

use crate::linter::LintRule;

use crate::common::Span;
use crate::diagnostics::Diagnostic;
use crate::grammar::*;

pub struct RedundantNestingLinter<'a> {
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> RedundantNestingLinter<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            diagnostics: vec![],
        }
    }
}

impl<'a> RedundantNestingLinter<'a> {
    fn visit_nesting(&mut self, mut expr: &'a InternedExpr, span: Span) {
        let mut nestings = 1;
        while let Expr::Parend(inner) | Expr::Bracketed(inner) = expr.as_ref() {
            expr = inner;
            nestings += 1;
        }

        if nestings > 1 {
            let opener = &self.source[span.lo..span.lo + 1];
            let closer = &self.source[span.hi - 1..span.hi];
            let inner_expr = expr.span.over(self.source);

            self.diagnostics.push(
                Diagnostic::span_warn(span, "Redundant nesting", Self::CODE, None).with_help(
                    format!(
                        r#"consider reducing this expression to "{}{}{}""#,
                        opener, inner_expr, closer
                    ),
                ),
            )
        }

        self.visit_expr(expr);
    }
}

impl<'a> StmtVisitor<'a> for RedundantNestingLinter<'a> {
    fn visit_parend(&mut self, expr: &'a InternedExpr, span: Span) {
        self.visit_nesting(expr, span);
    }

    fn visit_bracketed(&mut self, expr: &'a InternedExpr, span: Span) {
        self.visit_nesting(expr, span);
    }
}

impl<'a> LintRule<'a, StmtList> for RedundantNestingLinter<'a> {
    fn lint(stmt_list: &StmtList, source: &'a str) -> Vec<Diagnostic> {
        let mut linter = Self::new(&source);
        for stmt in stmt_list.iter() {
            linter.visit(stmt);
        }
        linter.diagnostics
    }
}
