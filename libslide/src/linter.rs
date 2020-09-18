//! Provides linter-like diagnostics for a slide program.

macro_rules! explain_lint {
    ($(#[doc = $doc:expr])+ $code:ident: $linter:ident) => {
        use crate::diagnostics::DiagnosticRecord;

        $(#[doc = $doc])+
        impl<'a> DiagnosticRecord for $linter<'a> {
            const CODE: &'static str = stringify!($code);
            const EXPLANATION: &'static str = concat!($($doc, "\n"),+);
        }
    };
}

mod stmt;
use stmt::*;

mod expr_pat;
use expr_pat::*;

use crate::diagnostics::{Diagnostic, DiagnosticRecord, DiagnosticRegistry};
use crate::grammar::{Grammar, InternedExprPat, StmtList};

/// Describes a slide program linter. A `Linter` is implemented on a slide [Grammar].
///
/// [Grammar]: [crate::grammar::Grammar].
pub trait LintRule<'a, G>
where
    Self: DiagnosticRecord,
    G: Grammar,
{
    /// Lints a grammar given the original source code of the program.
    fn lint(grammar: &G, source: &'a str) -> Vec<Diagnostic>;
}

/// Describes the configuration to use when linting a slide grammar.
pub struct LintConfig {
    stmt_linters: Vec<StmtLintRule>,
    expr_pat_linters: Vec<ExprPatLintRule>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            stmt_linters: vec![
                StmtLintRule::UnarySeriesLinter,
                StmtLintRule::RedundantNestingLinter,
            ],
            expr_pat_linters: vec![ExprPatLintRule::SimilarNamesLinter],
        }
    }
}

impl DiagnosticRegistry for LintConfig {
    /// All lint codes and their explanations.
    fn codes_with_explanations() -> Vec<(&'static str, &'static str)> {
        let mut vec = Vec::new();
        vec.extend(StmtLintRule::all_explanations());
        vec.extend(ExprPatLintRule::all_explanations());
        vec
    }
}

/// Lints a slide [statement list](crate::grammar::StmtList).
pub fn lint_stmt(stmt_list: &StmtList, source: &str) -> Vec<Diagnostic> {
    let config = LintConfig::default();
    let mut diags = vec![];
    for stmt in stmt_list.iter() {
        for linter in config.stmt_linters.iter() {
            diags.extend(linter.lint(stmt, source))
        }
    }
    diags
}

/// Lints a slide [expression pattern](crate::grammar::ExprPat).
pub fn lint_expr_pat(expr_pat: &InternedExprPat, source: &str) -> Vec<Diagnostic> {
    let config = LintConfig::default();
    let mut diags = vec![];
    for linter in config.expr_pat_linters {
        diags.extend(linter.lint(expr_pat, source))
    }
    diags
}
