//! Module `registry` describes a stateful registry of [`Document`](Document)s in a server session.

mod document;
mod document_parser;
mod response;
mod source_map;

pub(crate) use document::Document;
pub use document_parser::DocumentParser;
pub use source_map::SourceMap;

use crate::ptr::{p, P};
use crate::Program;

use libslide::Span;
use std::collections::{BTreeMap, HashMap};
use tower_lsp::lsp_types::{Position, Range, Url};

/// Describes a change to a [`Document`](Document).
pub enum Change {
    /// The [`Document`](Document) at the `Url` was modified with new content.
    Modified(Url, String),
    /// The [`Document`](Document) at the `Url` was removed.
    Removed(Url),
}

/// A mapping between file extensions and a [parser](DocumentParser) for that file type.
pub type DocumentParserMap = BTreeMap<String, DocumentParser>;

/// A stateful database of [`Document`](Document)s present in a session.
///
/// A `DocumentRegistry` provides mechanisms for applying [change](Change)s to documents present
/// in the registry and acts as a proxy between [server-level and program-level
/// queries](Self::with_program_at_uri_and_position). Thus, `DocumentRegistry` is the primary
/// mechanism for interfacing between LSP APIs and [`Program`](crate::Program) APIs.
pub(crate) struct DocumentRegistry {
    /// A map of file extensions and a [parser](DocumentParser) for that file type.
    parsers: DocumentParserMap,
    /// The slide [context](libslide::ProgramContext) to use when processing end evaluating slide
    /// programs.
    context: P<libslide::ProgramContext>,
    /// The actual mapping of LSP text documents (represented by a `Url`) to their
    /// [`Document`](Document) representation.
    registry: HashMap<Url, Document>,
}

impl DocumentRegistry {
    /// Creates a new registry with a set of document parsers and slide context.
    pub fn new(parsers: DocumentParserMap, context: P<libslide::ProgramContext>) -> Self {
        Self {
            parsers,
            context,
            registry: Default::default(),
        }
    }

    /// Applies a (document change)[Change] to the registry.
    pub fn apply_change(&mut self, apply_change: Change) {
        match apply_change {
            Change::Removed(fi) => {
                self.registry.remove(&fi);
            }
            Change::Modified(fi, src) => {
                if let Some(parser) = self.get_parser(&fi) {
                    let document = parser.parse(&src, p(fi.clone()), self.context.dupe());
                    self.registry.insert(fi, document);
                }
            }
        }
    }

    /// Retrieves the [`Document`](Document) corresponding to an LSP `Url`, if any.
    pub fn document(&self, uri: &Url) -> Option<&Document> {
        self.registry.get(uri)
    }

    /// Does some work with a program at the specified `uri` and `position`.
    ///
    /// This serves as the backbone of answering server queries, marshaling between [program-level
    /// query responses](crate::program::response) and [document-level query responses](response),
    /// the later generally adhering to the LSP API surface.
    ///
    /// The common pattern is to handle an LSP request by calling `DocumentRegistry#with_program_at`
    /// with the `Url` and `Position` the request is for, performing some work with the provided
    /// [`Program`](Program) via a `callback` that returns a program-level response that can be
    /// [converted to](response::ToDocumentResponse) a document-level response, and optionally
    /// converting that to an LSP API response if needed.
    pub fn with_program_at_uri_and_position<ProgramResponse: response::IntoDocumentResponse>(
        &self,
        uri: &Url,
        position: Position,
        callback: impl FnOnce(&Program, usize) -> Option<ProgramResponse>,
    ) -> Option<ProgramResponse::DocumentResponse> {
        let document = self.document(uri)?;
        let offset_in_document = document.source_map.to_offset(position);
        let program = document.program_at(offset_in_document)?;

        // Marshall to relative position in program.
        let offset_in_program = offset_in_document - program.start;

        // Get the program response.
        let program_response = callback(program, offset_in_program)?;

        // Marshall to absolute position in document and get the document response.
        let to_position = |offset| document.source_map.to_position(offset);
        let document_response =
            program_response.into_document_response(program.start, &to_position);
        Some(document_response)
    }

