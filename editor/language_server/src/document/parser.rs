//! Module `document_parser` describes how slide [Program](crate::Program)s should be parsed from a
//! document.

use super::Document;
use crate::ptr::P;
use crate::Program;

use libslide::ProgramContext;
use regex::RegexBuilder;
use tower_lsp::lsp_types::Url;

/// Responsible for parsing some kind of document into segements of slide programs.
/// The client is responsible for determining which documents a `DocumentParser` applies to.
#[derive(Debug)]
pub struct DocumentParser(regex::Regex);

impl DocumentParser {
    /// Creates a new document parser from a multi-line regex description of what slide program
    /// blocks look like in the document. If parsing the regex fails or does not meet the
    /// undermentioned regex requirements, an error is returned.
    ///
    /// The provided regex string will be parsed as a regex subject to the following constraints:
    /// - Must observe PCRE regex syntax
    /// - Must contain exactly one explicit capturing group to denote the contents of a slide
    ///   program. For example, `(.*)` and `` ```math\n((?:.|\n)*?)\n``` `` meet this requirement,
    ///   while `.*`, `(.*)(.*)`, and `` ```math\n((.|\n)*?)\n``` `` do not.
    /// - Will be parsed as a multi-line regex; be sure to include newlines explicitly if you want
    ///   them to be captured by the regex. For example, `(.*)` captures all characters except line
    ///   feeds; to also capture line feeds, use `((?:.|\n)*)`.
    pub fn build(parser: &str) -> Result<DocumentParser, regex::Error> {
        let mut re = RegexBuilder::new(parser);
        let re = re.multi_line(true);
        let re = re.build()?;

        // Validation
        if re.captures_len() != 2 {
            // First capturing group is the entire match.
            // Second capturing group should be the slide program.
            return Err(regex::Error::Syntax(format!(
                "must have exactly one explicit capturing group for a slide program; found {}",
                re.captures_len() - 1
            )));
        }

        Ok(Self(re))
    }

    /// Parses a document's source text with this document parser, returning a fresh
    /// [`Document`](Document) with all discovered [`Program`](Program)s.
    pub(crate) fn parse(
        &self,
        document_source: &str,
        document_uri: P<Url>,
        context: P<ProgramContext>,
    ) -> Document {
        let programs = self
            .0
            .captures_iter(&document_source)
            .map(|segment| {
                let program = segment
                    .get(1)
                    .expect("Inconsistent state: parser missing first capturing group");
                Program::new(
                    program.as_str().to_owned(),
                    document_uri.dupe(),
                    program.start(),
                    program.end(),
                    context.dupe(),
                )
            })
            .collect();

        Document::new(&document_source, programs)
    }
}

impl std::cmp::PartialEq for DocumentParser {
    /// Two document parsers are equal iff their regex representations are equivalent.
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str().eq(other.0.as_str())
    }
}

impl std::fmt::Display for DocumentParser {
    /// Formats the parser as its regex representation.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_str().fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::DocumentParser;

    mod build {
        use super::DocumentParser;

        #[test]
        fn ok() {
            assert!(DocumentParser::build("(.*)").is_ok());
        }

        #[test]
        fn invalid_regex() {
            assert_eq!(
                DocumentParser::build("[").unwrap_err(),
                regex::Error::Syntax(
                    r"regex parse error:
    [
    ^
error: unclosed character class"
                        .to_owned()
                )
            );
        }

        #[test]
        fn no_capturing_groups() {
            assert_eq!(
                DocumentParser::build(".*").unwrap_err(),
                regex::Error::Syntax(
                    "must have exactly one explicit capturing group for a slide program; found 0"
                        .to_owned()
                )
            );
        }

        #[test]
        fn multiple_capturing_groups() {
            assert_eq!(
                DocumentParser::build("(.*)(.*)").unwrap_err(),
                regex::Error::Syntax(
                    "must have exactly one explicit capturing group for a slide program; found 2"
                        .to_owned()
                )
            );
        }
    }

    mod parse {
        use super::DocumentParser;
        use crate::ptr::p;
        use pretty_assertions::assert_eq;
        use tower_lsp::lsp_types::Url;

        #[test]
        fn parse_document() {
            let document_content = r"
Hi, this is my document. Here is one math block:

```math
1 + 2 + 3
```

And here is another:

```math
e = a + b / c ^ d
f = 9 * 8
```
";
            let uri = p(Url::parse("file:///test").unwrap());
            let context = p(libslide::ProgramContext::default());

            let parser = DocumentParser::build(r"```math\n((?:.|\n)*?)\n```").unwrap();
            let document = parser.parse(document_content, uri.dupe(), context.dupe());

            assert_eq!(document.programs.len(), 2);
            let p1 = &document.programs[0];
            assert_eq!(p1.document_uri, uri);
            assert_eq!(p1.context, context);
            assert_eq!(p1.source, "1 + 2 + 3");
            assert_eq!(&document_content[p1.start..p1.end], "1 + 2 + 3");

            let p2 = &document.programs[1];
            assert_eq!(p2.document_uri, uri);
            assert_eq!(p2.context, context);
            assert_eq!(p2.source, "e = a + b / c ^ d\nf = 9 * 8");
            assert_eq!(
                &document_content[p2.start..p2.end],
                "e = a + b / c ^ d\nf = 9 * 8"
            );
        }
    }
}
