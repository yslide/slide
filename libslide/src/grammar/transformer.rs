use super::*;

/// A trait for transforming a grammar instance into another instance of the same grammar.
/// This transformer takes ownership of the expressions it transforms.
pub trait Transformer {
    fn transform_expr(&self, item: Expr) -> Expr;
    fn transform_const(&self, item: f64) -> Expr;
    fn transform_var(&self, item: Var) -> Expr;
    fn transform_binary_expr(&self, item: BinaryExpr) -> Expr;
    fn transform_unary_expr(&self, item: UnaryExpr) -> Expr;
    fn transform_parend(&self, item: Expr) -> Expr;
    fn transform_braced(&self, item: Expr) -> Expr;

    fn multiplex_transform_expr(&self, item: Expr) -> Expr {
        match item {
            Expr::Const(f) => self.transform_const(f),
            Expr::Var(v) => self.transform_var(v),
            Expr::BinaryExpr(binary_expr) => self.transform_binary_expr(binary_expr),
            Expr::UnaryExpr(unary_expr) => self.transform_unary_expr(unary_expr),
            Expr::Parend(expr) => self.transform_parend(*expr),
            Expr::Braced(expr) => self.transform_braced(*expr),
        }
    }
}
