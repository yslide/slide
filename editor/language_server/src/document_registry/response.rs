//! Module `response` describes the API for responses of language queries at the level of a
//! document, and generally correspond to the API surface of the LSP. In general, document-level
//! responses are converted from [program-level responses](crate::program::response) by the
//! implementation of the [`ToDocumentResponse`](ToDocumentResponse) trait. All such
//! implementations should reside in this module.

use crate::program::response::*;
use std::collections::HashMap;
use tower_lsp::lsp_types::*;

/// Describes how a response (namely a [program-level response](crate::program::response)) should
/// be converted to a response at the level of a [Document](super::document::Document) (namely on
/// the surface of the LSP API).
pub trait ToDocumentResponse {
    /// The document-level response targeted by the conversion.
    type DocumentResponse;

    /// Performs the conversion of `self` to the targeted
    /// [`DocumentResponse`](Self::DocumentResponse).
    fn to_document_response(
        self,
        program_offset_in_document: usize,
        document_offset_to_position: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse;
}

macro_rules! to_range {
    ($document_offset_to_position:ident, $program_offset:ident, $span:ident) => {
        Range::new(
            $document_offset_to_position($span.lo + $program_offset),
            $document_offset_to_position($span.hi + $program_offset),
        )
    };
}

enum ServerErrorCode {
    // Rename errors
    CursorNotOverVariable = 100,
}

impl ToDocumentResponse for ProgramLocation {
    type DocumentResponse = Location;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        let ProgramLocation { uri, span } = self;
        Location::new(uri, to_range!(o2p, program_offset, span))
    }
}

impl ToDocumentResponse for Vec<ProgramLocation> {
    type DocumentResponse = Vec<Location>;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        self.into_iter()
            .map(|ProgramLocation { uri, span }| {
                Location::new(uri, to_range!(o2p, program_offset, span))
            })
            .collect()
    }
}

impl ToDocumentResponse for Vec<ProgramLocationLink> {
    type DocumentResponse = Vec<LocationLink>;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        self.into_iter()
            .map(
                |ProgramLocationLink {
                     origin_selection_span,
                     target_uri,
                     target_span,
                     target_selection_span,
                 }| {
                    LocationLink {
                        origin_selection_range: Some(to_range!(
                            o2p,
                            program_offset,
                            origin_selection_span
                        )),
                        target_uri,
                        target_range: to_range!(o2p, program_offset, target_span),
                        target_selection_range: to_range!(
                            o2p,
                            program_offset,
                            target_selection_span
                        ),
                    }
                },
            )
            .collect()
    }
}

impl ToDocumentResponse for ProgramHoverResponse {
    type DocumentResponse = Hover;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        let Self { contents, span } = self;
        let range = Some(to_range!(o2p, program_offset, span));
        Hover { contents, range }
    }
}

impl ToDocumentResponse for ProgramHighlight {
    type DocumentResponse = DocumentHighlight;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        let ProgramHighlight { kind, span } = self;
        DocumentHighlight {
            kind: Some(kind),
            range: to_range!(o2p, program_offset, span),
        }
    }
}

impl ToDocumentResponse for Vec<ProgramHighlight> {
    type DocumentResponse = Vec<DocumentHighlight>;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        self.into_iter()
            .map(|ProgramHighlight { kind, span }| DocumentHighlight {
                kind: Some(kind),
                range: to_range!(o2p, program_offset, span),
            })
            .collect()
    }
}

impl ToDocumentResponse for ProgramDefinitionResponse {
    type DocumentResponse = GotoDefinitionResponse;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        match self {
            Self::Array(locs) => {
                GotoDefinitionResponse::Array(locs.to_document_response(program_offset, o2p))
            }
            Self::Link(links) => {
                GotoDefinitionResponse::Link(links.to_document_response(program_offset, o2p))
            }
        }
    }
}

impl ToDocumentResponse for Vec<ProgramDiagnostic> {
    type DocumentResponse = Vec<Diagnostic>;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        self.into_iter()
            .map(
                |ProgramDiagnostic {
                     span,
                     severity,
                     code,
                     source,
                     display_message,
                     related_information,
                     ..
                 }| {
                    Diagnostic {
                        range: to_range!(o2p, program_offset, span),
                        severity: Some(severity),
                        code: Some(NumberOrString::String(code)),
                        source: Some(source),
                        message: display_message,
                        related_information: Some(
                            related_information
                                .into_iter()
                                .map(
                                    |ProgramDiagnosticRelatedInformation { location, message }| {
                                        DiagnosticRelatedInformation {
                                            location: location
                                                .to_document_response(program_offset, o2p),
                                            message,
                                        }
                                    },
                                )
                                .collect(),
                        ),
                        tags: None,
                    }
                },
            )
            .collect()
    }
}

impl ToDocumentResponse for ProgramSymbolKind {
    type DocumentResponse = SymbolKind;

    fn to_document_response(
        self,
        _offset: usize,
        _o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        match self {
            Self::Variable => SymbolKind::Variable,
        }
    }
}

