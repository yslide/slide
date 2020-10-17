//! Traits for visiting slide grammar trees.

use super::*;
use crate::Span;

/// Descends down a statement list.
pub fn descend_stmt_list<'a, V: StmtVisitor<'a>>(visitor: &mut V, stmt_list: &'a StmtList) {
    for stmt in stmt_list.iter() {
        visitor.visit_stmt(stmt);
    }
}

/// Descends down a statement.
pub fn descend_stmt<'a, V: StmtVisitor<'a>>(visitor: &mut V, stmt: &'a Stmt) {
    visitor.visit_stmt_kind(&stmt.kind)
}

/// Descends down a specific statement kind.
pub fn descend_stmt_kind<'a, V: StmtVisitor<'a>>(visitor: &mut V, stmt_kind: &'a StmtKind) {
    match stmt_kind {
        StmtKind::Expr(expr) => visitor.visit_expr(expr),
        StmtKind::Assignment(asgn) => visitor.visit_asgn(asgn),
    }
}

/// Descends down an assignment.
pub fn descend_asgn<'a, V: StmtVisitor<'a>>(visitor: &mut V, asgn: &'a Assignment) {
    visitor.visit_expr(&asgn.lhs);
    visitor.visit_asgn_op(&asgn.asgn_op);
    visitor.visit_expr(&asgn.rhs);
}

/// Descends down an expression.
pub fn descend_expr<'a, V: StmtVisitor<'a>>(visitor: &mut V, expr: &'a RcExpr) {
    match expr.as_ref() {
        Expr::Const(k) => visitor.visit_const(k, expr.span),
        Expr::Var(v) => visitor.visit_var(v, expr.span),
        Expr::BinaryExpr(b) => visitor.visit_binary(b, expr.span),
        Expr::UnaryExpr(u) => visitor.visit_unary(u, expr.span),
        Expr::Parend(p) => visitor.visit_parend(p, expr.span),
        Expr::Bracketed(b) => visitor.visit_bracketed(b, expr.span),
    }
}

/// Descends down a binary expression.
pub fn descend_binary<'a, V: StmtVisitor<'a>>(
    visitor: &mut V,
    expr: &'a BinaryExpr<RcExpr>,
    _span: Span,
) {
    visitor.visit_expr(&expr.lhs);
    visitor.visit_binary_op(expr.op);
    visitor.visit_expr(&expr.rhs);
}

/// Descends down a unary expression.
pub fn descend_unary<'a, V: StmtVisitor<'a>>(
    visitor: &mut V,
    expr: &'a UnaryExpr<RcExpr>,
    _span: Span,
) {
    visitor.visit_unary_op(expr.op);
    visitor.visit_expr(&expr.rhs);
}

/// Descends down a parenthesized expression.
pub fn descend_parend<'a, V: StmtVisitor<'a>>(visitor: &mut V, expr: &'a RcExpr, _span: Span) {
    visitor.visit_expr(expr);
}

/// Descends down a bracketed expression.
pub fn descend_bracketed<'a, V: StmtVisitor<'a>>(visitor: &mut V, expr: &'a RcExpr, _span: Span) {
    visitor.visit_expr(expr);
}

