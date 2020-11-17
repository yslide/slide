//! Module `ast` provides utilities visiting the slide AST.

use libslide::visit::StmtVisitor;
use libslide::*;

/// Returns the most immediate expression at a file position, if any is present.
pub fn get_tightest_expr(pos: usize, program: &StmtList) -> Option<&RcExpr> {
    let mut finder = ExprFinder {
        tightest: None,
        pos,
    };
    finder.visit_stmt_list(program);
    finder.tightest
}
struct ExprFinder<'a> {
    tightest: Option<&'a RcExpr>,
    pos: usize,
}
impl<'a> StmtVisitor<'a> for ExprFinder<'a> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if stmt.span().contains(self.pos) {
            visit::descend_stmt(self, stmt);
        }
    }

    fn visit_expr(&mut self, expr: &'a RcExpr) {
        if expr.span.contains(self.pos) {
            self.tightest = Some(expr);
            visit::descend_expr(self, expr);
        }
    }
}
