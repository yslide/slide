//! Lints for an expression pattern in a slide program.

mod similar_names;
use similar_names::*;

use super::{LintExplanation, LintRule};
use crate::diagnostics::Diagnostic;
use crate::grammar::InternedExprPat;

use std::collections::HashMap;

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

            pub fn all_explanations() -> HashMap<&'static str, &'static str> {
                let mut map = HashMap::new();
                $(map.insert($linter::CODE, $linter::EXPLANATION);)*
                map
            }
        }
    };
}

define_expr_pat_lints! {
    SimilarNamesLinter,
}
