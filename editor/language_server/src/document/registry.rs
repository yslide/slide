//! Module `registry` describes a stateful registry of documents in a server session.

use super::response;
use super::{Document, DocumentParser, DocumentParserMap};

use crate::ptr::{p, P};
use crate::utils::{to_offset, to_position};
use crate::Program;

use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, Url};

pub enum ChangeKind {
    FileModified(Url, String),
    FileRemoved(Url),
}

pub(crate) struct DocumentRegistry {
    parsers: DocumentParserMap,
    context: P<libslide::ProgramContext>,
    registry: HashMap<Url, Document>,
}

impl DocumentRegistry {
    pub fn new(parsers: DocumentParserMap, context: P<libslide::ProgramContext>) -> Self {
        Self {
            parsers,
            context,
            registry: Default::default(),
        }
    }

    pub fn apply_change(&mut self, apply_change: ChangeKind) {
        match apply_change {
            ChangeKind::FileRemoved(fi) => {
                self.registry.remove(&fi);
            }
            ChangeKind::FileModified(fi, src) => self.file_modified(fi, src),
        }
    }

    pub fn document(&self, uri: &Url) -> Option<&Document> {
        self.registry.get(uri)
    }

    /// Does some work with a program at the specified `uri` and `position`.
    pub fn with_program_at<RelativeResponse: response::ToDocumentResponse>(
        &self,
        uri: &Url,
        position: Position,
        cb: impl FnOnce(&Program, usize) -> Option<RelativeResponse>,
    ) -> Option<RelativeResponse::Response> {
        let document = self.document(uri)?;
        let absolute_offset = to_offset(&position, &document.source);
        let program = document.program_at(absolute_offset)?;

        // Marshall to relative position in program
        let offset_in_program = absolute_offset - program.start;

        // Get the response
        let partial_response = cb(program, offset_in_program);

        // Marshall to absolute position in document
        let to_position = |offset| to_position(offset, document.source.as_ref());
        partial_response.map(|pr| pr.to_absolute(program.start, &to_position))
    }

    fn file_modified(&mut self, uri: Url, source: String) {
        if let Some(parser) = self.get_parser(&uri) {
            let document = parser.parse(source, p(uri.clone()), self.context.dupe());
            self.registry.insert(uri, document);
        }
    }

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
            .into_iter()
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
        use super::super::{ChangeKind, DocumentRegistry};
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
            registry.apply_change(ChangeKind::FileModified(fi_slide.clone(), "1 + 2".into()));
            assert_eq!(first_program(&registry, &fi_slide), "1 + 2");

            // Add fi_math: now fi_slide and fi_math should be registered
            registry.apply_change(ChangeKind::FileModified(fi_math.clone(), "3 + 4".into()));
            assert_eq!(registry.registry.len(), 2);
            assert_eq!(first_program(&registry, &fi_math), "3 + 4");

            // Change fi_slide: both should still be registered
            registry.apply_change(ChangeKind::FileModified(fi_slide.clone(), "1 + 10".into()));
            assert_eq!(registry.registry.len(), 2);
            assert_eq!(first_program(&registry, &fi_slide), "1 + 10");
        }

        #[test]
        fn multiple_edits_with_deletion() {
            let mut registry = SM_registry();
            let fi_slide = url("file:///fi.slide");
            let fi_math = url("file:///fi.math");

            // Add fi_slide
            registry.apply_change(ChangeKind::FileModified(fi_slide.clone(), "1 + 2".into()));
            assert_eq!(first_program(&registry, &fi_slide), "1 + 2");

            // Add fi_math: now fi_slide and fi_math should be registered
            registry.apply_change(ChangeKind::FileModified(fi_math.clone(), "3 + 4".into()));
            assert_eq!(registry.registry.len(), 2);
            assert_eq!(first_program(&registry, &fi_math), "3 + 4");

            // Delete fi_slide: fi_math should still be registered
            registry.apply_change(ChangeKind::FileRemoved(fi_slide.clone()));
            assert_eq!(registry.registry.len(), 1);
            assert_eq!(first_program(&registry, &fi_math), "3 + 4");

            // Add fi_slide: both should be registered
            registry.apply_change(ChangeKind::FileModified(fi_slide.clone(), "1 + 10".into()));
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

            registry.apply_change(ChangeKind::FileModified(
                fi_md.clone(),
                fi_content.to_string(),
            ));

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
