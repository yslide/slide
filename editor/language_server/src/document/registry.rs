//! Module `registry` describes a stateful registry of documents in a server session.

use super::response;
use super::{Document, DocumentParser, DocumentParserMap};

use crate::ptr::{p, P};
use crate::utils::{to_offset, to_position};
use crate::Program;

use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, Url};

/// Describes a change to a [`Document`](Document).
pub enum Change {
    /// The [`Document`](Document) at the `Url` was modified with new content.
    Modified(Url, String),
    /// The [`Document`](Document) at the `Url` was removed.
    Removed(Url),
}

/// A stateful database of [`Document`](Document)s present in a session.
///
/// A `DocumentRegistry` provides mechanisms for applying [change](Change)s to documents present
/// in the registry and acts as a proxy between [server-level and program-level
/// queries](Self::with_program_at). Thus, `DocumentRegistry` is the primary mechanism for
/// interfacing between LSP APIs and [`Program`](crate::Program) APIs.
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
                    let document = parser.parse(src, p(fi.clone()), self.context.dupe());
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
    /// This serves as the backbone of answering server queries, marshalling between [program-level
    /// query responses](crate::program::response) and [document-level query
    /// responses](super::response), the later generally adhering to the LSP API surface.
    ///
    /// The common pattern is to handle an LSP request by calling `DocumentRegistry#with_program_at`
    /// with the `Url` and `Position` the request is for, performing some work with the provided
    /// [`Program`](Program) via a `callback` that returns a program-level response that can be
    /// [converted to](response::ToDocumentResponse) a document-level response, and optionally
    /// converting that to an LSP API response if needed.
    pub fn with_program_at<ProgramResponse: response::ToDocumentResponse>(
        &self,
        uri: &Url,
        position: Position,
        callback: impl FnOnce(&Program, usize) -> Option<ProgramResponse>,
    ) -> Option<ProgramResponse::DocumentResponse> {
        let document = self.document(uri)?;
        let offset_in_document = to_offset(&position, &document.source);
        let program = document.program_at(offset_in_document)?;

        // Marshall to relative position in program.
        let offset_in_program = offset_in_document - program.start;

        // Get the program response.
        let program_response = callback(program, offset_in_program)?;

        // Marshall to absolute position in document and get the document response.
        let to_position = |offset| to_position(offset, document.source.as_ref());
        let document_response = program_response.to_document_response(program.start, &to_position);
        Some(document_response)
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
                    .with_program_at(&fi_md, Position::new(5, 0), |program, offset| {
                        program.get_hover_info(offset)
                    })
                    .unwrap();

                assert_eq!(hover.contents, hover_contents("= 1"));
            }

            {
                // test "b" in "b + c"
                let hover = registry
                    .with_program_at(&fi_md, Position::new(11, 0), |program, offset| {
                        program.get_hover_info(offset)
                    })
                    .unwrap();

                assert_eq!(hover.contents, hover_contents("= 10"));
            }
        }
    }
}