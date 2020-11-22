use crate::ToAbsoluteResponse;

use libslide::Span;
use tower_lsp::lsp_types::*;

macro_rules! to_range {
    ($document_offset_to_position:ident, $program_offset:ident, $span:ident) => {
        Range::new(
            $document_offset_to_position($span.lo + $program_offset),
            $document_offset_to_position($span.hi + $program_offset),
        )
    };
}

#[derive(Debug, Clone)]
pub struct LocalLocation {
    pub uri: Url,
    pub span: Span,
}

impl ToAbsoluteResponse for LocalLocation {
    type Response = Location;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        let LocalLocation { uri, span } = self;
        Location::new(uri, to_range!(o2p, program_offset, span))
    }
}

impl ToAbsoluteResponse for Vec<LocalLocation> {
    type Response = Vec<Location>;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        self.into_iter()
            .map(|LocalLocation { uri, span }| {
                Location::new(uri, to_range!(o2p, program_offset, span))
            })
            .collect()
    }
}

pub struct LocalLocationLink {
    pub origin_selection_span: Span,
    pub target_uri: Url,
    pub target_span: Span,
    pub target_selection_span: Span,
}

impl ToAbsoluteResponse for Vec<LocalLocationLink> {
    type Response = Vec<LocationLink>;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        self.into_iter()
            .map(
                |LocalLocationLink {
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

pub struct LocalHoverResponse {
    pub contents: HoverContents,
    pub span: Span,
}

impl crate::ToAbsoluteResponse for LocalHoverResponse {
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

pub struct LocalHighlight {
    pub kind: DocumentHighlightKind,
    pub span: libslide::Span,
}

impl ToAbsoluteResponse for LocalHighlight {
    type Response = DocumentHighlight;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        let LocalHighlight { kind, span } = self;
        DocumentHighlight {
            kind: Some(kind),
            range: to_range!(o2p, program_offset, span),
        }
    }
}

impl ToAbsoluteResponse for Vec<LocalHighlight> {
    type Response = Vec<DocumentHighlight>;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        self.into_iter()
            .map(|LocalHighlight { kind, span }| DocumentHighlight {
                kind: Some(kind),
                range: to_range!(o2p, program_offset, span),
            })
            .collect()
    }
}

pub enum LocalDefinitionResponse {
    Array(Vec<LocalLocation>),
    Link(Vec<LocalLocationLink>),
}

impl ToAbsoluteResponse for LocalDefinitionResponse {
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

#[derive(Debug, Clone)]
pub struct LocalDiagnosticRelatedInformation {
    pub location: LocalLocation,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct LocalDiagnostic {
    pub span: Span,
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub source: String,
    pub message: String,
    pub related_information: Vec<LocalDiagnosticRelatedInformation>,
}

impl ToAbsoluteResponse for Vec<LocalDiagnostic> {
    type Response = Vec<Diagnostic>;

    fn to_absolute(
        self,
        program_offset: usize,
        o2p: &impl Fn(usize) -> Position,
    ) -> Self::Response {
        self.into_iter()
            .map(
                |LocalDiagnostic {
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
                                .map(|LocalDiagnosticRelatedInformation { location, message }| {
                                    DiagnosticRelatedInformation {
                                        location: location.to_absolute(program_offset, o2p),
                                        message,
                                    }
                                })
                                .collect(),
                        ),
                        tags: None,
                    }
                },
            )
            .collect()
    }
}
