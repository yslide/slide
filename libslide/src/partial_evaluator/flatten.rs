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

use rug::Rational;
use std::collections::{BTreeMap, VecDeque};

/// Attempts to flatten an expression, folding constant expressions and like terms.
///
/// ## Examples
///
/// ```text
/// 1 + 2 + 3 -> 6
/// 1 - 5x / x -> -4
/// ```
///
/// Expressions flattened to a binary operation have the following conditions:
///
/// - Additions and subtractions become additions
pub fn flatten_expr(expr: RcExpr) -> RcExpr {
    match expr.as_ref() {
        // #a -> #a, $a -> $a
        // We can't do better than this.
        Expr::Const(_) | Expr::Var(_) => expr,

        // (_a) -> _a, [_a] -> _a
        // We can't do better than this.
        Expr::Parend(inner) | Expr::Bracketed(inner) => flatten_expr(inner.clone()),

        // _a + _b -> _c
        // _a - _b -> _c
        Expr::BinaryExpr(BinaryExpr { op, lhs, rhs })
            if op == &BinaryOperator::Plus || op == &BinaryOperator::Minus =>
        {
            flatten_add_or_sub(lhs.clone(), rhs.clone(), op == &BinaryOperator::Minus)
        }

        // _a * _b -> _c
        // _a / _b -> _c
        Expr::BinaryExpr(BinaryExpr { op, lhs, rhs })
            if op == &BinaryOperator::Mult || op == &BinaryOperator::Div =>
        {
            flatten_mul_or_div(lhs.clone(), rhs.clone(), op == &BinaryOperator::Div)
        }

        // TODO: handle everything else better
        Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) => {
            let lhs = flatten_expr(lhs.clone());
            let rhs = flatten_expr(rhs.clone());
            rc_expr!(
                Expr::BinaryExpr(BinaryExpr { op: *op, lhs, rhs }),
                expr.span
            )
        }

        Expr::UnaryExpr(UnaryExpr { op, rhs }) => {
            let rhs = flatten_expr(rhs.clone());
            rc_expr!(Expr::UnaryExpr(UnaryExpr { op: *op, rhs }), expr.span)
        }
    }
}

/// Flattens an addition or subtraction, folding constants and like terms as far as possible.
/// The flattened expression is always normalized to an addition.
///
/// ```text
/// 1 + 2x - 3 + x -> -2 + 3x
/// ```
fn flatten_add_or_sub(o_lhs: RcExpr, o_rhs: RcExpr, is_subtract: bool) -> RcExpr {
    let o_span = o_lhs.span.to(o_rhs.span);
    let lhs = flatten_expr(o_lhs);
    let rhs = flatten_expr(o_rhs);

    // Leading coefficients to fold constants into.
    let mut coeff = Rational::from(0);
    // Terms -> coefficients present in the expression.
    let mut terms = BTreeMap::<RcExpr, Rational>::new();

    let mut args = VecDeque::with_capacity(2);
    args.push_back(lhs);
    args.push_back(rhs);

    // If this is not a subtraction, the first two args are both on the add side.
    let mut args_before_sub = if is_subtract { 1 } else { 2 };

    while let Some(arg) = args.pop_front() {
        let sub_side = args_before_sub <= 0;
        args_before_sub -= 1;

        let arg = unwrap_expr(arg.clone());

        match arg.as_ref() {
            Expr::Const(konst) => {
                if sub_side {
                    coeff -= konst;
                } else {
                    coeff += konst;
                }
            }
            // `flatten` will always normalize add/sub expressions to add, so we only have to
            // handle that.
            Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) if op == &BinaryOperator::Plus => {
                if sub_side {
                    // 1 - (2 + 3) -> 1 - 2 - 3; add both operands to the sub side.
                    args.push_back(lhs.clone());
                    args.push_back(rhs.clone());
                } else {
                    // 1 + (2 + 3) -> 1 + 2 + 3
                    args.push_front(lhs.clone());
                    args.push_front(rhs.clone());
                    args_before_sub += 2;
                }
            }
            _ => {
                // Otherwise the arg is something we cannot further decompose in an add/sub context
                // (e.g. a variable or an exponentiation), so add it as a term.
                // TODO: see if we can handle other things more granularly
                let entry = terms.entry(arg).or_insert_with(|| Rational::from(0));
                if sub_side {
                    *entry -= 1;
                } else {
                    *entry += 1;
                }
            }
        }
    }

    let mut new_args: Vec<RcExpr> = Vec::with_capacity(1 + terms.len());
    if coeff != 0. {
        new_args.push(rc_expr!(Expr::Const(coeff), o_span));
    }
    for (term, coeff) in terms {
        if coeff == 0. {
            // The happiest path :)
            continue;
        } else if coeff == 1 {
            new_args.push(term.clone());
        } else if coeff == -1 {
            let neg = UnaryExpr::negate(term.clone());
            new_args.push(rc_expr!(Expr::UnaryExpr(neg), o_span));
        } else {
            let mult = BinaryExpr::mult(rc_expr!(Expr::Const(coeff), o_span), term.clone());
            new_args.push(rc_expr!(Expr::BinaryExpr(mult), o_span));
        }
    }

    match new_args.len() {
        0 => rc_expr!(Expr::Const(Rational::from(0)), o_span),
        1 => new_args.remove(0),
        _ => unflatten_binary_expr(&new_args, BinaryOperator::Plus, UnflattenStrategy::Left),
    }
}

