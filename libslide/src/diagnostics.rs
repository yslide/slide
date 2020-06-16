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
    pub kind: DiagnosticKind,
    pub span: Span,
    pub msg: String,
}

/// A diagnostic for slide source code.
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub span: Span,
    pub title: String,
    pub msg: Option<String>,
    pub associated_diagnostics: Vec<AssociatedDiagnostic>,
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
