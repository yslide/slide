//! libslide's diagnostic module.
//!
//! libslide does not emit user-facing diagnostic information itself, so the diagnostics returned
//! by libslide should be
//!
//! - as complete as possible, so that a consumer can process as little or as much information as
//!   they want
//! - easily transformable into some output form by downstream customers (namely the slide app)

use crate::common::Span;
use crate::{LintConfig, ParseErrors, PartialEvaluatorErrors, ScanErrors};

use std::collections::HashMap;

/// The kind of a slide diagnostic.
#[derive(PartialEq)]
pub enum DiagnosticKind {
    /// An error diagnostic. Generally, this diagnostic should be emitted for unrecoverable errors.
    /// In other cases, a warning or a note may be more applicable.
    Error,
    /// A warning diagnostic describes something that is not a strict error, but may not match
    /// canonical style, usage patterns, or otherwise may be error-prone.
    Warning,
    /// A note diagnostic is a generic annotation with no specific connotation like `error`. It can
    /// be particularly useful as an associated diagnostic, for example in expanding on a primary
    /// error.
    Note,
    /// A help diagnostic should instruct the user how their code can be changed to work correctly
    /// with slide.
    Help,
}

/// A secondary diagnostic associated with a primary `Diagnostic`.
pub struct AssociatedDiagnostic {
    /// The diagnostic kind.
    pub kind: DiagnosticKind,
    /// Source location for which the diagnostic is applicable.
    pub span: Span,
    /// Diagnostic message.
    pub msg: String,
}

/// A diagnostic for slide source code.
pub struct Diagnostic {
    /// The diagnostic kind.
    pub kind: DiagnosticKind,
    /// The diagnostic code.
    pub code: Option<&'static str>,
    /// Source location for which the diagnostic is applicable.
    pub span: Span,
    /// A summarizing title for the diagnostic.
    pub title: String,
    /// Diagnostic message.
    pub msg: Option<String>,
    /// Any additional diagnostics associated with this one.
    /// The additional diagnostics may or may not cover the same span as this one.
    pub associated_diagnostics: Vec<AssociatedDiagnostic>,
    /// Any additional diagnostics associated with this one, not explicitly covering any span.
    /// Implicitly, these diagnostics cover the span of the primary one.
    pub unspanned_associated_diagnostics: Vec<AssociatedDiagnostic>,
}

/// Describes a code and detailed explanation for a diagnostic.
pub trait DiagnosticRecord {
    /// Diagnostic code.
    const CODE: &'static str;
    /// Detailed diagnostic explanation.
    const EXPLANATION: &'static str;
}

/// Describes an individual registry of slide diagnostics.
pub trait DiagnosticRegistry {
    /// Retrieves all diagnostic codes owned by this registry and their explanations.
    fn codes_with_explanations() -> Vec<(&'static str, &'static str)>;
}

macro_rules! span_diag {
    ($(#[doc = $doc:expr] $name:ident as $kind:ident)*) => {$(
        #[doc = $doc]
        pub(crate) fn $name<S, M, C, N>(span: S, title: M, code:C, msg: N) -> Diagnostic
        where
            S: Into<Span>,
            M: Into<String>,
            C: Into<Option<&'static str>>,
            N: Into<Option<String>>,
        {
            Diagnostic {
                kind: DiagnosticKind::$kind,
                span: span.into(),
                title: title.into(),
                code: code.into(),
                msg: msg.into(),
                associated_diagnostics: Vec::with_capacity(2),
                unspanned_associated_diagnostics: Vec::with_capacity(2),
            }
        }
    )*}
}

macro_rules! with_diag {
    ($(#[doc = $doc:expr] $name:ident as $kind:ident)*) => {$(
        #[doc = $doc]
        pub(crate) fn $name<M>(mut self, note: M) -> Diagnostic
        where
            M: Into<String>,
        {
            self.unspanned_associated_diagnostics
                .push(AssociatedDiagnostic {
                    kind: DiagnosticKind::$kind,
                    span: self.span,
                    msg: note.into(),
                });
            self
        }
    )*}
}

macro_rules! with_spanned_diag {
    ($(#[doc = $doc:expr] $name:ident as $kind:ident)*) => {$(
        #[doc = $doc]
        pub(crate) fn $name<S, M>(mut self, span: S, note: M) -> Diagnostic
        where
            S: Into<Span>,
            M: Into<String>,
        {
            self.associated_diagnostics.push(AssociatedDiagnostic {
                kind: DiagnosticKind::$kind,
                span: span.into(),
                msg: note.into(),
            });
            self
        }
    )*}
}

impl Diagnostic {
    span_diag! {
        /// Creates an error diagnostic at a span.
        span_err as Error
        /// Creates a warning diagnostic at a span.
        span_warn as Warning
    }

    with_diag! {
        /// Adds a note to the diagnostic.
        with_note as Note
        /// Adds a help message to the diagnostic.
        with_help as Help
    }

    with_spanned_diag! {
        /// Adds an error to the diagnostic, possibly at a different span.
        with_spanned_err as Error
        /// Adds a warning to the diagnostic, possibly at a different span.
        with_spanned_warn as Warning
        /// Adds a help message to the diagnostic, possibly at a different span.
        with_spanned_help as Help
        /// Adds a note to the diagnostic, possibly at a different span.
        with_spanned_note as Note
    }
}

macro_rules! include_diagnostic_registries {
    ($($registry:ident)*) => {
        impl Diagnostic {
            /// All diagnostic codes and their explanations.
            pub fn all_codes_with_explanations() -> HashMap<&'static str, &'static str> {
                let mut map = HashMap::new();
                $(map.extend($registry::codes_with_explanations());)*
                map
            }
        }

        #[cfg(test)]
        mod check_codes {
            use super::{Diagnostic, DiagnosticRegistry};
            use crate::*;

            use std::collections::{HashMap, BTreeSet};

            #[test]
            fn check_conflicts() {
                let mut vec = Vec::new();
                $(vec.extend($registry::codes_with_explanations());)*
                assert_eq!(vec.len(), Diagnostic::all_codes_with_explanations().len());
            }

            /// Each code must be of form Sdddd, where S is L/S/P and d is a digit.
            #[test]
            fn check_format() {
                let codes = Diagnostic::all_codes_with_explanations();

                for code in codes.keys() {
                    assert_eq!(code.len(), 5);
                    assert!(matches!(
                        code.chars().next(),
                        Some('L') | Some('S') | Some('P') | Some('V')
                    ));
                    for ch in code.chars().skip(1) {
                        assert!(matches!(ch, '0'..='9'));
                    }
                }
            }

            /// No code numbers should be skipped.
            #[test]
            fn check_density() {
                let codes = Diagnostic::all_codes_with_explanations();
                let mut segments: HashMap<&str, BTreeSet<usize>> = HashMap::new();

                for code in codes.keys() {
                    segments.entry(&code[0..1]).or_default().insert(code[1..].parse().unwrap());
                }

                for (segment, codes) in segments {
                    let expected_codes: BTreeSet<_> = (1..=codes.len()).into_iter().collect();
                    let missing: BTreeSet<_> = expected_codes.difference(&codes).collect();
                    let unexpected: BTreeSet<_> = codes.difference(&expected_codes).collect();
                    assert!(missing.is_empty(), r#"Expected {} "{}" codes
Missing: {:?}
Unexpected: {:?}"#, expected_codes.len(), segment, missing, unexpected);
                }
            }
        }
    }
}

include_diagnostic_registries! {
    LintConfig
    ParseErrors
    ScanErrors
    PartialEvaluatorErrors
}
