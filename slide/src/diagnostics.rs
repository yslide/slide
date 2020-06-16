use libslide::diagnostics::{AssociatedDiagnostic, Diagnostic, DiagnosticKind};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use std::io::Write;
use termcolor::{BufferedStandardStream, ColorChoice, WriteColor};

pub fn emit_slide_diagnostics(file: Option<&str>, source: String, diagnostics: Vec<Diagnostic>) {
    let source = source + " "; // we might emit an EOF diagnostic, so add extra space.
    let mut stderr = BufferedStandardStream::stderr(ColorChoice::Auto);
    let can_color = atty::is(atty::Stream::Stderr) && stderr.supports_color();

    for diagnostic in diagnostics {
        let main_annotation_type = convert_diagnostic_kind(&diagnostic.kind);
        let mut annotations = Vec::with_capacity(diagnostic.associated_diagnostics.len() + 1);
        // The first annotation always points to the code that generated this diagnostic.
        let label = diagnostic.msg.unwrap_or_default();
        annotations.push(SourceAnnotation {
            label: &label,
            annotation_type: main_annotation_type,
            range: diagnostic.span.into(),
        });
        let mut footer = Vec::with_capacity(1);
        // Add the associated diagnostics as the remaining annotations for the main diagnostic.
        for associated_diagnostic in diagnostic.associated_diagnostics.iter() {
            footer.push(convert_associated_diagnostic(associated_diagnostic));
        }

        let snippet = Snippet {
            title: Some(Annotation {
                label: Some(&diagnostic.title),
                id: None,
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
                color: can_color,
                ..Default::default()
            },
        };
        writeln!(&mut stderr, "{}\n", DisplayList::from(snippet)).unwrap();
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

/// Converts a slide DiagnosticKind to an AnnotationType.
fn convert_diagnostic_kind(diagnostic_kind: &DiagnosticKind) -> AnnotationType {
    match diagnostic_kind {
        DiagnosticKind::Error => AnnotationType::Error,
        DiagnosticKind::Note => AnnotationType::Note,
        DiagnosticKind::Help => AnnotationType::Help,
    }
}
