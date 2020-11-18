explain_lint! {
    ///The unary series lint detects trivially-reducible chains of unary operators.
    ///
    ///For example, the following chains of unary expressions can be reduced to a more trivial form:
    ///
    ///```text
    ///---1   -> -1
    ///+++1   ->  1
    ///+-+-+- -> -1
    ///```
    ///
    ///Chaining unary operators is not standard style in mathematical expressions and can be
    ///misleading. For example, `--x` may be interpreted to be the prefix decrement operator available
    ///in some computer programming languages, which is absent in canonical mathematical notation.
    L0002: UnarySeriesLinter
}

use crate::linter::LintRule;

use crate::common::Span;
use crate::diagnostics::{Autofix, Diagnostic, Edit};
use crate::grammar::visit::StmtVisitor;
use crate::grammar::*;

pub struct UnarySeriesLinter<'a> {
    source: &'a str,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> UnarySeriesLinter<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            diagnostics: vec![],
        }
    }
}

impl<'a> StmtVisitor<'a> for UnarySeriesLinter<'a> {
    fn visit_unary(&mut self, expr: &'a UnaryExpr<RcExpr>, start_span: Span) {
        let mut is_neg = expr.op == UnaryOperator::SignNegative;
        let mut nested = &expr.rhs;
        let mut count = 1;
        while let Expr::UnaryExpr(UnaryExpr { op, rhs }) = nested.as_ref() {
            if op == &UnaryOperator::SignNegative {
                is_neg = !is_neg;
            }
            nested = rhs;
            count += 1;
        }

        if count > 1 {
            let span = start_span.to(nested.span);
            let inner_expr = nested.span.over(self.source);
            let reduced_expr = format!("{}{}", if is_neg { "-" } else { "" }, inner_expr);

            self.diagnostics.push(
                Diagnostic::span_warn(
                    span,
                    "Trivially reducible unary operator chain",
                    Self::CODE,
                    None,
                )
                .with_autofix(Autofix::for_sure(
                    "reduce this expression",
                    Edit::Replace(reduced_expr),
                )),
            )
        }

        self.visit_expr(nested);
    }
}

impl<'a> LintRule<'a, StmtList> for UnarySeriesLinter<'a> {
    fn lint(stmt_list: &StmtList, source: &'a str) -> Vec<Diagnostic> {
        let mut linter = Self::new(&source);
        linter.visit_stmt_list(stmt_list);
        linter.diagnostics
    }
}