/// Describes a [statement list](super::StmtList) visitor.
pub trait StmtVisitor<'a>: Sized {
    /// Visits a statement list.
    fn visit_stmt_list(&mut self, stmt_list: &'a StmtList) {
        descend_stmt_list(self, stmt_list);
    }

    /// Visits a statement.
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        descend_stmt(self, stmt);
    }

    /// Visits a specific statement kind.
    fn visit_stmt_kind(&mut self, stmt_kind: &'a StmtKind) {
        descend_stmt_kind(self, stmt_kind);
    }

    /// Visits an assignment.
    fn visit_asgn(&mut self, asgn: &'a Assignment) {
        descend_asgn(self, asgn);
    }

    /// Visits an assignment operator.
    fn visit_asgn_op(&mut self, _asgn_op: &'a AssignmentOp) {}

    /// Visits an expression.
    fn visit_expr(&mut self, expr: &'a RcExpr) {
        descend_expr(self, expr);
    }

    /// Visits a constant.
    fn visit_const(&mut self, _konst: &'a f64, _span: Span) {}

    /// Visits a variable.
    fn visit_var(&mut self, _var: &'a InternedStr, _span: Span) {}

    /// Visits a binary operator.
    fn visit_binary_op(&mut self, _op: BinaryOperator) {}

    /// Visits a binary expression.
    fn visit_binary(&mut self, expr: &'a BinaryExpr<RcExpr>, span: Span) {
        descend_binary(self, expr, span);
    }

    /// Visits a unary operator.
    fn visit_unary_op(&mut self, _op: UnaryOperator) {}

    /// Visits a unary expression.
    fn visit_unary(&mut self, expr: &'a UnaryExpr<RcExpr>, span: Span) {
        descend_unary(self, expr, span);
    }

    /// Visits a parenthesized expression.
    fn visit_parend(&mut self, expr: &'a RcExpr, span: Span) {
        descend_parend(self, expr, span);
    }

    /// Visits a bracketed expression.
    fn visit_bracketed(&mut self, expr: &'a RcExpr, span: Span) {
        descend_bracketed(self, expr, span);
    }
}

/// Descends down an expression pattern.
pub fn descend_expr_pat<'a, V: ExprPatVisitor<'a>>(visitor: &mut V, expr_pat: &'a RcExprPat) {
    match expr_pat.as_ref() {
        ExprPat::Const(k) => visitor.visit_const(k),
        ExprPat::VarPat(v) => visitor.visit_var_pat(v, expr_pat.span),
        ExprPat::ConstPat(k) => visitor.visit_const_pat(k, expr_pat.span),
        ExprPat::AnyPat(a) => visitor.visit_any_pat(a, expr_pat.span),
        ExprPat::BinaryExpr(b) => visitor.visit_binary_expr_pat(b),
        ExprPat::UnaryExpr(u) => visitor.visit_unary_expr_pat(u),
        ExprPat::Parend(p) => visitor.visit_parend_expr_pat(p, expr_pat.span),
        ExprPat::Bracketed(b) => visitor.visit_bracketed_expr_pat(b, expr_pat.span),
    }
}

/// Descends down a binary expression pattern.
pub fn descend_binary_expr_pat<'a, V: ExprPatVisitor<'a>>(
    visitor: &mut V,
    expr: &'a BinaryExpr<RcExprPat>,
) {
    visitor.visit_expr_pat(&expr.lhs);
    visitor.visit_binary_op(expr.op);
    visitor.visit_expr_pat(&expr.rhs);
}

/// Descends down a unary expression pattern.
pub fn descend_unary_expr_pat<'a, V: ExprPatVisitor<'a>>(
    visitor: &mut V,
    expr: &'a UnaryExpr<RcExprPat>,
) {
    visitor.visit_unary_op(expr.op);
    visitor.visit_expr_pat(&expr.rhs);
}

/// Descends down a parenthesized expression pattern.
pub fn descend_parend_expr_pat<'a, V: ExprPatVisitor<'a>>(
    visitor: &mut V,
    expr: &'a RcExprPat,
    _span: Span,
) {
    visitor.visit_expr_pat(expr);
}

/// Descends down a bracketed expression pattern.
pub fn descend_bracketed_expr_pat<'a, V: ExprPatVisitor<'a>>(
    visitor: &mut V,
    expr: &'a RcExprPat,
    _span: Span,
) {
    visitor.visit_expr_pat(expr);
}

/// Describes an [expression pattern](super::ExprPat) visitor.
pub trait ExprPatVisitor<'a>: Sized {
    /// Visits an expression pattern.
    fn visit_expr_pat(&mut self, expr_pat: &'a RcExprPat) {
        descend_expr_pat(self, expr_pat);
    }

    /// Visits a constant.
    fn visit_const(&mut self, _konst: &f64) {}

    /// Visits a variable pattern.
    fn visit_var_pat(&mut self, _var_pat: &'a str, _span: Span) {}

    /// Visits a constant pattern.
    fn visit_const_pat(&mut self, _const_pat: &'a str, _span: Span) {}

    /// Visits an any pattern.
    fn visit_any_pat(&mut self, _any_pat: &'a str, _span: Span) {}

    /// Visits a binary operator.
    fn visit_binary_op(&mut self, _op: BinaryOperator) {}

    /// Visits a binary expression pattern.
    fn visit_binary_expr_pat(&mut self, expr: &'a BinaryExpr<RcExprPat>) {
        descend_binary_expr_pat(self, expr);
    }

    /// Visits a unary operator.
    fn visit_unary_op(&mut self, _op: UnaryOperator) {}

    /// Visits a unary expression pattern.
    fn visit_unary_expr_pat(&mut self, expr: &'a UnaryExpr<RcExprPat>) {
        descend_unary_expr_pat(self, expr);
    }

    /// Visits a parenthesized expression pattern.
    fn visit_parend_expr_pat(&mut self, expr: &'a RcExprPat, span: Span) {
        descend_parend_expr_pat(self, expr, span);
    }

    /// Visits a bracketed expression pattern.
    fn visit_bracketed_expr_pat(&mut self, expr: &'a RcExprPat, span: Span) {
        descend_bracketed_expr_pat(self, expr, span);
    }
}
