//! Lints for an expression pattern in a slide program.

mod similar_names;
use similar_names::*;

use super::{DiagnosticRecord, LintRule};
use crate::diagnostics::Diagnostic;
use crate::grammar::InternedExprPat;

macro_rules! define_expr_pat_lints {
    ($($linter:ident,)*) => {
        /// A lint rule applying to a statement in a slide program.
        pub enum ExprPatLintRule {
            $($linter),*
        }

        impl ExprPatLintRule {
            pub fn lint(&self, expr_pat: &InternedExprPat, source: &str) -> Vec<Diagnostic> {
                match self {
                    $(Self::$linter => $linter::lint(expr_pat, source)),*
                }
            }

            pub fn all_explanations() -> Vec<(&'static str, &'static str)> {
                let mut vec = Vec::new();
                $(vec.push(($linter::CODE, $linter::EXPLANATION));)*
                vec
            }
        }
    };
}

define_expr_pat_lints! {
    SimilarNamesLinter,
}
