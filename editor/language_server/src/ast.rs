//! Module `ast` provides utilities visiting the slide AST.

use libslide::visit::StmtVisitor;
use libslide::*;

pub(crate) type AST = StmtList;

/// Finds the nearest slide [expression](libslide::Expr) around an offset
/// position. For example, in
///
/// ```math
/// 5 * ((1 |+ 2) / 4)
/// ```
///
/// where `|` is the offset position, the expression corresponding to `1 + 2`
/// will be found.
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
