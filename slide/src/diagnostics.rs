//! User-facing slide diagnostics.
//!
//! The diagnostics module translates [libslide diagnostics](libslide::diagnostics) into a form
//! pleasant for standard output.

use libslide::diagnostics::{AssociatedDiagnostic, Autofix, Diagnostic, DiagnosticKind, Edit};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

pub fn sanitize_source_for_diagnostics(source: &str) -> String {
    let source = source.to_owned();
    source + " " // we might emit an EOF diagnostic, so add extra space.
}

pub fn emit_slide_diagnostics(
    file: Option<&str>,
    source: &str,
    diagnostics: &[Diagnostic],
    color: bool,
) -> String {
    if diagnostics.is_empty() {
        return String::new();
    }

    let last_i = diagnostics.len() - 1;
    let mut emitted_diagnostics = String::new();

    for (i, diagnostic) in diagnostics.iter().enumerate() {
        let main_annotation_type = convert_diagnostic_kind(&diagnostic.kind);
        let mut annotations = Vec::with_capacity(diagnostic.associated_diagnostics.len() + 1);

        // The first annotation always points to the code that generated this diagnostic.
        // Generally, this is immediately followed by an autofix message (if we have one) over the
        //   code.
        // However, to provide a nice interface, in the case that the diagnostic does not have any
        //   message, and we have an autofix message to provide, we merge the autofix message into
        //   the diagnostic message, and thus only annotate the code once.
        let autofix = diagnostic
            .autofix
            .as_ref()
            .map(|af| convert_autofix(&af, diagnostic.span.into()))
            .unwrap_or_else(SourceAnnotationShim::dummy);
        let main_label = diagnostic.msg.clone().unwrap_or_default();

        let has_autofix = !autofix.is_dummy();
        let merge_autofix = main_label.is_empty() && has_autofix;

        let (first_label, first_annotation_type) = if merge_autofix {
            (&autofix.label, AnnotationType::Help)
        } else {
            (&main_label, main_annotation_type)
        };

        annotations.push(SourceAnnotation {
            label: first_label,
            annotation_type: first_annotation_type,
            range: diagnostic.span.into(),
        });
        if has_autofix && !merge_autofix {
            annotations.push(autofix.deshim());
        }

        // Add the associated diagnostics as the remaining annotations for the main diagnostic.
        for associated_diagnostic in diagnostic.associated_diagnostics.iter() {
            annotations.push(SourceAnnotation {
                label: &associated_diagnostic.msg,
                annotation_type: convert_diagnostic_kind(&associated_diagnostic.kind),
                range: associated_diagnostic.span.into(),
            });
        }

        // Add the unspanned associated diagnostics to the diagnostic footer.
        let mut footer = Vec::with_capacity(1);
        for associated_diagnostic in diagnostic.unspanned_associated_diagnostics.iter() {
            footer.push(convert_associated_diagnostic(associated_diagnostic));
        }

        let snippet = Snippet {
            title: Some(Annotation {
                label: Some(&diagnostic.title),
                id: Some(diagnostic.code),
                annotation_type: main_annotation_type,
            }),
            footer,
            slices: vec![Slice {
                source: &source,
                line_start: 1,
                origin: file,
                fold: true,
                annotations,
            }],
            opt: FormatOptions {
                color,
                ..Default::default()
            },
        };
        let suffix = if i != last_i { "\n" } else { "" };
        emitted_diagnostics.push_str(&format!("{}\n{}", DisplayList::from(snippet), suffix));
    }
    emitted_diagnostics
}

/// Converts a slide DiagnosticKind to an AnnotationType.
fn convert_diagnostic_kind(diagnostic_kind: &DiagnosticKind) -> AnnotationType {
    match diagnostic_kind {
        DiagnosticKind::Error => AnnotationType::Error,
        DiagnosticKind::Warning => AnnotationType::Warning,
        DiagnosticKind::Note => AnnotationType::Note,
        DiagnosticKind::Help => AnnotationType::Help,
    }
}

/// Converts a slide AssociatedDiagnostic to a SourceAnnotation.
fn convert_associated_diagnostic(diagnostic: &AssociatedDiagnostic) -> Annotation {
    Annotation {
        label: Some(&diagnostic.msg),
        id: None,
        annotation_type: convert_diagnostic_kind(&diagnostic.kind),
    }
}

/// Converts a slide [`Autofix`](Autofix) to a [`SourceAnnotationShim`](SourceAnnotationShim),
/// which can then be converted to a `SourceAnnotation`.
fn convert_autofix(autofix: &Autofix, range: (usize, usize)) -> SourceAnnotationShim {
    let suffix = match &autofix.fix {
        Edit::Delete => "".to_owned(),
        Edit::Replace(replacement) => format!(": `{}`", replacement),
    };
    SourceAnnotationShim {
        range,
        label: format!("{}{}", autofix.msg, suffix),
        annotation_type: AnnotationType::Help,
    }
}

/// `SourceAnnotation` has a `label` field of type `&'a str`, but
/// [`convert_autofix`](convert_autofix) creates a fresh `String` when translating the diagnostic
/// message. Thus it would be impossible for us to directly return a type of `SourceAnnotation` from
/// there, because the lifetime of `label` would be only as long as the local reference to the
/// message we created and was destroyed when the function exits.
///
/// For this reason, we provide a shim a captures the `String` label in a container returned from
/// such functions. The shim container is owned the caller, and so the caller can then create a
/// `SourceAnnotation` as desired, lasting the lifetime of the shim.
struct SourceAnnotationShim {
    range: (usize, usize),
    label: String,
    annotation_type: AnnotationType,
}
impl SourceAnnotationShim {
    fn deshim(&self) -> SourceAnnotation {
        SourceAnnotation {
            range: self.range,
            label: &self.label,
            annotation_type: self.annotation_type,
        }
    }

    const DUMMY_RANGE: (usize, usize) = (31459, 26535);

    /// Because sometimes `SourceAnnotation`s needing a shim are created `Option`s, there is a need
    /// to provide a shim whether the `Option` has a value or not. The "dummy shim" provides a
    /// workaround for this need; if a user needs to unwrap the `Option` to create a shim, they can
    /// default to the dummy shim and use [`is_dummy`](Self::is_dummy) to check if the shim is
    /// synthetic or real.
    fn dummy() -> Self {
        Self {
            range: Self::DUMMY_RANGE,
            label: String::new(),
            annotation_type: AnnotationType::Info,
        }
    }

    /// Returns whether the shim is synthetic. See [`dummy`](Self::dummy).
    fn is_dummy(&self) -> bool {
        self.range == Self::DUMMY_RANGE
    }
}