/// Flattens a multiplication or division, folding constants and like terms as far as possible.
/// The flattened expression is always normalized to a multiplication.
///
/// ```text
/// 10 * 2x / 5 / 2 / 4x -> x^2/2
/// ```
///
/// # How this is done
///
/// Consider the expression `x*2/y/(5/(x/y)) ~ (/ (/ (* x 2) y) (/ 5 (/ x y)))`. If they can be
/// unrolled to a series of terms `*x, *2, /y, /5, *x, /y`, all we have to do is combine like terms
/// and constants, and we're done. Turns out the trickier, and more interesting part, is exactly how
/// to unfold the expression. (There's a reason our example is mostly division.)
///
/// > Note that the flattening process does *not* play with commutativity; doing so would never
/// > correct.
///
/// First, let's assume we've unfolded all subexpressions. This means that all subexpressions will
/// be in multiplicative form; in particular, the example above becomes
///
/// ```text
/// x*2*(1/y)/(5*(1/(x*(1/y)))) ~ (/ (* (* x 2) (/ 1 y)) (* 5 (/ 1 (* x (/ 1 y)))))
/// ```
///
/// As we unfold subexpressions, we attach their operands to a double-ended list. The left side of
/// the list represents terms that should be multiplied in the final expression, and the right side
/// represents terms that should be divided. Initially this is just the LHS and RHS of the top
/// level expression. We also keep a registry of unfoldable terms and a variable to fold constants
/// into. In our example, this initially looks like
///
/// ```text
///           mul                       div                    registry       const
/// ----------------------||----------------------------
/// | (* (* x 2) (/ 1 y)) || (* 5 (/ 1 (* x (/ 1 y)))) |           ∅            1
/// ----------------------||----------------------------
///                       ^^-- pivot
/// ```
///
/// Now we go down the list and unfold any binary expression we see, or add fully-unfolded terms to
/// the registry.
///
/// We always handle the "multiplication" side first.
///
/// ## Multiplication side
///
/// This part is pretty straightforward. When we see a multiplication, we just add both operand to
/// the front of the list. Unfolding `(* (* x 2) (/ 1 y))`, we get
///
/// ```text
///           mul                       div                  registry       const
/// --------------------||----------------------------
/// | (* x 2) | (/ 1 y) || (* 5 (/ 1 (* x (/ 1 y)))) |           ∅            1
/// --------------------||----------------------------
/// ```
///
/// Which unfolds to
///
/// ```text
///           mul                   div                    registry       const
/// ------------------||----------------------------
/// | x | 2 | (/ 1 y) || (* 5 (/ 1 (* x (/ 1 y)))) |           ∅            1
/// ------------------||----------------------------
/// ```
///
/// In the next two steps see an unfoldable and a constant, which we add to the registry an fold
/// accordingly.
///
/// ```text
///    mul                  div                    registry       const
/// ----------||----------------------------
/// | (/ 1 y) || (* 5 (/ 1 (* x (/ 1 y)))) |        {x: 1}          2
/// ----------||----------------------------            ^-- mul: +1, div: -1 for this field
/// ```
///
/// When we see a division, we add the first operand to the multiplication side and the second
/// operand to the division side.
///
/// ```text
///  mul                div                    registry       const
/// ----||--------------------------------
/// | 1 || (* 5 (/ 1 (* x (/ 1 y)))) | y |      {x: 1}          2
/// ----||--------------------------------
/// ```
///
/// Folding the last term, we only get the division side remaining.
///
/// ```text
/// m                div                    registry       const
/// -||--------------------------------
/// ||| (* 5 (/ 1 (* x (/ 1 y)))) | y |      {x: 1}          2
/// -||--------------------------------
/// ```
///
/// ## Division side
///
/// This part is a bit trickier because we need to handle nested divisions, which may be equivalent
/// to multiplications on the top level. Maybe it's already clear how to do this; if not, we'll get
/// to it in a bit.
///
/// First, let's unfold the first multiplication on the division side by adding both operands to
/// the division side.
///
/// > To understand why this work, observe that `1 / (2 * 3)` is equivalent to `(1 / 2) / 3`.
///
/// ```text
/// m                div                  registry       const
/// -||------------------------------
/// ||| y | 5 | (/ 1 (* x (/ 1 y))) |      {x: 1}          2
/// -||------------------------------
/// ```
///
/// The next two terms are added to the registry and constant-folded, respectively.
///
/// ```text
/// m           div                  registry       const
/// -||----------------------
/// ||| (/ 1 (* x (/ 1 y))) |      {x: 1, y: -1}     2/5
/// -||----------------------
/// ```
///
/// Now we see a division `A` on the division side. This is the same thing as multiplying the
/// reciprocal of `A` on the top level!
///
/// > Let's break down a simpler example. Observe that `1 / (2 / 3)` is equivalent to `3 / 2`. The
/// > flattening list for `1 / (2 / 3)` after the folding of `1` looks like
/// >
/// > ```text
/// > m     div       const
/// > -||----------
/// > ||| (/ 2 3) |     1
/// > -||----------
/// > ```
/// >
/// > Now we simply add the reciprocal of the division expression to the multiplication side.
/// >
/// > ```text
/// >     mul     d   const
/// > ----------||-
/// > | (/ 3 2) |||     1
/// > ----------||-
/// > ```
/// >
/// > And we already know this gets unfolded as
/// >
/// > ```text
/// >  mul  div   const
/// > ----||----
/// > | 3 || 2 |    1
/// > ----||----
/// > ```
/// >
/// > So we can skip adding the entire division to the multiplication side, instead adding the
/// > operands where appropriate. The rest of the constant folding follows trivially.
///
/// Back to the original example, whose current state is
///
/// ```text
/// m           div                  registry       const
/// -||----------------------
/// ||| (/ 1 (* x (/ 1 y))) |      {x: 1, y: -1}     2/5
/// -||----------------------
/// ```
///
/// Let's apply our "division in division" algorithm: the left operand gets divided, and the right
/// operand gets multiplied.
///
/// ```text
///       mul         div         registry       const
/// ----------------||----
/// | (* x (/ 1 y)) || 1 |      {x: 1, y: -1}     2/5
/// ----------------||----
/// ```
///
/// Now we unfold the new expressions on the multiplication side.
///
/// ```text
///      mul        div         registry       const
/// --------------||----
/// | x | (/ 1 y) || 1 |      {x: 1, y: -1}     2/5
/// --------------||----
/// ```
///
/// Two steps this time:
///
/// ```text
///  mul    div           registry       const
/// ----||--------
/// | 1 || 1 | y |      {x: 2, y: -1}     2/5
/// ----||--------
/// ```
///
/// Three steps this time:
///
/// ```text
/// m  d        registry       const
/// -||-
/// ||||      {x: 2, y: -2}     2/5
/// -||-
/// ```
///
/// And now, all that needs to be done is to construct the flattened expression `2/5 * x^2 / y^-2`.
fn flatten_mul_or_div(o_lhs: RcExpr, o_rhs: RcExpr, is_div: bool) -> RcExpr {
    let o_span = o_lhs.span.to(o_rhs.span);
    let lhs = flatten_expr(o_lhs);
    let rhs = flatten_expr(o_rhs);

    let mut coeff = Rational::from(1);
    // Term -> # of times it is multiplied. Negative values are equivalent to division.
    let mut terms = BTreeMap::<RcExpr, Rational>::new();

    let mut args = VecDeque::with_capacity(2);
    args.push_back(lhs);
    args.push_back(rhs);

    // If this is not a division, the first two args are both on the mul side.
    let mut args_before_div = if is_div { 1 } else { 2 };

    while let Some(arg) = args.pop_front() {
        let div_side = args_before_div <= 0;
        args_before_div -= 1;

        let arg = unwrap_expr(arg.clone());

        match arg.as_ref() {
            Expr::Const(konst) => {
                if div_side {
                    coeff /= konst;
                } else {
                    coeff *= konst;
                }
            }
            Expr::BinaryExpr(BinaryExpr { op, lhs, rhs })
                if op == &BinaryOperator::Mult || op == &BinaryOperator::Div =>
            {
                if div_side {
                    if op == &BinaryOperator::Mult {
                        // 1 / (2 * 3) -> 1 / 2 / 3; add both operands to the div side.
                        args.push_back(lhs.clone());
                        args.push_back(rhs.clone());
                    } else {
                        // 1 / (2 / 3) -> 3 / 2; here we multiply by the reciprocal, so 3 goes on
                        // the mul side and 2 goes on the div side.
                        args.push_front(rhs.clone());
                        args_before_div = 1;
                        args.push_back(lhs.clone());
                    }
                } else {
                    // mul side
                    if op == &BinaryOperator::Mult {
                        // 1 * (2 * 3) -> 1 * 2 * 3
                        args.push_front(lhs.clone());
                        args.push_front(rhs.clone());
                        args_before_div += 2;
                    } else {
                        // 1 * (2 / 3) -> 1 * 2 / 3; add 2 to the mul side and 3 to the div side.
                        args.push_front(lhs.clone());
                        args_before_div += 1;
                        args.push_back(rhs.clone());
                    }
                }
            }
            _ => {
                // Otherwise the arg is something we cannot further decompose in this context
                // (e.g. a variable or an exponentiation), so add it as a term.
                // TODO: see if we can handle other things more granularly
                let entry = terms
                    .entry(arg.clone())
                    .or_insert_with(|| Rational::from(0));
                if div_side {
                    *entry -= 1;
                } else {
                    *entry += 1;
                }
            }
        }
    }

    let mut new_args: Vec<RcExpr> = Vec::with_capacity(1 + terms.len());
    if coeff != 1 {
        new_args.push(rc_expr!(Expr::Const(coeff), o_span));
    }
    for (term, coeff) in terms {
        if coeff == 0. {
            // The happiest path :)
            continue;
        } else if coeff == 1 {
            // 1 * x
            new_args.push(term.clone());
        } else if coeff == -1 {
            // -1 * x ~ 1/x
            let reciprocal = BinaryExpr::div(
                rc_expr!(Expr::Const(Rational::from(1)), o_span),
                term.clone(),
            );
            new_args.push(rc_expr!(Expr::BinaryExpr(reciprocal), o_span));
        } else {
            let exponentiation =
                BinaryExpr::exp(term.clone(), rc_expr!(Expr::Const(coeff), o_span));
            new_args.push(rc_expr!(Expr::BinaryExpr(exponentiation), o_span));
        }
    }

    match new_args.len() {
        0 => rc_expr!(Expr::Const(Rational::from(1)), o_span),
        1 => new_args.remove(0),
        _ => unflatten_binary_expr(&new_args, BinaryOperator::Mult, UnflattenStrategy::Left),
    }
}

