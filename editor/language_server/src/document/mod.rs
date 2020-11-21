//! Module `document` describes the document model used by the slide LS.

use crate::ast::AST;
use crate::shims::convert_diagnostics;

use std::collections::{BTreeMap, HashMap};
use tower_lsp::lsp_types::{Diagnostic, Url};

mod document_parser;
pub(crate) use document_parser::DocumentParser;

pub(crate) type DocumentParserMap = BTreeMap<String, DocumentParser>;

pub(crate) struct ProgramInfo {
    pub source: String,
    pub uri: Url,
    pub original: AST,
    pub simplified: AST,
}

pub enum ChangeKind {
    FileModified(Url, String),
    FileRemoved(Url),
}

pub(crate) struct DocumentRegistry {
    #[allow(unused)]
    parsers: DocumentParserMap,
    context: libslide::ProgramContext,
    registry: HashMap<Url, ProgramInfo>,
}

impl DocumentRegistry {
    pub fn new(parsers: DocumentParserMap, context: libslide::ProgramContext) -> Self {
        Self {
            parsers,
            context,
            registry: Default::default(),
        }
    }

    // TODO: the server should ask for diagnostics itself
    pub fn change(&mut self, change: ChangeKind) -> Vec<Diagnostic> {
        match change {
            ChangeKind::FileRemoved(fi) => {
                self.registry.remove(&fi);
                vec![]
            }
            ChangeKind::FileModified(fi, src) => self.file_modified(fi, src),
        }
    }

    pub fn program(&self, uri: &Url) -> Option<&ProgramInfo> {
        self.registry.get(uri)
    }

    pub fn context(&self) -> &libslide::ProgramContext {
        &self.context
    }

    fn file_modified(&mut self, uri: Url, source: String) -> Vec<Diagnostic> {
        use libslide::*;

        // On document change, we do the following:
        //   1. Reparse the program source code
        //   2. Evaluate the program
        //      - There is a tradeoff between evaluating everything at once on change and lazily
        //        evaluating on queries. For now, we need to do it in this step because some
        //        diagnostics (i.e. validation) cannot be done without performing evaluation
        //        anyway.
        //        A future flow could be to use a "query" model, whereby we incrementally parse,
        //        evaluate, and publish diagnostics localized to a single statement.
        //        But we are far away from that being important.
        //   3. Since we're already here, publish any diagnostics we discovered.
        //
        // We cache both the original program AST and evaluated AST so we can answer later queries
        // for original/optimized statements without re-evaluation.

        // 1. Parse
        let ScanResult {
            tokens,
            diagnostics: scan_diags,
        } = scan(&*source);
        let ParseResult {
            program,
            diagnostics: parse_diags,
        } = parse_statements(tokens, &source);
        let lint_diags = lint_stmt(&program, &source);
        // 2. Eval
        let EvaluationResult {
            simplified,
            diagnostics: eval_diags,
        } = evaluate(program.clone(), &self.context).expect("Evaluation failed.");

        // 3. Publish diagnostics
        let diags = [scan_diags, parse_diags, lint_diags, eval_diags]
            .iter()
            .flat_map(|diags| convert_diagnostics(diags, "slide", &uri, &source))
            .collect();

        // Final: save results
        self.registry.insert(
            uri.clone(),
            ProgramInfo {
                source,
                uri,
                original: program,
                simplified,
            },
        );

        diags
    }
}
