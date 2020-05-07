use super::*;

/// A trait for transforming a grammar instance into another instance of the same grammar.
/// This transformer takes ownership of the expressions it transforms.
pub trait Transformer {
    /// Transforms an expr into another expr.
    fn transform_expr(&self, item: Expr) -> Expr;

    /// Default const transformer - return the same const expr
    fn transform_const(&self, item: f64) -> Expr {
        Expr::Const(item)
    }

    /// Default var transformer - return the same var expr
    fn transform_var(&self, item: Var) -> Expr {
        Expr::Var(item)
    }

    /// Default binary expression transformer - return the same binary expression with its operands
    /// transformed.
    fn transform_binary_expr(&self, item: BinaryExpr) -> Expr {
        let lhs = self.transform_expr(*item.lhs);
        let rhs = self.transform_expr(*item.rhs);
        BinaryExpr {
            op: item.op,
            lhs: lhs.into(),
            rhs: rhs.into(),
        }
        .into()
    }

    /// Default unary expression transformer - return the same unary expression with its operand
    /// transformed.
    fn transform_unary_expr(&self, item: UnaryExpr) -> Expr {
        let rhs = self.transform_expr(*item.rhs);
        UnaryExpr {
            op: item.op,
            rhs: rhs.into(),
        }
        .into()
    }

    /// Default paren'd expression transformer - transform the nested expression and return it
    /// paren'd.
    fn transform_parend(&self, item: Expr) -> Expr {
        Expr::Parend(self.transform_expr(item).into())
    }

    /// Default braced expression transformer - transform the nested expression and return it
    /// braced.
    fn transform_braced(&self, item: Expr) -> Expr {
        Expr::Braced(self.transform_expr(item).into())
    }

    /// Routes expression variants to their respective transformation functions.
    /// This transformer is intended to provide a routing mechanism for `transform_expr`.
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
