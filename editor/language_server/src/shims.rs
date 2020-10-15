//! Module `diagnostics` converts slide diagnostics to LSP types.

use libslide::diagnostics as s;
use libslide::Span;
use tower_lsp::lsp_types::*;

pub fn convert_diagnostics(
    diagnostics: &[s::Diagnostic],
    provider: &str,
    uri: &Url,
    source: &str,
) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| Diagnostic {
            range: to_range(&diagnostic.span, source),
            severity: Some(to_severity(&diagnostic.kind)),
            code: Some(NumberOrString::String(diagnostic.code.to_string())),
            source: Some(provider.to_string()),
            message: flatten_diagnostic_msg(diagnostic),
            related_information: Some(flatten_related(diagnostic, uri, source)),
            tags: None,
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
    source: &str,
) -> Vec<DiagnosticRelatedInformation> {
    diagnostic
        .associated_diagnostics
        .iter()
        .chain(diagnostic.unspanned_associated_diagnostics.iter())
        .map(|ad| DiagnosticRelatedInformation {
            location: Location::new(uri.clone(), to_range(&ad.span, source)),
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

pub fn to_range(span: &Span, source: &str) -> Range {
    Range::new(to_position(span.lo, source), to_position(span.hi, source))
}

// https://docs.rs/wast/25.0.2/src/wast/ast/token.rs.html#24-36
// TODO: batch this or provide a offset mapping.
pub fn to_position(offset: usize, source: &str) -> Position {
    let mut cur = 0;
    // Use split_terminator instead of lines so that if there is a `\r`,
    // it is included in the offset calculation. The `+1` values below
    // account for the `\n`.
    for (i, line) in source.split_terminator('\n').enumerate() {
        if cur + line.len() + 1 > offset {
            return Position::new(i as u64, (offset - cur) as u64);
        }
        cur += line.len() + 1;
    }
    Position::new(source.lines().count() as u64, 0)
}

pub fn to_offset(position: &Position, source: &str) -> usize {
    // Use split_terminator instead of lines so that if there is a `\r`,
    // it is included in the offset calculation. The `+1` values below
    // account for the `\n`.
    source
        .split_terminator('\n')
        .take(position.line as usize)
        .fold(0, |acc, line| acc + line.len() + 1)
        + position.character as usize
}
