//! Module `ast` provides utilities visiting the slide AST.

use libslide::visit::StmtVisitor;
use libslide::*;

pub type Ast = StmtList;

pub enum AstItem<'a> {
    Assignment(&'a Assignment),
    Expr(&'a RcExpr),
}

impl<'a> AstItem<'a> {
    pub fn span(&self) -> Span {
        match self {
            Self::Assignment(a) => a.span,
            Self::Expr(e) => e.span,
        }
    }
}

/// Retrieves the path of items descended to a program byte offset. The path is ordered from
/// broadest to narrowest item around the offset. For example, in
///
/// ```math
/// a = 1 ^ (|2 + 3)
/// ```
///
/// The path would be { "a = 1 ^ (2 + 3)", "1 ^ (2 + 3)", "2 + 3", "2" }.
pub fn get_item_path_to_offset(pos: usize, program: &Ast) -> Vec<AstItem> {
    let mut finder = ItemPathFinder { pos, path: vec![] };
    finder.visit_stmt_list(program);
    finder.path
}
struct ItemPathFinder<'a> {
    pos: usize,
    path: Vec<AstItem<'a>>,
}
impl<'a> StmtVisitor<'a> for ItemPathFinder<'a> {
    fn visit_asgn(&mut self, asgn: &'a Assignment) {
        if asgn.span.contains(self.pos) {
            self.path.push(AstItem::Assignment(asgn));
            visit::descend_asgn(self, asgn);
        }
    }

    fn visit_expr(&mut self, expr: &'a RcExpr) {
        if expr.span.contains(self.pos) {
            self.path.push(AstItem::Expr(expr));
            visit::descend_expr(self, expr);
        }
    }
}

/// Finds the nearest slide [expression](libslide::Expr) around an offset
/// position. For example, in
///
/// ```math
/// 5 * ((1 |+ 2) / 4)
/// ```
///
/// where `|` is the offset position, the expression corresponding to `1 + 2`
/// will be found.
pub fn get_tightest_expr(pos: usize, program: &Ast) -> Option<&RcExpr> {
    match get_item_path_to_offset(pos, program).pop() {
        Some(AstItem::Expr(e)) => Some(e),
        _ => None,
    }
}

/// Finds the slide item exactly covering a [span](Span), if any. For example,
///
/// ```math
/// a = 1 + 2 * 3 / 4 ^ 5
///     ^                 will return "1"
///     ^^^^^             will return "1 + 2"
///             ^^^^^     will return "3 / 4"
/// ^^^^^^^^^^^^^^^^^^^^^ will return "a = 1 + 2 * 3 / 4 ^ 5"
///       ^^^^^           will return nothing
/// ```
pub fn get_item_at_span(span: Span, program: &Ast) -> Option<AstItem> {
    let mut finder = ItemAtSpanFinder { item: None, span };
    finder.visit_stmt_list(program);
    finder.item
}
struct ItemAtSpanFinder<'a> {
    item: Option<AstItem<'a>>,
    span: Span,
}
impl<'a> StmtVisitor<'a> for ItemAtSpanFinder<'a> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if stmt.span().supersets(self.span) {
            visit::descend_stmt(self, stmt);
        }
    }

    fn visit_asgn(&mut self, asgn: &'a Assignment) {
        if asgn.span == self.span {
            self.item = Some(AstItem::Assignment(asgn));
        } else if asgn.span.supersets(self.span) {
            visit::descend_asgn(self, asgn);
        }
    }

    fn visit_expr(&mut self, expr: &'a RcExpr) {
        if expr.span == self.span {
            self.item = Some(AstItem::Expr(expr));
        } else if expr.span.supersets(self.span) {
            visit::descend_expr(self, expr);
        }
    }
}
