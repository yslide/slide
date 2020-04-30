use crate::grammar::*;

/// An expression visitor.
/// This visitor takes ownership of the expressions it visits.
pub trait Visitor {
    type Result;

    fn visit_expr(&mut self, item: Expr) -> Self::Result {
        match item {
            Expr::Float(f) => self.visit_float(f),
            Expr::Int(i) => self.visit_int(i),
            Expr::Var(v) => self.visit_var(v),
            Expr::BinaryExpr(binary_expr) => self.visit_binary_expr(binary_expr),
            Expr::UnaryExpr(unary_expr) => self.visit_unary_expr(unary_expr),
        }
    }

    fn visit_float(&mut self, item: f64) -> Self::Result;
    fn visit_int(&mut self, item: i64) -> Self::Result;
    fn visit_var(&mut self, item: Var) -> Self::Result;
    fn visit_binary_expr(&mut self, item: BinaryExpr) -> Self::Result;
    fn visit_unary_expr(&mut self, item: UnaryExpr) -> Self::Result;
}
