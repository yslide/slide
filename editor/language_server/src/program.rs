//! Module `program` describes a single slide program, and is the heart of the server's query and
//! analysis work.

use libslide::ProgramContext;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::ast::AST;
use crate::ptr::P;

#[derive(Debug)]
struct Analysis {
    original: RwLock<Option<AST>>,
    simplified: RwLock<Option<AST>>,
    diagnostics: RwLock<Option<Vec<Diagnostic>>>,
}

impl Analysis {
    fn unknown() -> Self {
        Self {
            original: RwLock::new(None),
            simplified: RwLock::new(None),
            diagnostics: RwLock::new(None),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Program {
    // TODO: make this a &str with the same lifetime as the entire document source (?)
    // May be less efficient actually if document is very large, think about this later.
    pub source: String,
    pub document_uri: P<Url>,
    #[allow(unused)]
    pub start: usize,
    #[allow(unused)]
    pub end: usize,

    pub context: P<ProgramContext>,

    analysis: Analysis,
}

impl Program {
    pub fn new(
        source: String,
        document_uri: P<Url>,
        start: usize,
        end: usize,
        context: P<ProgramContext>,
    ) -> Self {
        Self {
            source,
            document_uri,
            start,
            end,
            context,
            analysis: Analysis::unknown(),
        }
    }

    pub fn original_ast(&self) -> MappedRwLockReadGuard<AST> {
        self.with_analysis(|a| &a.original)
    }

    pub fn simplified_ast(&self) -> MappedRwLockReadGuard<AST> {
        self.with_analysis(|a| &a.simplified)
    }

    pub fn diagnostics(&self) -> MappedRwLockReadGuard<Vec<Diagnostic>> {
        self.with_analysis(|a| &a.diagnostics)
    }

    fn with_analysis<'a, T>(
        &'a self,
        cb: impl FnOnce(&'a Analysis) -> &'a RwLock<Option<T>>,
    ) -> MappedRwLockReadGuard<T> {
        self.ensure_analysis();
        RwLockReadGuard::map(cb(&self.analysis).read(), |data| {
            data.as_ref().expect("Bad state: analysis not complete")
        })
    }

    fn ensure_analysis(&self) {
        // TODO: more rigorous way to check for analysis? (Move analysis between uninitialized,
        // fresh, etc; provide tests)
        if self.analysis.diagnostics.read().is_none() {
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
            } = scan(&*self.source);
            let ParseResult {
                program,
                diagnostics: parse_diags,
            } = parse_statements(tokens, &self.source);
            let lint_diags = lint_stmt(&program, &self.source);
            // 2. Eval
            let EvaluationResult {
                simplified,
                diagnostics: eval_diags,
            } = evaluate(program.clone(), &self.context).expect("Evaluation failed.");

            // 3. Diagnostics
            let diags = [scan_diags, parse_diags, lint_diags, eval_diags]
                .iter()
                .flat_map(|diags| {
                    crate::shims::convert_diagnostics(
                        diags,
                        "slide",
                        &self.document_uri,
                        &self.source,
                    )
                })
                .collect();

            // Update fresh analysis
            *self.analysis.original.write() = Some(program);
            *self.analysis.simplified.write() = Some(simplified);
            *self.analysis.diagnostics.write() = Some(diags);
        }
    }
}

#[cfg(test)]
mod test {
    use super::Program;
    use crate::ptr::p;
    use tower_lsp::lsp_types::Url;

    fn with_fresh_program(program: &str, test: impl FnOnce(Program)) {
        let program = Program::new(
            program.to_owned(),
            p(Url::parse("file:///test").unwrap()),
            0,
            program.len(),
            p(libslide::ProgramContext::default()),
        );
        test(program);
    }

    #[test]
    fn diagnostics() {
        with_fresh_program("1 + ", |p| assert_eq!(p.diagnostics().len(), 1));
    }

    #[test]
    fn original_ast() {
        with_fresh_program("1 + a", |p| {
            assert_eq!(p.original_ast().to_string(), "1 + a")
        });
    }

    #[test]
    fn simplified_ast() {
        with_fresh_program("1 + 2", |p| assert_eq!(p.simplified_ast().to_string(), "3"));
    }
}
