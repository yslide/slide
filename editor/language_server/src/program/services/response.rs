use libslide::Span;
use tower_lsp::lsp_types::*;

#[derive(Debug, Clone)]
pub struct ProgramLocation {
    pub uri: Url,
    pub span: Span,
}

pub struct ProgramLocationLink {
    pub origin_selection_span: Span,
    pub target_uri: Url,
    pub target_span: Span,
    pub target_selection_span: Span,
}

pub struct ProgramHoverResponse {
    pub contents: HoverContents,
    pub span: Span,
}

pub struct ProgramHighlight {
    pub kind: DocumentHighlightKind,
    pub span: libslide::Span,
}

pub enum ProgramDefinitionResponse {
    Array(Vec<ProgramLocation>),
    Link(Vec<ProgramLocationLink>),
}

#[derive(Debug, Clone)]
pub struct ProgramDiagnosticRelatedInformation {
    pub location: ProgramLocation,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ProgramDiagnostic {
    pub span: Span,
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub source: String,
    pub title: String,
    pub display_message: String,
    pub related_information: Vec<ProgramDiagnosticRelatedInformation>,
    pub autofix: Option<libslide::diagnostics::Autofix>,
}

pub enum ProgramSymbolKind {
    Variable,
}

pub struct ProgramSymbolInformation {
    pub name: String,
    pub kind: ProgramSymbolKind,
    pub documentation: String,
    pub location: ProgramLocation,
}

pub struct ProgramTextEdit {
    pub span: Span,
    pub edit: String,
}

pub enum ProgramCannotRenameBecause {
    CursorNotOverVariable,
}

pub struct ProgramCanRenameResponse {
    pub span: Span,
    pub placeholder: String,
}

pub struct ProgramRenameResponse {
    pub uri: Url,
    pub edits: Vec<ProgramTextEdit>,
}

pub struct ProgramFoldingRanges(pub Vec<Span>);

pub struct ProgramSelectionRanges(pub Vec<Span>);

pub enum ProgramActionKind {
    DiagnosticFix,
    Rewrite,
}

pub struct ProgramAction {
    pub title: String,
    pub kind: ProgramActionKind,
    pub resolved_diagnostic: Option<ProgramDiagnostic>,
    pub uri: Url,
    pub edit: ProgramTextEdit,
    pub is_preferred: bool,
}
