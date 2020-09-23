explain_lint! {
    ///The similar names lint detects expression patterns with very similar names.
    ///
    ///For example, the following pattern expression has different patterns with the same suffix "a":
    ///
    ///```text
    ///$a + #a + _a + $a
    ///```
    ///
    ///While this is expression is semantically valid, it can be difficuly to read and misleading,
    ///since "a" is used in three separate and independent patterns. A clearer expression would be
    ///
    ///```text
    ///$a + #b + _c + $a
    ///```
    L0003: SimilarNamesLinter
}

use crate::common::Span;
use crate::diagnostics::Diagnostic;
use crate::grammar::*;
use crate::linter::LintRule;

use std::collections::{BTreeMap, BTreeSet};

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
enum NameKind {
    Var,
    Const,
    Any,
}

impl std::fmt::Display for NameKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            NameKind::Var => "var",
            NameKind::Const => "const",
            NameKind::Any => "any",
        })
    }
}

#[derive(Default, Debug)]
struct NameCollection {
    var_pat: Vec<Span>,
    const_pat: Vec<Span>,
    any_pat: Vec<Span>,
}

impl NameCollection {
    fn used_patterns(&self) -> BTreeSet<NameKind> {
        let mut used = BTreeSet::new();
        if !self.var_pat.is_empty() {
            used.insert(NameKind::Var);
        }
        if !self.const_pat.is_empty() {
            used.insert(NameKind::Const);
        }
        if !self.any_pat.is_empty() {
            used.insert(NameKind::Any);
        }
        used
    }

    fn has_conflicts(&self) -> bool {
        self.used_patterns().len() > 1
    }

    fn all_spans_sorted(&self) -> Vec<(NameKind, Span)> {
        let mut spans =
            Vec::with_capacity(self.var_pat.len() + self.const_pat.len() + self.any_pat.len());
        spans.extend(self.var_pat.iter().map(|sp| (NameKind::Var, *sp)));
        spans.extend(self.const_pat.iter().map(|sp| (NameKind::Const, *sp)));
        spans.extend(self.any_pat.iter().map(|sp| (NameKind::Any, *sp)));
        spans.sort_by(|a, b| a.1.cmp(&b.1));
        spans
    }
}

#[derive(Default)]
pub struct SimilarNamesLinter<'a> {
    names: BTreeMap<&'a str, NameCollection>,
}

impl<'a> SimilarNamesLinter<'a> {
    fn check_names(self) -> Vec<Diagnostic> {
        self.names
            .into_iter()
            .filter_map(|(name, collection)| {
                if !collection.has_conflicts() {
                    return None;
                }

                let mut spans = collection.all_spans_sorted().into_iter();

                let (first_kind, first_span) = spans.next().unwrap();

                let other_spans: Vec<_> =
                    spans.filter(|(kind, _span)| kind != &first_kind).collect();

                let other_kinds: BTreeSet<_> =
                    other_spans.iter().map(|(kind, _span)| kind).collect();
                let num_other_kinds = other_kinds.len();
                let mut other_kinds = other_kinds.into_iter();

                let other_kinds = if num_other_kinds == 1 {
                    other_kinds.next().unwrap().to_string()
                } else {
                    format!(
                        "{} and {}",
                        other_kinds.next().unwrap(),
                        other_kinds.next().unwrap()
                    )
                };

                let mut diag = Diagnostic::span_warn(
                    first_span,
                    format!("Similar name \"{}\" used by multiple patterns", name),
                    Self::CODE,
                    if other_spans.len() == 1 {
                        if other_kinds.starts_with('a') {
                            format!("\"{}\" is used by an {} pattern as well", name, other_kinds)
                        } else {
                            format!("\"{}\" is used by a {} pattern as well", name, other_kinds)
                        }
                    } else {
                        format!("\"{}\" is used by {} patterns as well", name, other_kinds)
                    },
                );

                for (kind, span) in other_spans.iter() {
                    diag = diag.with_spanned_note(*span, format!("{} pattern here", kind))
                }

                Some(diag)
            })
            .collect()
    }
}

impl<'a> ExprPatVisitor<'a> for SimilarNamesLinter<'a> {
    fn visit_var_pat(&mut self, var_pat: &'a str, span: Span) {
        let name = &var_pat[1..];
        self.names.entry(name).or_default().var_pat.push(span);
    }

    fn visit_const_pat(&mut self, const_pat: &'a str, span: Span) {
        let name = &const_pat[1..];
        self.names.entry(name).or_default().const_pat.push(span);
    }

    fn visit_any_pat(&mut self, any_pat: &'a str, span: Span) {
        let name = &any_pat[1..];
        self.names.entry(name).or_default().any_pat.push(span);
    }
}

impl<'a> LintRule<'a, RcExprPat> for SimilarNamesLinter<'a> {
    fn lint(expr_pat: &RcExprPat, _source: &'a str) -> Vec<Diagnostic> {
        let mut linter = Self::default();
        linter.visit(expr_pat);
        linter.check_names()
    }
}
