//! libslide's diagnostic module.
//!
//! libslide does not emit user-facing diagnostic information itself, so the diagnostics returned
//! by libslide should be
//!
//! - as complete as possible, so that a consumer can process as little or as much information as
//!   they want
//! - easily transformable into some output form by downstream customers (namely the slide app)

use crate::common::Span;

/// The kind of a slide diagnostic.
pub enum DiagnosticKind {
    /// An error diagnostic. Generally, this diagnostic should be emitted for unrecoverable errors.
    /// In other cases, a warning or a note may be more applicable.
    Error,
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

impl Diagnostic {
    /// Creates an error diagnostic at a span.
    pub(crate) fn span_err<S, M, N>(span: S, title: M, err: N) -> Diagnostic
    where
        S: Into<Span>,
        M: Into<String>,
        N: Into<Option<String>>,
    {
        Diagnostic {
            kind: DiagnosticKind::Error,
            span: span.into(),
            title: title.into(),
            msg: err.into(),
            associated_diagnostics: Vec::with_capacity(2),
            unspanned_associated_diagnostics: Vec::with_capacity(2),
        }
    }

    /// Adds a note to the diagnostic.
    pub(crate) fn with_note<M>(mut self, note: M) -> Diagnostic
    where
        M: Into<String>,
    {
        self.unspanned_associated_diagnostics
            .push(AssociatedDiagnostic {
                kind: DiagnosticKind::Note,
                span: self.span,
                msg: note.into(),
            });
        self
    }

    /// Adds a help message to the diagnostic.
    pub(crate) fn with_help<M>(mut self, note: M) -> Diagnostic
    where
        M: Into<String>,
    {
        self.unspanned_associated_diagnostics
            .push(AssociatedDiagnostic {
                kind: DiagnosticKind::Help,
                span: self.span,
                msg: note.into(),
            });
        self
    }

    /// Adds a help message to the diagnostic, possibly at a different span.
    pub(crate) fn with_help_note<S, M>(mut self, span: S, note: M) -> Diagnostic
    where
        S: Into<Span>,
        M: Into<String>,
    {
        self.associated_diagnostics.push(AssociatedDiagnostic {
            kind: DiagnosticKind::Help,
            span: span.into(),
            msg: note.into(),
        });
        self
    }
}
