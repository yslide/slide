//! Module `diagnostics` marshals between
//! [libslide diagnostics](libslide::diagnostics::Diagnostic) and LSP types.

use super::response::*;

use libslide::diagnostics as s;
use tower_lsp::lsp_types::*;

pub fn convert_diagnostics(
    diagnostics: &[s::Diagnostic],
    provider: &str,
    uri: &Url,
) -> Vec<ProgramDiagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| ProgramDiagnostic {
            span: diagnostic.span,
            severity: to_severity(&diagnostic.kind),
            code: diagnostic.code.to_string(),
            source: provider.to_string(),
            title: diagnostic.title.to_string(),
            display_message: flatten_diagnostic_msg(diagnostic),
            related_information: flatten_related(diagnostic, uri),
            autofix: diagnostic.autofix.clone(),
        })
        .collect()
}

fn flatten_diagnostic_msg(diagnostic: &s::Diagnostic) -> String {
    match &diagnostic.msg {
        Some(msg) => format!("{} \\ {}", diagnostic.title, msg),
        None => diagnostic.title.to_string(),
    }
}

fn flatten_related(
    diagnostic: &s::Diagnostic,
    uri: &Url,
) -> Vec<ProgramDiagnosticRelatedInformation> {
    diagnostic
        .associated_diagnostics
        .iter()
        .chain(diagnostic.unspanned_associated_diagnostics.iter())
        .map(|ad| ProgramDiagnosticRelatedInformation {
            location: ProgramLocation {
                uri: uri.clone(),
                span: ad.span,
            },
            message: ad.msg.to_string(),
        })
        .collect()
}

fn to_severity(diagnostic_kind: &s::DiagnosticKind) -> DiagnosticSeverity {
    match diagnostic_kind {
        s::DiagnosticKind::Error => DiagnosticSeverity::Error,
        s::DiagnosticKind::Warning => DiagnosticSeverity::Warning,
        s::DiagnosticKind::Note => DiagnosticSeverity::Information,
        s::DiagnosticKind::Help => DiagnosticSeverity::Hint,
    }
}
