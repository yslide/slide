//! Module `document` describes the model by which text files are handled in a server session.
//!
//! Every document in a server session may have multiple [`Program`](crate::Program)s, which serves
//! the heart of all query work made by the server. A document is a conduit between a text file and
//! the programs within it.
//!
//! Programs are extracted from a document using a [`DocumentParser`](self::DocumentParser).

use crate::ptr::{p, P};
use crate::shims::to_offset;
use crate::Program;

use std::collections::{BTreeMap, HashMap};
use tower_lsp::lsp_types::{Diagnostic, Position, Url};

mod document_parser;

pub(crate) use document_parser::DocumentParser;

pub(crate) type DocumentParserMap = BTreeMap<String, DocumentParser>;

pub(crate) struct Document {
    _uri: P<Url>,
    // TODO: give programs a source mapping of lsp positions to offsets or something so we
    // don't have to keep the entire document text.
    source: String,
    pub programs: Vec<Program>,
}

impl Document {
    fn new(uri: P<Url>, source: String, programs: Vec<Program>) -> Self {
        Self {
            _uri: uri,
            source,
            programs,
        }
    }

    /// Retrieves diagnostics cross all [Program](crate::Program)s in this document.
    pub fn all_diagnostics(&self) -> Vec<Diagnostic> {
        self.programs
            .iter()
            .map(|p| p.diagnostics().clone())
            .flatten()
            .collect()
    }

    fn program_at(&self, _offset: usize) -> Option<&Program> {
        // TODO: algorithm for getting actual program
        self.programs.first()
    }
}

pub(crate) enum ChangeKind {
    FileModified(Url, String),
    FileRemoved(Url),
}

pub(crate) struct DocumentRegistry {
    #[allow(unused)]
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
    pub fn with_program_at<T>(
        &self,
        uri: &Url,
        position: Position,
        cb: impl FnOnce(&Program, usize) -> Option<T>,
    ) -> Option<T> {
        let document = self.document(uri)?;
        let absolute_offset = to_offset(&position, &document.source);
        let program = document.program_at(absolute_offset)?;
        let offset_in_program = absolute_offset - program.start;

        // TODO: marshall back to absolute location
        cb(program, offset_in_program)
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
    use super::{DocumentParser, DocumentParserMap, DocumentRegistry};
    use crate::ptr::p;
    use tower_lsp::lsp_types::Url;

    fn mk_parsers(parsers: Vec<(&str, &str)>) -> DocumentParserMap {
        parsers
            .into_iter()
            .map(|(name, parser)| (name.to_owned(), DocumentParser::build(parser).unwrap()))
            .collect()
    }

    fn url(url: &str) -> Url {
        Url::parse(url).unwrap()
    }

    #[allow(non_snake_case)]
    fn SM_registry() -> DocumentRegistry {
        DocumentRegistry::new(
            mk_parsers(vec![("slide", "(.*)"), ("math", "(.+)")]),
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
}
