//! Module `folding_ranges` provides service to determine ranges that a user may want to fold in a
//! slide program.

use crate::Program;

use libslide::visit::StmtVisitor;
use libslide::*;

impl Program {
    /// Returns [span](Span)s of foldable ranges in a program.
    /// Such ranges generally correspond to expressions and definitions in a program.
    pub fn folding_ranges(&self) -> Vec<Span> {
        let ast = self.original_ast();
        let mut ranges_collector = FoldingRangeCollector {
            folding_ranges: vec![],
        };
        ranges_collector.visit_stmt_list(&ast);
        ranges_collector.folding_ranges
    }
}

struct FoldingRangeCollector {
    folding_ranges: Vec<Span>,
}
impl<'a> StmtVisitor<'a> for FoldingRangeCollector {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        self.folding_ranges.push(*stmt.span());
    }
}
