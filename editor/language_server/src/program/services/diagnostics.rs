//! Module `diagnostics` marshalls between [libslide diagnostics](libslide::diagnostic::Diagnostic)
//! and LSP types.

use super::local_response::*;

use libslide::diagnostics as s;
use tower_lsp::lsp_types::*;

pub fn convert_diagnostics(
    diagnostics: &[s::Diagnostic],
    provider: &str,
    uri: &Url,
) -> Vec<LocalDiagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| LocalDiagnostic {
            span: diagnostic.span,
            severity: to_severity(&diagnostic.kind),
            code: diagnostic.code.to_string(),
            source: provider.to_string(),
            message: flatten_diagnostic_msg(diagnostic),
            related_information: flatten_related(diagnostic, uri),
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
) -> Vec<LocalDiagnosticRelatedInformation> {
    diagnostic
        .associated_diagnostics
        .iter()
        .chain(diagnostic.unspanned_associated_diagnostics.iter())
        .map(|ad| LocalDiagnosticRelatedInformation {
            location: LocalLocation {
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