impl ToDocumentResponse for Vec<ProgramSymbolInformation> {
    type DocumentResponse = Vec<SymbolInformation>;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        self.into_iter()
            .map(
                |ProgramSymbolInformation {
                     name,
                     kind,
                     location,
                     ..
                 }| {
                    // The `deprecated` field is marked as deprecated.. but also required on the
                    // struct 🤔
                    #[allow(deprecated)]
                    SymbolInformation {
                        name,
                        kind: kind.to_document_response(program_offset, o2p),
                        location: location.to_document_response(program_offset, o2p),
                        deprecated: None,
                        container_name: None,
                    }
                },
            )
            .collect()
    }
}

impl ToDocumentResponse for ProgramTextEdit {
    type DocumentResponse = TextEdit;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        let ProgramTextEdit { span, edit } = self;
        TextEdit {
            range: to_range!(o2p, program_offset, span),
            new_text: edit,
        }
    }
}

impl ToDocumentResponse for ProgramCanRenameResponse {
    type DocumentResponse = PrepareRenameResponse;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        let ProgramCanRenameResponse { span, placeholder } = self;
        PrepareRenameResponse::RangeWithPlaceholder {
            range: to_range!(o2p, program_offset, span),
            placeholder,
        }
    }
}

impl ToDocumentResponse for ProgramCannotRenameBecause {
    type DocumentResponse = tower_lsp::jsonrpc::Error;

    fn to_document_response(
        self,
        _program_offset: usize,
        _o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        use tower_lsp::jsonrpc::{Error, ErrorCode};
        let (code, message) = match self {
            ProgramCannotRenameBecause::CursorNotOverVariable => (
                ServerErrorCode::CursorNotOverVariable,
                "cursor is not over a variable",
            ),
        };
        Error {
            code: ErrorCode::ServerError(code as i64),
            message: message.to_owned(),
            data: None,
        }
    }
}

impl ToDocumentResponse for Result<ProgramCanRenameResponse, ProgramCannotRenameBecause> {
    type DocumentResponse = tower_lsp::jsonrpc::Result<Option<PrepareRenameResponse>>;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        self.map(|v| Some(v.to_document_response(program_offset, o2p)))
            .map_err(|e| e.to_document_response(program_offset, o2p))
    }
}

impl ToDocumentResponse for ProgramRenameResponse {
    type DocumentResponse = WorkspaceEdit;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        let ProgramRenameResponse { uri, edits } = self;
        WorkspaceEdit {
            changes: {
                let mut changes = HashMap::with_capacity(1);
                changes.insert(
                    uri,
                    edits
                        .into_iter()
                        .map(|e| e.to_document_response(program_offset, o2p))
                        .collect(),
                );
                Some(changes)
            },
            ..WorkspaceEdit::default()
        }
    }
}

impl ToDocumentResponse for ProgramFoldingRanges {
    type DocumentResponse = Vec<FoldingRange>;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        self.0
            .into_iter()
            .map(|span| {
                let Range {
                    start:
                        Position {
                            line: start_line,
                            character: start_ch,
                        },
                    end:
                        Position {
                            line: end_line,
                            character: end_ch,
                        },
                } = to_range!(o2p, program_offset, span);
                FoldingRange {
                    start_line,
                    start_character: Some(start_ch),
                    end_line,
                    end_character: Some(end_ch),
                    kind: None,
                }
            })
            .collect()
    }
}

impl ToDocumentResponse for ProgramSelectionRanges {
    type DocumentResponse = SelectionRange;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        self.0
            .into_iter()
            .map(|span| to_range!(o2p, program_offset, span))
            .fold(None, |parent, cur_range| {
                Some(Box::new(SelectionRange {
                    range: cur_range,
                    parent,
                }))
            })
            .map(|boxed| *boxed)
            .expect("Bad state: expected at least one selection range")
    }
}

impl ToDocumentResponse for ProgramActionKind {
    type DocumentResponse = CodeActionKind;

    fn to_document_response(
        self,
        _: usize,
        _: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        match self {
            Self::DiagnosticFix => CodeActionKind::QUICKFIX,
            Self::Rewrite => CodeActionKind::REFACTOR_REWRITE,
        }
    }
}

impl ToDocumentResponse for ProgramAction {
    type DocumentResponse = CodeAction;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        let ProgramAction {
            title,
            kind,
            resolved_diagnostic,
            uri,
            edit,
            is_preferred,
        } = self;
        CodeAction {
            title,
            kind: Some(kind.to_document_response(program_offset, o2p)),
            diagnostics: resolved_diagnostic
                .map(|d| vec![d].to_document_response(program_offset, o2p)),
            edit: {
                let mut changes = HashMap::new();
                changes.insert(uri, vec![edit.to_document_response(program_offset, o2p)]);
                Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..WorkspaceEdit::default()
                })
            },
            command: None,
            is_preferred: Some(is_preferred),
        }
    }
}

impl ToDocumentResponse for Vec<ProgramAction> {
    type DocumentResponse = Vec<CodeAction>;

    fn to_document_response(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::DocumentResponse {
        self.into_iter()
            .map(|pa| pa.to_document_response(program_offset, o2p))
            .collect()
    }
}
