//! Module `response` describes the API for responses of language queries at the level of a
//! document, and generally correspond to the API surface of the LSP. In general, document-level
//! responses are converted from [program-level responses](crate::program::response) by the
//! implementation of the [`ToDocumentResponse`](ToDocumentResponse) trait. All such
//! implementations should reside in this module.

use crate::program::response::*;
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
                     message,
                     related_information,
                 }| {
                    Diagnostic {
                        range: to_range!(o2p, program_offset, span),
                        severity: Some(severity),
                        code: Some(NumberOrString::String(code)),
                        source: Some(source),
                        message,
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