/// Unwraps an expression in parentheses/brackets, or returns the original expression if it cannot
/// be unwrapped.
fn unwrap_expr(arg: RcExpr) -> RcExpr {
    match arg.as_ref() {
        Expr::Parend(inner) | Expr::Bracketed(inner) => inner.clone(),
        _ => arg,
    }
}

#[cfg(test)]
mod tests {
    use super::flatten_expr;
    use crate::parse_expr;
    use crate::utils::normalize;
    use crate::{Emit, EmitConfig, ProgramContext};

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
        // TODO: (- (- c)) -> c
        "a - (b - c) -> (+ (+ a (- b)) (- (- c)))",
        "10 * 2x / 5 / 2 / 4x -> (* 0.5 (^ x 2))",
        "x * 2 / y / (5 / (x / y)) -> (* (* 0.4 (^ x 2)) (^ y -2))",
        "x * x -> (^ x 2)",
        "x / x -> 1",
        // TODO: currently (* (^ x 2) (^ x -2)). This can be fixed by properly handling exponents
        // when flattening mul/div.
        // "x * x / x * x / x / x -> 1",
        "x / x * x / x * x / x -> 1",
    ];

    #[test]
    fn flatten_cases() {
        for case in CASES {
            let mut split = case.split(" -> ");
            let lhs = split.next().unwrap();
            let expr = parse_expr!(lhs);
            let expected_flattened = split.next().unwrap();

            let ctxt = ProgramContext::test();
            let flattened =
                normalize(flatten_expr(expr)).emit_s_expression(&EmitConfig::new(&ctxt, &[]));

            assert_eq!(flattened, expected_flattened);
        }
    }
}
