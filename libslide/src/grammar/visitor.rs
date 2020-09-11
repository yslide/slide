//! Traits for visiting slide grammar trees.

use super::*;
use crate::Span;

/// Describes a [statement][super::Stmt] visitor.
pub trait StmtVisitor<'a> {
    fn visit(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::Expr(expr) => self.visit_expr(expr),
            Stmt::Assignment(asgn) => self.visit_asgn(asgn),
        }
    }

    fn visit_asgn(&mut self, asgn: &'a Assignment) {
        self.visit_var(&asgn.var);
        self.visit_expr(&asgn.rhs);
    }

    fn visit_expr(&mut self, expr: &'a InternedExpr) {
        match expr.as_ref() {
            Expr::Const(k) => self.visit_const(k),
            Expr::Var(v) => self.visit_var(v),
            Expr::BinaryExpr(b) => self.visit_binary(b),
            Expr::UnaryExpr(u) => self.visit_unary(u, expr.span),
            Expr::Parend(p) => self.visit_parend(p, expr.span),
            Expr::Bracketed(b) => self.visit_bracketed(b, expr.span),
        }
    }

    fn visit_const(&mut self, _konst: &'a f64) {}

    fn visit_var(&mut self, _var: &'a str) {}

    fn visit_binary_op(&mut self, _op: BinaryOperator) {}

    fn visit_binary(&mut self, expr: &'a BinaryExpr<InternedExpr>) {
        self.visit_expr(&expr.lhs);
        self.visit_binary_op(expr.op);
        self.visit_expr(&expr.rhs);
    }

    fn visit_unary_op(&mut self, _op: UnaryOperator) {}

    fn visit_unary(&mut self, expr: &'a UnaryExpr<InternedExpr>, _span: Span) {
        self.visit_unary_op(expr.op);
        self.visit_expr(&expr.rhs);
    }

    fn visit_parend(&mut self, expr: &'a InternedExpr, _span: Span) {
        self.visit_expr(expr);
    }

    fn visit_bracketed(&mut self, expr: &'a InternedExpr, _span: Span) {
        self.visit_expr(expr);
    }
}

/// Describes an [expression pattern][super::ExprPat] visitor.
pub trait ExprPatVisitor<'a> {
    fn visit(&mut self, expr_pat: &'a InternedExprPat) {
        match expr_pat.as_ref() {
            ExprPat::Const(k) => self.visit_const(k),
            ExprPat::VarPat(v) => self.visit_var_pat(v, expr_pat.span),
            ExprPat::ConstPat(k) => self.visit_const_pat(k, expr_pat.span),
            ExprPat::AnyPat(a) => self.visit_any_pat(a, expr_pat.span),
            ExprPat::BinaryExpr(b) => self.visit_binary(b),
            ExprPat::UnaryExpr(u) => self.visit_unary(u, expr_pat.span),
            ExprPat::Parend(p) => self.visit_parend(p, expr_pat.span),
            ExprPat::Bracketed(b) => self.visit_bracketed(b, expr_pat.span),
        }
    }

    fn visit_const(&mut self, _konst: &f64) {}

    fn visit_var_pat(&mut self, _var_pat: &'a str, _span: Span) {}

    fn visit_const_pat(&mut self, _const_pat: &'a str, _span: Span) {}

    fn visit_any_pat(&mut self, _any_pat: &'a str, _span: Span) {}

    fn visit_binary_op(&mut self, _op: BinaryOperator) {}

    fn visit_binary(&mut self, expr: &'a BinaryExpr<InternedExprPat>) {
        self.visit(&expr.lhs);
        self.visit_binary_op(expr.op);
        self.visit(&expr.rhs);
    }

    fn visit_unary_op(&mut self, _op: UnaryOperator) {}

    fn visit_unary(&mut self, expr: &'a UnaryExpr<InternedExprPat>, _span: Span) {
        self.visit_unary_op(expr.op);
        self.visit(&expr.rhs);
    }

    fn visit_parend(&mut self, expr: &'a InternedExprPat, _span: Span) {
        self.visit(expr);
    }

    fn visit_bracketed(&mut self, expr: &'a InternedExprPat, _span: Span) {
        self.visit(expr);
    }
}