    /// Like [`with_program_at_uri_and_position`](Self::with_program_at_uri_and_position), but
    /// matches a program at a range and provides the callback a span.
    pub fn with_program_at_uri_and_range<ProgramResponse: response::IntoDocumentResponse>(
        &self,
        uri: &Url,
        range: Range,
        callback: impl FnOnce(&Program, Span) -> Option<ProgramResponse>,
    ) -> Option<ProgramResponse::DocumentResponse> {
        let document = self.document(uri)?;
        let (start_offset_in_document, end_offset_in_document) = {
            let Range { start, end } = range;
            (
                document.source_map.to_offset(start),
                document.source_map.to_offset(end),
            )
        };
        let program =
            document.program_including(start_offset_in_document, end_offset_in_document)?;

        // Marshall to relative position in program.
        let span_in_program = Span::new(
            start_offset_in_document - program.start,
            end_offset_in_document - program.start,
        );

        // Get the program response.
        let program_response = callback(program, span_in_program)?;

        // Marshall to absolute position in document and get the document response.
        let to_position = |offset| document.source_map.to_position(offset);
        let document_response =
            program_response.into_document_response(program.start, &to_position);
        Some(document_response)
    }

    /// Like [`with_program_at_uri_and_position`](Self::with_program_at_uri_and_position), but works
    /// on all [`Program`](Program)s in the document corresponding to the `uri`.
    ///
    /// Returns a `Vec` of responses, each response corresponding to a `Program` in the document.
    /// `Program`s returning an empty response are dropped, and linear order of `Program` response
    /// is not guaranteed.
    pub fn with_programs_at_uri<ProgramResponse: response::IntoDocumentResponse>(
        &self,
        uri: &Url,
        callback: impl Fn(&Program) -> Option<ProgramResponse>,
    ) -> Option<Vec<ProgramResponse::DocumentResponse>> {
        let document = self.document(uri)?;

        // Get the program-level responses and marshal back to document-level responses.
        let to_position = |offset| document.source_map.to_position(offset);
        let document_response = document
            .programs
            .iter()
            .filter_map(|program| callback(program).map(|r| (r, program.start)))
            .map(|(program_response, program_offset)| {
                program_response.into_document_response(program_offset, &to_position)
            })
            .collect();
        Some(document_response)
    }

    /// Like [`with_programs_at_uri`](Self::with_program_at_uri_and_position), but for all documents
    /// in the server session.
    ///
    /// Returns a `Vec` of responses, each response corresponding to a `Program` in the server
    /// session. `Program`s returning an empty response are dropped, and linear order of `Program`
    /// response is not guaranteed.
    pub fn with_all_programs<ProgramResponse: response::IntoDocumentResponse>(
        &self,
        callback: impl Fn(&Program) -> Option<ProgramResponse>,
    ) -> Option<Vec<ProgramResponse::DocumentResponse>> {
        let response = self
            .registry
            .keys()
            .filter_map(|uri| self.with_programs_at_uri(uri, &callback))
            .flatten()
            .collect();
        Some(response)
    }

    /// Retrieves a [`DocumentParser`](DocumentParser) for the given `Url` by its file extension,
    /// if one is known.
    fn get_parser(&self, uri: &Url) -> Option<&DocumentParser> {
        let ext = std::path::Path::new(uri.path())
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default();
        self.parsers.get(ext)
    }
}

#[cfg(test)]
mod document_registry_tests {
    use super::*;
    use crate::ptr::p;
    use tower_lsp::lsp_types::Url;

    fn mk_parsers(parsers: &[(&str, &str)]) -> DocumentParserMap {
        parsers
            .iter()
            .map(|(name, parser)| (name.to_string(), DocumentParser::build(parser).unwrap()))
            .collect()
    }

    fn url(url: &str) -> Url {
        Url::parse(url).unwrap()
    }

    #[allow(non_snake_case)]
    fn SM_registry() -> DocumentRegistry {
        DocumentRegistry::new(
            mk_parsers(&[("slide", "(.*)"), ("math", "(.+)")]),
            p(Default::default()),
        )
    }

    #[test]
    fn get_parser() {
        let registy = SM_registry();
        for (fi, parser) in &[
            ("file:///fi.slide", Some("(.*)")),
            ("file:///fi.math", Some("(.+)")),
            ("file:///slide", None),
            ("file:///fi.txt", None),
        ] {
            assert_eq!(
                registy.get_parser(&url(fi)).map(ToString::to_string),
                parser.map(|s| s.to_owned())
            );
        }
    }

