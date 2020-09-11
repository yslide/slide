//! Provides linter-like diagnostics for a slide program.

macro_rules! explain_lint {
    ($(#[doc = $doc:expr])+ $code:ident: $linter:ident) => {
        use crate::linter::LintExplanation;

        $(#[doc = $doc])+
        impl<'a> LintExplanation for $linter<'a> {
            const CODE: &'static str = stringify!($code);
            const EXPLANATION: &'static str = concat!($($doc, "\n"),+);
        }
    };
}

mod stmt;
use stmt::*;

mod expr_pat;
use expr_pat::*;

use crate::diagnostics::Diagnostic;
use crate::grammar::{Grammar, InternedExprPat, Stmt};

use std::collections::HashMap;

/// Describes a slide program linter. A `Linter` is implemented on a slide [Grammar].
///
/// [Grammar]: [crate::grammar::Grammar].
pub trait LintRule<'a, G>
where
    Self: LintExplanation,
    G: Grammar,
{
    /// Lints a grammar given the original source code of the program.
    fn lint(grammar: &G, source: &'a str) -> Vec<Diagnostic>;
}

pub trait LintExplanation {
    const CODE: &'static str;
    const EXPLANATION: &'static str;
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

impl LintConfig {
    /// All lint codes and their explanations.
    pub fn all_codes_with_explanations() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();
        map.extend(StmtLintRule::all_explanations());
        map.extend(ExprPatLintRule::all_explanations());
        map
    }
}

/// Lints a slide [statement](crate::grammar::Stmt).
pub fn lint_stmt(stmt: &Stmt, source: &str) -> Vec<Diagnostic> {
    let config = LintConfig::default();
    let mut diags = vec![];
    for linter in config.stmt_linters {
        diags.extend(linter.lint(stmt, source))
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

#[cfg(test)]
mod test {
    use super::LintConfig;

    /// Each code must be of form Ldddd, where d is a digit.
    #[test]
    fn code_format() {
        let lint_codes = LintConfig::all_codes_with_explanations();

        for code in lint_codes.keys() {
            assert_eq!(code.len(), 5);
            assert!(code.starts_with('L'));
            for ch in code.chars().skip(1) {
                assert!(matches!(ch, '0'..='9'));
            }
        }
    }

    #[test]
    fn no_conflicting_codes() {
        let LintConfig {
            stmt_linters,
            expr_pat_linters,
        } = LintConfig::default();

        let num_lints = stmt_linters.len() + expr_pat_linters.len();
        let num_lint_codes = LintConfig::all_codes_with_explanations().keys().count();

        assert_eq!(num_lints, num_lint_codes);
    }
}
