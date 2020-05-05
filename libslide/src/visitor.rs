use crate::grammar::*;

/// An expression visitor.
/// This visitor takes ownership of the expressions it visits.
pub trait Visitor {
    type Result;

    fn visit_expr(&mut self, item: Expr) -> Self::Result {
        match item {
            Expr::Const(f) => self.visit_const(f),
            Expr::Var(v) => self.visit_var(v),
            Expr::BinaryExpr(binary_expr) => self.visit_binary_expr(binary_expr),
            Expr::UnaryExpr(unary_expr) => self.visit_unary_expr(unary_expr),
            Expr::Parend(expr) => self.visit_parend(*expr),
            Expr::Braced(expr) => self.visit_braced(*expr),
        }
    }

    fn visit_const(&mut self, item: f64) -> Self::Result;
    fn visit_var(&mut self, item: Var) -> Self::Result;
    fn visit_binary_expr(&mut self, item: BinaryExpr) -> Self::Result;
    fn visit_unary_expr(&mut self, item: UnaryExpr) -> Self::Result;

    /// Default visitor for parenthesized expressions: just visit the expression.
    fn visit_parend(&mut self, item: Expr) -> Self::Result {
        self.visit_expr(item)
    }

    /// Default visitor for braced expressions: just visit the expression.
    fn visit_braced(&mut self, item: Expr) -> Self::Result {
        self.visit_expr(item)
    }
}
