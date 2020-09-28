use super::*;
use crate::Span;

/// A trait for transforming one grammar into another.
/// This transformer takes ownership of the grammar it transforms.
pub trait Transformer<T: Grammar, U: Grammar> {
    fn transform(&self, item: T) -> U;
}

/// A trait for transforming one expression into another.
pub trait ExpressionTransformer<'a> {
    fn transform(&self, expr: &'a RcExpr) -> RcExpr {
        match expr.as_ref() {
            Expr::Const(k) => self.transform_const(k, expr.span),
            Expr::Var(v) => self.transform_var(v, expr.span),
            Expr::BinaryExpr(b) => self.transform_binary(b, expr.span),
            Expr::UnaryExpr(u) => self.transform_unary(u, expr.span),
            Expr::Parend(p) => self.transform_parend(p, expr.span),
            Expr::Bracketed(b) => self.transform_bracketed(b, expr.span),
        }
    }

    fn transform_const(&self, konst: &f64, span: Span) -> RcExpr {
        rc_expr!(Expr::Const(*konst), span)
    }

    fn transform_var(&self, var: &'a InternedStr, span: Span) -> RcExpr {
        rc_expr!(Expr::Var(*var), span)
    }

    fn transform_binary_op(&self, op: BinaryOperator) -> BinaryOperator {
        op
    }

    fn transform_binary(&self, expr: &'a BinaryExpr<RcExpr>, span: Span) -> RcExpr {
        rc_expr!(
            Expr::BinaryExpr(BinaryExpr {
                op: self.transform_binary_op(expr.op),
                lhs: self.transform(&expr.lhs),
                rhs: self.transform(&expr.rhs),
            }),
            span
        )
    }

    fn transform_unary_op(&self, op: UnaryOperator) -> UnaryOperator {
        op
    }

    fn transform_unary(&self, expr: &'a UnaryExpr<RcExpr>, span: Span) -> RcExpr {
        rc_expr!(
            Expr::UnaryExpr(UnaryExpr {
                op: self.transform_unary_op(expr.op),
                rhs: self.transform(&expr.rhs),
            }),
            span
        )
    }

    fn transform_parend(&self, expr: &'a RcExpr, span: Span) -> RcExpr {
        rc_expr!(Expr::Parend(self.transform(expr)), span)
    }

    fn transform_bracketed(&self, expr: &'a RcExpr, span: Span) -> RcExpr {
        rc_expr!(Expr::Bracketed(self.transform(expr)), span)
    }
}
