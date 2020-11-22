//! Module `init` describes initialization options of the slide language server.

use crate::document_registry::{DocumentParser, DocumentParserMap};

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// Initialization options available to the server.
#[derive(Default, Debug, PartialEq)]
pub struct InitializationOptions {
    /// Document file extension -> [`DocumentParser`](DocumentParser) for that document.
    pub document_parsers: DocumentParserMap,
}

/// A diagnostic produced while constructing [`InitializationOptions`](InitializationOptions).
#[derive(Debug, PartialEq)]
pub enum InitializationDiagnostic {
    /// Entirely failed to parse the provided options from a JSON value. This likely means the
    /// options JSON is malformed.
    CouldntParse(String),
    /// Found no `document_parsers` key in the options JSON value, or the value is not an object.
    NoDocumentParserMap,
    /// Could not construct a [`DocumentParser`](DocumentParser) for a given file extension in the
    /// `document_parsers` object.
    InvalidDocumentParser(/** extension */ String, /** why */ String),
}

/// The raw values parsed from an initialization options JSON value, used to construct
/// [`InitializationOptions`](InitializationOptions).
#[derive(Deserialize)]
struct SerializedInitializationOptions {
    document_parsers: Option<BTreeMap<String, String>>,
}

impl InitializationOptions {
    /// Creates a fresh [`InitializationOptions`](InitializationOptions) from a JSON value, also
    /// returning any [diagnostics](InitializationDiagnostic) discovered during the options'
    /// construction.
    pub fn from_json(json: Option<Value>) -> (Self, Vec<InitializationDiagnostic>) {
        let opts: SerializedInitializationOptions =
            match serde_json::from_value(json.unwrap_or_default()) {
                Ok(opts) => opts,
                Err(e) => {
                    return (
                        Default::default(),
                        vec![InitializationDiagnostic::CouldntParse(e.to_string())],
                    );
                }
            };

        let SerializedInitializationOptions { document_parsers } = opts;
        let mut diags = vec![];
        let document_parsers = {
            match document_parsers.as_ref() {
                None => diags.push(InitializationDiagnostic::NoDocumentParserMap),
                Some(d) if d.is_empty() => {
                    diags.push(InitializationDiagnostic::NoDocumentParserMap)
                }
                _ => {}
            };
            document_parsers
                .unwrap_or_default()
                .into_iter()
                .filter_map(|(name, parser)| match DocumentParser::build(&parser) {
                    Ok(parser) => Some((name, parser)),
                    Err(why) => {
                        diags.push(InitializationDiagnostic::InvalidDocumentParser(
                            name,
                            why.to_string(),
                        ));
                        None
                    }
                })
                .collect()
        };

        let opts = Self { document_parsers };
        (opts, diags)
    }
}

impl std::fmt::Display for InitializationDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
                Self::CouldntParse(why) => format!("Failed to parse language server options:\n{}", why),
                Self::NoDocumentParserMap => "No `document_parsers` in server options; slide LS will be a no-op for all documents".to_owned(),
                Self::InvalidDocumentParser(doc, why) => format!("Failed to build parser regex for `{}`:\n{}", doc, why),
            }.fmt(f)
    }
}

#[cfg(test)]
mod test {
    use super::{DocumentParser, InitializationDiagnostic, InitializationOptions};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    fn mk_options(document_parsers: Vec<(&str, &str)>) -> InitializationOptions {
        InitializationOptions {
            document_parsers: document_parsers
                .into_iter()
                .map(|(fi, re)| (fi.to_owned(), DocumentParser::build(re).unwrap()))
                .collect(),
        }
    }

    #[test]
    fn no_options() {
        let (opts, diags) = InitializationOptions::from_json(None);

        assert_eq!(opts, InitializationOptions::default());
        assert_eq!(
            diags,
            vec![InitializationDiagnostic::CouldntParse(
                "invalid type: null, expected struct SerializedInitializationOptions".to_owned()
            )]
        );
    }

    #[test]
    fn missing_options() {
        let (opts, diags) = InitializationOptions::from_json(Some(json!({
            "abcd": 1,
        })));

        assert_eq!(opts, InitializationOptions::default());
        assert_eq!(diags, vec![InitializationDiagnostic::NoDocumentParserMap]);
    }

    #[test]
    fn unparsable_document_parser() {
        let (opts, diags) = InitializationOptions::from_json(Some(json!({
            "document_parsers": {
                "slide": "["
            },
        })));

        assert_eq!(opts, InitializationOptions::default());
        assert_eq!(
            diags,
            vec![InitializationDiagnostic::InvalidDocumentParser(
                "slide".to_owned(),
                r"regex parse error:
    [
    ^
error: unclosed character class"
                    .to_owned()
            )]
        );
    }

    #[test]
    fn document_parser_invalid_number_capturing_groups() {
        let (opts, diags) = InitializationOptions::from_json(Some(json!({
            "document_parsers": {
                "one": "(.*)(.*)",
                "two": ".*",
            },
        })));

        assert_eq!(opts, InitializationOptions::default());
        assert_eq!(
            diags,
            vec![
                InitializationDiagnostic::InvalidDocumentParser(
                    "one".to_owned(),
                    "must have exactly one explicit capturing group for a slide program; found 2"
                        .to_owned(),
                ),
                InitializationDiagnostic::InvalidDocumentParser(
                    "two".to_owned(),
                    "must have exactly one explicit capturing group for a slide program; found 0"
                        .to_owned(),
                ),
            ]
        );
    }

    #[test]
    fn valid_document_parsers() {
        let (opts, diags) = InitializationOptions::from_json(Some(json!({
            "document_parsers": {
                "math": "(.*)",
                "md": r"```math\n((?:.|\n)*?)\n```",
            },
        })));

        assert_eq!(
            opts,
            mk_options(vec![
                ("math", "(.*)"),
                ("md", r"```math\n((?:.|\n)*?)\n```"),
            ])
        );
        assert!(diags.is_empty());
    }
}
