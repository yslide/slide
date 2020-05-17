use crate::grammar::*;
use crate::utils::*;

use std::rc::Rc;

macro_rules! get_binary_args {
    ($expr:expr, $op:pat) => {
        match $expr.as_ref() {
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

macro_rules! get_flattened_binary_args {
    ($expr:expr, $op:expr) => {
        match $expr.as_ref() {
            Expr::BinaryExpr(child) if child.op == $op => {
                Some(get_flattened_binary_args($expr, $op))
            }
            _ => None,
        }
    };
}

macro_rules! get_unary_arg {
    ($expr:expr, $op:pat) => {
        match $expr.as_ref() {
            Expr::UnaryExpr(UnaryExpr { op: $op, rhs }) => Some(rhs),
            _ => None,
        }
    };
}

pub(super) fn add(expr: Rc<Expr>) -> Option<Rc<Expr>> {
    let mut args = get_flattened_binary_args!(expr, BinaryOperator::Plus)?;
    let mut konst = 0.;
    let mut i = 0;
    for _ in 0..args.len() {
        match args[i].as_ref() {
            Expr::Const(f) => {
                konst += f;
                args.swap_remove(i);
            }
            _ => i += 1,
        }
    }
    if konst != 0. {
        args.push(Rc::new(konst.into()));
    }

    Some(unflatten_binary_expr(
        &args,
        BinaryOperator::Plus,
        UnflattenStrategy::Left,
    ))
}

pub(super) fn subtract(expr: Rc<Expr>) -> Option<Rc<Expr>> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Minus)?;
    Some(Rc::new(Expr::Const(l - r)))
}

pub(super) fn multiply(expr: Rc<Expr>) -> Option<Rc<Expr>> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Mult)?;
    Some(Rc::new(Expr::Const(l * r)))
}

pub(super) fn divide(expr: Rc<Expr>) -> Option<Rc<Expr>> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Div)?;
    Some(Rc::new(Expr::Const(l / r)))
}

pub(super) fn modulo(expr: Rc<Expr>) -> Option<Rc<Expr>> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Mod)?;
    Some(Rc::new(Expr::Const(l % r)))
}

pub(super) fn exponentiate(expr: Rc<Expr>) -> Option<Rc<Expr>> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Exp)?;
    Some(Rc::new(Expr::Const(l.powf(*r))))
}

pub(super) fn posate(expr: Rc<Expr>) -> Option<Rc<Expr>> {
    get_unary_arg!(expr, UnaryOperator::SignPositive).map(Rc::clone)
}

pub(super) fn negate(expr: Rc<Expr>) -> Option<Rc<Expr>> {
    match get_unary_arg!(expr, UnaryOperator::SignNegative)?.as_ref() {
        Expr::Const(n) => Some(Rc::new(Expr::Const(-n))),
        _ => None,
    }
}
