//! This module tries to flatten expressions as far as possible without the intervention of more
//! complex evaluation rules. Flattening might be used as a "first pass" to normalize an expression,
//! or by a more complex rule for the same reason.
//!
//! # Examples
//!
//! ## Targets of this module
//!
//! ```text
//! 1 + 2 + 3 -> 6
//! 1 - 5x / x -> -4
//! ```
//!
//! ## Non targets of this module
//!
//! ```text
//! x^2 + 4x + 4 -> (x + 2)^2
//! ```

use crate::grammar::*;
use crate::utils::{unflatten_binary_expr, UnflattenStrategy};

use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

/// Attempts to flatten an expression, folding constant expressions and like terms.
///
/// ## Examples
///
/// ```text
/// 1 + 2 + 3 -> 6
/// 1 - 5x / x -> -4
/// ```
pub fn flatten_expr(expr: &Rc<Expr>) -> Rc<Expr> {
    match expr.as_ref() {
        // #a -> #a, $a -> $a
        // We can't do better than this.
        Expr::Const(_) | Expr::Var(_) => Rc::clone(expr),

        // (_a) -> _a, [_a] -> _a
        // We can't do better than this.
        Expr::Parend(inner) | Expr::Bracketed(inner) => Rc::clone(inner),

        // _a + _b -> _c
        // _a - _b -> _c
        Expr::BinaryExpr(BinaryExpr { op, lhs, rhs })
            if op == &BinaryOperator::Plus || op == &BinaryOperator::Minus =>
        {
            flatten_add_or_sub(lhs, rhs, op == &BinaryOperator::Minus)
        }

        // TODO: handle everything else
        _ => Rc::clone(expr),
    }
}

/// Flattens an addition or subtraction, folding constants and like terms as far as possible.
/// The flattened expression is always normalized to an addition.
///
/// ```text
/// 1 + 2x - 3 + x -> -2 + 3x
/// ```
pub fn flatten_add_or_sub(o_lhs: &Rc<Expr>, o_rhs: &Rc<Expr>, is_subtract: bool) -> Rc<Expr> {
    let lhs = flatten_expr(o_lhs);
    let rhs = flatten_expr(o_rhs);

    // Leading coefficients to fold constants into.
    let mut coeff = 0.;
    // Terms -> coefficients present in the expression.
    let mut terms = HashMap::<&Rc<Expr>, f64>::new();

    let mut args = VecDeque::with_capacity(2);
    let base_args = [lhs, rhs];
    args.extend(base_args.iter());

    // Number of arguments we need to visit before we hit the negative (subtracted) side of this
    // expression. This is only relevant if `is_subtract` is true.
    let mut args_before_neg = 1;

    while let Some(arg) = args.pop_front() {
        let is_neg = is_subtract && args_before_neg == 0;
        args_before_neg -= 1;

        match arg.as_ref() {
            Expr::Const(konst) => {
                if is_neg {
                    coeff -= konst;
                } else {
                    coeff += konst;
                }
            }
            Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) if op == &BinaryOperator::Plus => {
                // Note: addition is commutative, so we add the nested terms to list of terms we
                // should flatten.
                // If we're on the negative side of the original expression, the nested terms should
                // also be negated and are added to the back. Otherwise they are positive and are
                // added to the front.
                if is_neg {
                    // If we're on the negative side of the expression, add the terms to the back
                    // since they should also be negated.
                    args.push_back(lhs);
                    args.push_back(rhs);
                } else {
                    // If we're on the positive side of the expression, the terms should also be
                    // positive.
                    args.push_front(lhs);
                    args.push_front(rhs);
                    args_before_neg += 2;
                }
            }
            _ => {
                // Otherwise the arg is something we cannot further decompose in an add/sub context
                // (e.g. a variable or an exponentiation), so add it as a term.
                // TODO: see if we can handle other things more granularly
                let entry = terms.entry(arg).or_insert(0.);
                if is_neg {
                    *entry -= 1.;
                } else {
                    *entry += 1.;
                }
            }
        }
    }

    let mut new_args: Vec<Rc<Expr>> = Vec::with_capacity(1 + terms.len());
    if coeff != 0. {
        new_args.push(Rc::from(Expr::Const(coeff)));
    }
    for (term, coeff) in terms {
        if coeff == 0. {
            // The happiest path :)
            continue;
        } else if (coeff - 1.).abs() < std::f64::EPSILON {
            // coeff == 1
            new_args.push(Rc::clone(term));
        } else if (coeff - -1.).abs() < std::f64::EPSILON {
            // coeff == -1
            let neg = UnaryExpr::negate(Rc::clone(term));
            new_args.push(Rc::from(Expr::UnaryExpr(neg)));
        } else {
            let mult = BinaryExpr::mult(Expr::Const(coeff), Rc::clone(term));
            let expr: Expr = mult.into();
            new_args.push(Rc::from(expr));
        }
    }

    match new_args.len() {
        0 => Rc::from(Expr::Const(0.)),
        1 => new_args.remove(0),
        _ => unflatten_binary_expr(&new_args, BinaryOperator::Plus, UnflattenStrategy::Left),
    }
}

#[cfg(test)]
mod tests {
    use super::flatten_expr;
    use crate::grammar::*;
    use crate::utils::normalize;
    use crate::{parse_expression, scan};

    use std::rc::Rc;

    fn parse(program: &str) -> Expr {
        let tokens = scan(program);
        let (parsed, _) = parse_expression(tokens);
        match parsed {
            Stmt::Expr(expr) => expr,
            _ => unreachable!(),
        }
    }

    static CASES: &[&str] = &[
        "1 + 2 + 3 -> 6",
        "1 + x + x -> (+ 1 (* x 2))",
        // TODO: currently (+ x (* 2 x))
        // "x + x + x -> (* 3 x)",
        "x + y + 1 -> (+ (+ x y) 1)",
        "x + 0 -> x",
        "1 - 1 -> 0",
        "1 + 2 - 3 -> 0",
        "1 - 2 + 3 -> 2",
        "a - a + 1 -> 1",
        "a + 1 - 1 -> a",
    ];

    #[test]
    fn flatten_cases() {
        for case in CASES {
            let mut split = case.split(" -> ");
            let expr = parse(split.next().unwrap());
            let expected_flattened = split.next().unwrap();

            let flattened = normalize(flatten_expr(&Rc::from(expr))).s_form();

            assert_eq!(flattened, expected_flattened);
        }
    }
}