    mod changes {
        use super::super::{Change, DocumentRegistry};
        use super::{url, SM_registry};
        use tower_lsp::lsp_types::Url;

        fn first_program<'a>(registry: &'a DocumentRegistry, fi: &Url) -> &'a str {
            registry
                .registry
                .get(fi)
                .unwrap_or_else(|| panic!("No document for {}", fi))
                .programs[0]
                .source
                .as_ref()
        }

        #[test]
        fn multiple_edits() {
            let mut registry = SM_registry();
            let fi_slide = url("file:///fi.slide");
            let fi_math = url("file:///fi.math");

            // Add fi_slide
            registry.apply_change(Change::Modified(fi_slide.clone(), "1 + 2".into()));
            assert_eq!(first_program(&registry, &fi_slide), "1 + 2");

            // Add fi_math: now fi_slide and fi_math should be registered
            registry.apply_change(Change::Modified(fi_math.clone(), "3 + 4".into()));
            assert_eq!(registry.registry.len(), 2);
            assert_eq!(first_program(&registry, &fi_math), "3 + 4");

            // Change fi_slide: both should still be registered
            registry.apply_change(Change::Modified(fi_slide.clone(), "1 + 10".into()));
            assert_eq!(registry.registry.len(), 2);
            assert_eq!(first_program(&registry, &fi_slide), "1 + 10");
        }

        #[test]
        fn multiple_edits_with_deletion() {
            let mut registry = SM_registry();
            let fi_slide = url("file:///fi.slide");
            let fi_math = url("file:///fi.math");

            // Add fi_slide
            registry.apply_change(Change::Modified(fi_slide.clone(), "1 + 2".into()));
            assert_eq!(first_program(&registry, &fi_slide), "1 + 2");

            // Add fi_math: now fi_slide and fi_math should be registered
            registry.apply_change(Change::Modified(fi_math.clone(), "3 + 4".into()));
            assert_eq!(registry.registry.len(), 2);
            assert_eq!(first_program(&registry, &fi_math), "3 + 4");

            // Delete fi_slide: fi_math should still be registered
            registry.apply_change(Change::Removed(fi_slide.clone()));
            assert_eq!(registry.registry.len(), 1);
            assert_eq!(first_program(&registry, &fi_math), "3 + 4");

            // Add fi_slide: both should be registered
            registry.apply_change(Change::Modified(fi_slide.clone(), "1 + 10".into()));
            assert_eq!(registry.registry.len(), 2);
            assert_eq!(first_program(&registry, &fi_slide), "1 + 10");
            assert_eq!(first_program(&registry, &fi_math), "3 + 4");
        }
    }

    mod with_program_at {
        use super::*;

        #[test]
        fn absolute_position_conversion() {
            use tower_lsp::lsp_types::*;

            let mut registry = DocumentRegistry::new(
                mk_parsers(&[("md", r"```math\n((?:.|\n)*?)\n```")]),
                p(Default::default()),
            );
            let fi_md = url("file:///test.md");
            let fi_content = r" // 0
# Hi!                           // 1
                                // 2
```math
   a = 1
a + b
```                             // 6
                                // 7
## Othello                      // 8
                                // 9
```math
b + c
   b = 10
```                             // 13";

            registry.apply_change(Change::Modified(fi_md.clone(), fi_content.to_string()));

            fn hover_contents(value: &str) -> HoverContents {
                HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                    language: "math".to_owned(),
                    value: value.to_owned(),
                }))
            }

            {
                // test "a" in "a + b"
                let hover = registry
                    .with_program_at_uri_and_position(
                        &fi_md,
                        Position::new(5, 0),
                        |program, offset| program.get_hover_info(offset),
                    )
                    .unwrap();

                assert_eq!(hover.contents, hover_contents("= 1"));
            }

            {
                // test "b" in "b + c"
                let hover = registry
                    .with_program_at_uri_and_position(
                        &fi_md,
                        Position::new(11, 0),
                        |program, offset| program.get_hover_info(offset),
                    )
                    .unwrap();

                assert_eq!(hover.contents, hover_contents("= 10"));
            }
        }
    }
}
