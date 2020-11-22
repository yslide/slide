//! Module `document` describes the model by which text files are handled in a server session.
//!
//! Every document in a server session may have multiple [`Program`](crate::Program)s, which serves
//! the heart of all query work made by the server. A document is a conduit between a text file and
//! the programs within it, and serves as the main abstraction for the language server.
//!
//! Programs are extracted from a document using a [`DocumentParser`](super::DocumentParser).

use super::response::ToDocumentResponse;
use super::source_map::SourceMap;
use crate::Program;

use tower_lsp::lsp_types::Diagnostic;

/// A `Document` describes a text file known to a server session, and contains information about
/// slide [`Program`](crate::Program)s in the file. One `Document` may have multiple `Programs`,
/// which are discovered by [`DocumentParser`](super::DocumentParser)s.
///
/// `Document`s are owned by a [`DocumentRegistry`](super::DocumentRegistry), where they serve as a
/// conduit between requests at the level of the LSP server and the program-local queries. See
/// [`DocumentRegistry`](super::DocumentRegistry) and the [`response` module](super::response) for
/// more details.
pub(crate) struct Document {
    /// The [`SourceMap`](SourceMap) for the text of the document.
    pub source_map: SourceMap,
    /// List of [`Program`](crate::Program)s in this document.
    ///
    /// This list must observe the invariant of `programs_{i}.end <= programs_{i+1}.start`.
    pub programs: Vec<Program>,
}

impl Document {
    /// Creates a new document with the document source text and [Program](crate::Program)s parsed
    /// out of the document.
    pub fn new(source: &str, programs: Vec<Program>) -> Self {
        Self {
            source_map: SourceMap::new(source),
            programs,
        }
    }

    /// Retrieves diagnostics across all [Program](crate::Program)s present in this document.
    pub fn all_diagnostics(&self) -> Vec<Diagnostic> {
        let to_position = |offset| self.source_map.to_position(offset);
        self.programs
            .iter()
            .map(|p| {
                p.diagnostics()
                    .clone()
                    .to_document_response(p.start, &to_position)
            })
            .flatten()
            .collect()
    }

    /// Retrieves the [Program](crate::Program) present at the document offset position, if any.
    pub fn program_at(&self, offset: usize) -> Option<&Program> {
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
    use super::{super::DocumentParser, Document};
    use crate::ptr::p;
    use tower_lsp::lsp_types::Url;

    fn math_document(content: &str) -> Document {
        DocumentParser::build(r"```math\n((?:.|\n)*?)\n```")
            .unwrap()
            .parse(
                content,
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
