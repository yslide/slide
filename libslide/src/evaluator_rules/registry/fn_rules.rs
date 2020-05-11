use crate::grammar::*;

macro_rules! get_binary_args {
    ($expr:expr, $op:pat) => {
        match $expr {
            Expr::BinaryExpr(BinaryExpr { op: $op, lhs, rhs }) => {
                match (lhs.as_ref(), rhs.as_ref()) {
                    (Expr::Const(l), Expr::Const(r)) => Some((l, r)),
                    _ => None,
                }
            }
            _ => None,
        }
    };
}

macro_rules! get_unary_arg {
    ($expr:expr, $op:pat) => {
        match $expr {
            Expr::UnaryExpr(UnaryExpr { op: $op, rhs }) => Some(rhs),
            _ => None,
        }
    };
}

pub(super) fn add(expr: &Expr) -> Option<Expr> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Plus)?;
    Some(Expr::Const(l + r))
}

pub(super) fn subtract(expr: &Expr) -> Option<Expr> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Minus)?;
    Some(Expr::Const(l - r))
}

pub(super) fn multiply(expr: &Expr) -> Option<Expr> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Mult)?;
    Some(Expr::Const(l * r))
}

pub(super) fn divide(expr: &Expr) -> Option<Expr> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Div)?;
    Some(Expr::Const(l / r))
}

pub(super) fn modulo(expr: &Expr) -> Option<Expr> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Mod)?;
    Some(Expr::Const(l % r))
}

pub(super) fn exponentiate(expr: &Expr) -> Option<Expr> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Exp)?;
    Some(Expr::Const(l.powf(*r)))
}

pub(super) fn posate(expr: &Expr) -> Option<Expr> {
    get_unary_arg!(expr, UnaryOperator::SignPositive).map(|e| e.as_ref().clone())
}

pub(super) fn negate(expr: &Expr) -> Option<Expr> {
    match get_unary_arg!(expr, UnaryOperator::SignNegative)?.as_ref() {
        Expr::Const(n) => Some(Expr::Const(-n)),
        _ => None,
    }
}
