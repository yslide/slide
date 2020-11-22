//! Module `document` describes the model by which text files are handled in a server session.
//!
//! Every document in a server session may have multiple [`Program`](crate::Program)s, which serves
//! the heart of all query work made by the server. A document is a conduit between a text file and
//! the programs within it.
//!
//! Programs are extracted from a document using a [`DocumentParser`](self::DocumentParser).

use crate::ptr::P;
use crate::utils::to_position;
use crate::Program;

use std::collections::BTreeMap;
use tower_lsp::lsp_types::{Diagnostic, Url};

mod parser;
mod registry;
mod response;

pub(crate) use parser::DocumentParser;
pub(crate) use registry::ChangeKind;
pub(crate) use registry::DocumentRegistry;
pub(crate) use response::ToDocumentResponse;

pub type DocumentParserMap = BTreeMap<String, DocumentParser>;

pub(crate) struct Document {
    _uri: P<Url>,
    // TODO: give programs a source mapping of lsp positions to offsets or something to make
    // lookups of offsets in a source faster.
    source: String,
    /// List of [`Program`](crate::Program)s in this document.
    ///
    /// This list must observe the invariant of `programs_{i}.end <= programs_{i+1}.start`.
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
        let to_position = |offset| to_position(offset, self.source.as_ref());
        self.programs
            .iter()
            .map(|p| p.diagnostics().clone().to_absolute(p.start, &to_position))
            .flatten()
            .collect()
    }

    fn program_at(&self, offset: usize) -> Option<&Program> {
        let idx = self
            .programs
            .binary_search_by(|program| {
                if offset < program.start {
                    std::cmp::Ordering::Greater
                } else if offset >= program.end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .ok()?;
        Some(&self.programs[idx])
    }
}

#[cfg(test)]
mod document_tests {
    use super::{Document, DocumentParser};
    use crate::ptr::p;
    use tower_lsp::lsp_types::Url;

    fn math_document(content: &str) -> Document {
        DocumentParser::build(r"```math\n((?:.|\n)*?)\n```")
            .unwrap()
            .parse(
                content.to_owned(),
                p(Url::parse("file:///math.md").unwrap()),
                p(Default::default()),
            )
    }

    #[test]
    fn all_diagnostics() {
        let document = math_document(
            r"
# Hello 

```math
1 + 
```

## Othello

```math
3 +
```",
        );

        assert_eq!(document.all_diagnostics().len(), 2);
    }

    #[test]
    fn get_program() {
        let content = r"
# Hello 

```math
1 + 1
```

## Othello

```math
3 + 3
```";
        let document = math_document(content);

        let start_p1 = content.find("1 + 1").unwrap();
        let start_p2 = content.find("3 + 3").unwrap();

        assert_eq!(document.program_at(start_p1).unwrap().source, "1 + 1");
        assert_eq!(document.program_at(start_p2).unwrap().source, "3 + 3");
    }
}
