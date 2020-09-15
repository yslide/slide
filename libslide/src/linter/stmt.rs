//! Lints for a statement in a slide program.

mod redundant_nesting;
mod unary_series;
use redundant_nesting::*;
use unary_series::*;

use super::{DiagnosticRecord, LintRule};
use crate::diagnostics::Diagnostic;
use crate::grammar::Stmt;

macro_rules! define_stmt_lints {
    ($($linter:ident,)*) => {
        /// A lint rule applying to a statement in a slide program.
        pub enum StmtLintRule {
            $($linter),*
        }

        impl StmtLintRule {
            pub fn lint(&self, stmt: &Stmt, source: &str) -> Vec<Diagnostic> {
                match self {
                    $(Self::$linter => $linter::lint(stmt, source)),*
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

define_stmt_lints! {
    UnarySeriesLinter,
    RedundantNestingLinter,
}
