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
}

/// A diagnostic for slide source code.
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub span: Span,
    pub msg: String,
    pub associated_diagnostics: Vec<Diagnostic>,
}

impl Diagnostic {
    /// Creates an error diagnostic at a span.
    pub(crate) fn span_err<S, M>(span: S, err: M) -> Diagnostic
    where
        S: Into<Span>,
        M: Into<String>,
    {
        Diagnostic {
            kind: DiagnosticKind::Error,
            span: span.into(),
            msg: err.into(),
            associated_diagnostics: Vec::with_capacity(2),
        }
    }

    /// Adds a note to the diagnostic, possibly at a different span.
    pub(crate) fn with_note<S, M>(mut self, span: S, note: M) -> Diagnostic
    where
        S: Into<Span>,
        M: Into<String>,
    {
        self.associated_diagnostics.push(Diagnostic {
            kind: DiagnosticKind::Note,
            span: span.into(),
            msg: note.into(),
            associated_diagnostics: vec![],
        });
        self
    }
}
