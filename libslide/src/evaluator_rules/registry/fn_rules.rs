use crate::grammar::*;
use crate::math::*;
use crate::utils::*;

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
            Expr::UnaryExpr(UnaryExpr { op: $op, rhs }) => Some(*rhs),
            _ => None,
        }
    };
}

pub(super) fn add(expr: InternedExpr) -> Option<InternedExpr> {
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
    args.push(intern_expr!(konst.into()));

    Some(unflatten_binary_expr(
        &args,
        BinaryOperator::Plus,
        UnflattenStrategy::Left,
    ))
}

pub(super) fn subtract(expr: InternedExpr) -> Option<InternedExpr> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Minus)?;
    Some(intern_expr!(Expr::Const(l - r)))
}

pub(super) fn multiply(expr: InternedExpr) -> Option<InternedExpr> {
    let mut args = get_flattened_binary_args!(expr, BinaryOperator::Mult)?;
    let mut konst = 1.;
    let mut i = 0;
    for _ in 0..args.len() {
        match args[i].as_ref() {
            Expr::Const(f) => {
                konst *= f;
                args.swap_remove(i);
            }
            _ => i += 1,
        }
    }
    args.push(intern_expr!(konst.into()));

    Some(unflatten_binary_expr(
        &args,
        BinaryOperator::Mult,
        UnflattenStrategy::Left,
    ))
}

pub(super) fn divide(expr: InternedExpr) -> Option<InternedExpr> {
    match expr.as_ref() {
        Expr::BinaryExpr(BinaryExpr {
            op: BinaryOperator::Div,
            lhs,
            rhs,
        }) => match (lhs.as_ref(), rhs.as_ref()) {
            (Expr::Const(l), Expr::Const(r)) => Some(intern_expr!(Expr::Const(l / r))),
            _ => {
                // Now we try to convert the numerator/denominator into polynomials and cancel them.
                let (numerator, relative_to) = Poly::from_expr(*lhs, None).ok()?;
                let relative_to = match relative_to {
                    Some(e) => e,
                    // Cancelling should only work with term'd polynomials. If the expression has
                    // no terms for whatever reason, let another rule take care of it.
                    None => return None,
                };
                let (denominator, _) = Poly::from_expr(*rhs, Some(relative_to)).ok()?;
                let (_, numerator, denominator) = gcd_poly_zz_heu(numerator, denominator).ok()?;

                // Woo! The polynomials have a gcd we can cancel them with.
                let numer_expr = numerator.to_expr(relative_to);
                if denominator.is_one() {
                    Some(numer_expr)
                } else {
                    let denom_expr = denominator.to_expr(relative_to);
                    let division = BinaryExpr::div(numer_expr, denom_expr);
                    Some(intern_expr!(Expr::BinaryExpr(division)))
                }
            }
        },
        _ => None,
    }
}

pub(super) fn modulo(expr: InternedExpr) -> Option<InternedExpr> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Mod)?;
    Some(intern_expr!(Expr::Const(l % r)))
}

pub(super) fn exponentiate(expr: InternedExpr) -> Option<InternedExpr> {
    let (l, r) = get_binary_args!(expr, BinaryOperator::Exp)?;
    Some(intern_expr!(Expr::Const(l.powf(*r))))
}

pub(super) fn posate(expr: InternedExpr) -> Option<InternedExpr> {
    get_unary_arg!(expr, UnaryOperator::SignPositive)
}

pub(super) fn negate(expr: InternedExpr) -> Option<InternedExpr> {
    match get_unary_arg!(expr, UnaryOperator::SignNegative)?.as_ref() {
        Expr::Const(n) => Some(intern_expr!(Expr::Const(-n))),
        _ => None,
    }
}
