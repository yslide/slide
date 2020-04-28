use crate::parser::types::*;

/// An expression visitor.
/// This visitor takes ownership of the expressions it visits.
pub trait Visitor {
    type Result;

    fn visit_expr(&mut self, item: Expr) -> Self::Result;

    fn visit_float(&mut self, item: f64) -> Self::Result;
    fn visit_int(&mut self, item: i64) -> Self::Result;
    fn visit_var(&mut self, item: Var) -> Self::Result;
    fn visit_binary_expr(&mut self, item: BinaryExpr) -> Self::Result;
    fn visit_unary_expr(&mut self, item: UnaryExpr) -> Self::Result;
}
