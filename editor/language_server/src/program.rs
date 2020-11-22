//! Module `program` describes a single slide program, and is the heart of the server's query and
//! analysis work.

use libslide::ProgramContext;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use tower_lsp::lsp_types::Url;

use crate::ast::AST;
use crate::ptr::P;

mod services;
pub use services::response;

/// Fully populated program analyses.
#[derive(Debug)]
struct CompletedAnalysis {
    /// The "original" AST parsed from the slide program text.
    original: AST,
    /// The AST of the slide program after undergoing evaluation.
    simplified: AST,
    /// Diagnostics for the slide program.
    diagnostics: Vec<response::ProgramDiagnostic>,
}

/// State of slide program analysis performed for a [Program](Program).
#[derive(Debug)]
enum Analysis {
    /// No analyses are known or complete.
    Unknown,
    /// All analyses are complete.
    Complete(CompletedAnalysis),
}

impl Analysis {
    /// Creates a fresh [`Analysis`](Analysis) with no analyses completed.
    fn unknown() -> Self {
        Self::Unknown
    }

    /// Creates a fresh [complete analysis](Analysis::Complete).
    fn fresh(
        original: AST,
        simplified: AST,
        diagnostics: Vec<response::ProgramDiagnostic>,
    ) -> Self {
        Self::Complete(CompletedAnalysis {
            original,
            simplified,
            diagnostics,
        })
    }

    fn is_complete(&self) -> bool {
        matches!(self, Analysis::Complete{..})
    }
}

/// A slide program found inside a [`Document`](crate::document_registry::Document).
/// Used to answer language queries made by a server session.
#[derive(Debug)]
pub(crate) struct Program {
    /// The text source of the slide program.
    // TODO: make this a &str with the same lifetime as the entire document source (?)
    // May be less efficient actually if document is very large, think about this later.
    pub source: String,
    /// The `Url` of the document this program resides in.
    pub document_uri: P<Url>,
    /// The start offset of this program in the enclosing document.
    pub start: usize,
    /// The end offset of this program in the enclosing document.
    pub end: usize,

    /// The slide [context](ProgramContext) the slide program described by this program should be
    /// processed and evaluated with.
    pub context: P<ProgramContext>,

    /// [`Analysis`](Analysis) performed for this program.
    ///
    /// Analysis is performed lazily, for example when diagnostics for the program are requested.
    analysis: RwLock<Analysis>,
}

impl Program {
    /// Creates a new [`Program`](Program) relative to its location in document.
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
            analysis: RwLock::new(Analysis::unknown()),
        }
    }

    /// Returns the "original" [AST](crate::ast::AST) of the program, after parsing but before
    /// evaluation.
    pub fn original_ast(&self) -> MappedRwLockReadGuard<AST> {
        MappedRwLockReadGuard::map(self.get_analysis(), |a| &a.original)
    }

    /// Returns the "simplified" [AST](crate::ast::AST) of the program, after evaluation.
    pub fn simplified_ast(&self) -> MappedRwLockReadGuard<AST> {
        MappedRwLockReadGuard::map(self.get_analysis(), |a| &a.simplified)
    }

    /// Returns diagnostics for the program.
    pub fn diagnostics(&self) -> MappedRwLockReadGuard<Vec<response::ProgramDiagnostic>> {
        MappedRwLockReadGuard::map(self.get_analysis(), |a| a.diagnostics.as_ref())
    }

    /// Ensures [analysis](Analysis) for the program is complete and returns the analysis data.
    fn get_analysis(&self) -> MappedRwLockReadGuard<CompletedAnalysis> {
        self.ensure_analysis();
        RwLockReadGuard::map(self.analysis.read(), |analysis| match analysis {
            Analysis::Complete(analysis) => analysis,
            Analysis::Unknown => unreachable!("Bad state: analysis is not complete"),
        })
    }

    /// Populates a program's [analysis data](Analysis) if it is not already complete.
    fn ensure_analysis(&self) {
        if self.analysis.read().is_complete() {
            // Analysis is already known, nothing more to do.
            return;
        }

        use libslide::*;

        // 1. Parse
        let ScanResult {
            tokens,
            diagnostics: scan_diags,
        } = scan(&*self.source);
        let ParseResult {
            program: original,
            diagnostics: parse_diags,
        } = parse_statements(tokens, &self.source);
        let lint_diags = lint_stmt(&original, &self.source);
        // 2. Eval
        let EvaluationResult {
            simplified,
            diagnostics: eval_diags,
        } = evaluate(original.clone(), &self.context).expect("Evaluation failed.");

        // 3. Diagnostics
        let diagnostics = [scan_diags, parse_diags, lint_diags, eval_diags]
            .iter()
            .flat_map(|diags| {
                services::diagnostics::convert_diagnostics(diags, "slide", &self.document_uri)
            })
            .collect();

        // Update fresh analysis
        *self.analysis.write() = Analysis::fresh(original, simplified, diagnostics);
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
