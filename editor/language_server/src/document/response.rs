//! Module `response` describes the API for responses of language queries at the level of a
//! document, and generally correspond to the API surface of the LSP. In general, document-level
//! responses are converted from [program-level responses](crate::program::response) by the
//! implementation of the [`ToDocumentResponse`](ToDocumentResponse) trait. All such
//! implementations should reside in this module.

use crate::program::response::*;
use tower_lsp::lsp_types::*;

pub trait ToDocumentResponse {
    type Response;

    fn to_absolute(
        self,
        program_offset: usize,
        document_offset_to_position: &impl Fn(usize) -> Position,
    ) -> Self::Response;
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
    type Response = Location;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        let ProgramLocation { uri, span } = self;
        Location::new(uri, to_range!(o2p, program_offset, span))
    }
}

impl ToDocumentResponse for Vec<ProgramLocation> {
    type Response = Vec<Location>;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        self.into_iter()
            .map(|ProgramLocation { uri, span }| {
                Location::new(uri, to_range!(o2p, program_offset, span))
            })
            .collect()
    }
}

impl ToDocumentResponse for Vec<ProgramLocationLink> {
    type Response = Vec<LocationLink>;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
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
    type Response = Hover;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        let Self { contents, span } = self;
        let range = Some(to_range!(o2p, program_offset, span));
        Hover { contents, range }
    }
}

impl ToDocumentResponse for ProgramHighlight {
    type Response = DocumentHighlight;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        let ProgramHighlight { kind, span } = self;
        DocumentHighlight {
            kind: Some(kind),
            range: to_range!(o2p, program_offset, span),
        }
    }
}

impl ToDocumentResponse for Vec<ProgramHighlight> {
    type Response = Vec<DocumentHighlight>;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        self.into_iter()
            .map(|ProgramHighlight { kind, span }| DocumentHighlight {
                kind: Some(kind),
                range: to_range!(o2p, program_offset, span),
            })
            .collect()
    }
}

impl ToDocumentResponse for ProgramDefinitionResponse {
    type Response = GotoDefinitionResponse;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        match self {
            Self::Array(locs) => {
                GotoDefinitionResponse::Array(locs.to_absolute(program_offset, o2p))
            }
            Self::Link(links) => {
                GotoDefinitionResponse::Link(links.to_absolute(program_offset, o2p))
            }
        }
    }
}

impl ToDocumentResponse for Vec<ProgramDiagnostic> {
    type Response = Vec<Diagnostic>;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
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
                                            location: location.to_absolute(program_offset, o2p),
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
