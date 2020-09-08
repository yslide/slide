use crate::grammar::*;

use std::collections::{HashSet, VecDeque};

pub fn get_symmetric_expressions(expr: InternedExpr) -> Vec<InternedExpr> {
    match expr.as_ref() {
        Expr::BinaryExpr(BinaryExpr { op, .. }) => match op {
            BinaryOperator::Plus | BinaryOperator::Mult => {
                let args = get_flattened_binary_args(expr, *op);
                let mut sym_args = Vec::with_capacity(args.len());
                let mut result = Vec::with_capacity((args.len() - 1) * 2);
                for i in 0..args.len() {
                    sym_args.push(args[i]);
                    for arg in args.iter().take(i) {
                        sym_args.push(*arg);
                    }
                    for arg in args.iter().skip(i + 1) {
                        sym_args.push(*arg);
                    }
                    result.push(unflatten_binary_expr(
                        &sym_args,
                        *op,
                        UnflattenStrategy::Left,
                    ));
                    result.push(unflatten_binary_expr(
                        &sym_args,
                        *op,
                        UnflattenStrategy::Right,
                    ));
                    sym_args.clear();
                }
                result
            }
            _ => vec![expr],
        },
        _ => vec![expr],
    }
}

macro_rules! insert_front {
    ($container:ident, $items:expr) => {
        for item in $items.into_iter().rev() {
            $container.push_front(item);
        }
    };
}

macro_rules! insert_back {
    ($container:ident, $items:expr) => {
        $container.extend($items);
    };
}

pub fn get_flattened_binary_args(
    expr: InternedExpr,
    parent_op: BinaryOperator,
) -> Vec<InternedExpr> {
    match expr.as_ref() {
        Expr::BinaryExpr(
            child
            @
            BinaryExpr {
                op: BinaryOperator::Plus,
                ..
            },
        ) if parent_op == BinaryOperator::Plus => {
            // ... + ((1 + 2) + (3 + 4)) => ... + 1, 2, 3, 4
            let mut args = VecDeque::with_capacity(2);
            let flattened_left = get_flattened_binary_args(child.lhs, child.op);
            let flattened_right = get_flattened_binary_args(child.rhs, child.op);
            insert_front!(args, flattened_left);
            insert_back!(args, flattened_right);
            args.into_iter().collect()
        }

        Expr::BinaryExpr(
            child
            @
            BinaryExpr {
                op: BinaryOperator::Mult,
                ..
            },
        ) if parent_op == BinaryOperator::Mult => {
            // ... * ((1 * 2) * (3 * 4)) => ... * 1, 2, 3, 4
            let mut args = VecDeque::with_capacity(2);
            let flattened_left = get_flattened_binary_args(child.lhs, child.op);
            let flattened_right = get_flattened_binary_args(child.rhs, child.op);
            insert_front!(args, flattened_left);
            insert_back!(args, flattened_right);
            args.into_iter().collect()
        }

        Expr::BinaryExpr(
            child
            @
            BinaryExpr {
                op: BinaryOperator::Minus,
                ..
            },
        ) if parent_op == BinaryOperator::Plus => {
            // ... + (2 - 3) => ... + 2, -3
            // ... + ((1 + 2) - (2 + 3)) => ... + 1, 2, -2, -3
            let mut args = VecDeque::with_capacity(2);
            let flattened_left = get_flattened_binary_args(child.lhs, child.op);
            let flattened_right = get_flattened_binary_args(child.rhs, child.op);
            insert_front!(args, flattened_left);
            insert_back!(args, flattened_right.into_iter().map(negate));
            args.into_iter().collect()
        }
        _ => vec![expr],
    }
}

fn negate(expr: InternedExpr) -> InternedExpr {
    match expr.as_ref() {
        // #a -> -#a
        Expr::Const(f) => intern_expr!(Expr::Const(-f), expr.span),

        // $a -> -$a
        Expr::Var(_) => intern_expr!(
            Expr::UnaryExpr(UnaryExpr {
                op: UnaryOperator::SignNegative,
                rhs: expr,
            }),
            expr.span
        ),

        // +_a => -_a
        Expr::UnaryExpr(UnaryExpr {
            op: UnaryOperator::SignPositive,
            rhs,
        }) => intern_expr!(
            Expr::UnaryExpr(UnaryExpr {
                op: UnaryOperator::SignPositive,
                rhs: *rhs,
            }),
            expr.span
        ),

        // -_a => _a
        Expr::UnaryExpr(UnaryExpr {
            op: UnaryOperator::SignNegative,
            rhs,
        }) => *rhs,

        // _a <op> _b => -(_a <op> _b)
        // TODO: We could expand factorable expressions further:
        //       -(_a + _b) = -_a + -_b
        //       -(_a - _b) = -_a - -_b
        Expr::BinaryExpr(_) => intern_expr!(
            Expr::UnaryExpr(UnaryExpr {
                op: UnaryOperator::SignNegative,
                rhs: expr,
            }),
            expr.span
        ),

        Expr::Parend(expr) | Expr::Bracketed(expr) => negate(*expr),
    }
}

pub enum UnflattenStrategy {
    Left,  // (+ (+ 1 2) 3)
    Right, // (+ 1 (+ 2 3))
}

pub fn unflatten_binary_expr<E>(args: &[E], op: BinaryOperator, strategy: UnflattenStrategy) -> E
where
    E: InternedExpression,
{
    fn _left<E: InternedExpression>(args: &[E], op: BinaryOperator) -> E {
        let mut args = args.iter();
        let mut lhs = *args.next().unwrap();
        for rhs in args {
            lhs = E::binary(
                BinaryExpr { op, lhs, rhs: *rhs },
                /* TODO: propagate span */ crate::DUMMY_SP,
            )
        }
        lhs
    }

    fn _right<E: InternedExpression>(args: &[E], op: BinaryOperator) -> E {
        let mut args = args.iter().rev();
        let mut rhs = *args.next().unwrap();
        for lhs in args {
            rhs = E::binary(
                BinaryExpr { op, lhs: *lhs, rhs },
                /* TODO: propagate span */ crate::DUMMY_SP,
            )
        }
        rhs
    }

    match strategy {
        UnflattenStrategy::Left => _left(args, op),
        UnflattenStrategy::Right => _right(args, op),
    }
}

/// Returns all unique patterns in a pattern expression.
pub fn unique_pats<'a>(expr: &'a InternedExprPat) -> HashSet<&'a InternedExprPat> {
    fn unique_pats<'a>(expr: &'a InternedExprPat, set: &mut HashSet<&'a InternedExprPat>) {
        match expr.as_ref() {
            ExprPat::VarPat(_) | ExprPat::ConstPat(_) | ExprPat::AnyPat(_) => {
                set.insert(expr);
            }
            ExprPat::BinaryExpr(BinaryExpr { lhs, rhs, .. }) => {
                unique_pats(&lhs, set);
                unique_pats(&rhs, set);
            }
            ExprPat::UnaryExpr(UnaryExpr { rhs, .. }) => {
                unique_pats(&rhs, set);
            }
            ExprPat::Parend(e) | ExprPat::Bracketed(e) => {
                unique_pats(&e, set);
            }
            ExprPat::Const(_) => {}
        }
    }

    let mut hs = HashSet::new();
    unique_pats(expr, &mut hs);
    hs
}

pub fn normalize(expr: InternedExpr) -> InternedExpr {
    match expr.as_ref() {
        Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) => {
            let partially_normalized = Expr::BinaryExpr(BinaryExpr {
                op: *op,
                lhs: normalize(*lhs),
                rhs: normalize(*rhs),
            });
            let mut flattened_args =
                get_flattened_binary_args(intern_expr!(partially_normalized, expr.span), *op);
            flattened_args.sort();
            unflatten_binary_expr(&flattened_args, *op, UnflattenStrategy::Left)
        }
        Expr::UnaryExpr(UnaryExpr { op, rhs }) => intern_expr!(
            Expr::UnaryExpr(UnaryExpr {
                op: *op,
                rhs: normalize(*rhs),
            }),
            expr.span
        ),
        Expr::Parend(expr) => {
            let inner = normalize(*expr);
            intern_expr!(Expr::Parend(inner), inner.span)
        }
        Expr::Bracketed(expr) => {
            let inner = normalize(*expr);
            intern_expr!(Expr::Bracketed(inner), inner.span)
        }

        _ => expr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_expression, parse_expression_pattern, scan};

    fn parse(s: &'static str) -> InternedExpr {
        let toks = scan(s).tokens;
        match parse_expression(toks) {
            (Stmt::Expr(expr), _) => expr,
            _ => unreachable!(),
        }
    }

    #[test]
    fn get_symmetric_expressions_add() {
        let parsed = parse("1 - 2 + 3 + (x * 4) + 5");
        let exprs: Vec<String> = get_symmetric_expressions(parsed)
            .into_iter()
            .map(|e| e.to_string())
            .collect();

        assert_eq!(
            exprs,
            vec![
                // Same versions with left and right associativity
                "1 + -2 + 3 + (x * 4) + 5",
                "1 + -2 + 3 + (x * 4) + 5",
                "-2 + 1 + 3 + (x * 4) + 5",
                "-2 + 1 + 3 + (x * 4) + 5",
                "3 + 1 + -2 + (x * 4) + 5",
                "3 + 1 + -2 + (x * 4) + 5",
                "(x * 4) + 1 + -2 + 3 + 5",
                "(x * 4) + 1 + -2 + 3 + 5",
                "5 + 1 + -2 + 3 + (x * 4)",
                "5 + 1 + -2 + 3 + (x * 4)"
            ]
        );
    }

    #[test]
    fn get_symmetric_expressions_mult() {
        let parsed = parse("1 ^ 2 * 3 * (x - 4) * 5");
        let exprs: Vec<String> = get_symmetric_expressions(parsed)
            .into_iter()
            .map(|e| e.to_string())
            .collect();

        assert_eq!(
            exprs,
            vec![
                // Same versions with left and right associativity
                "1 ^ 2 * 3 * (x - 4) * 5",
                "1 ^ 2 * 3 * (x - 4) * 5",
                "3 * 1 ^ 2 * (x - 4) * 5",
                "3 * 1 ^ 2 * (x - 4) * 5",
                "(x - 4) * 1 ^ 2 * 3 * 5",
                "(x - 4) * 1 ^ 2 * 3 * 5",
                "5 * 1 ^ 2 * 3 * (x - 4)",
                "5 * 1 ^ 2 * 3 * (x - 4)",
            ]
        );
    }

    #[test]
    fn flattened_binary_args() {
        let expr = parse("1 - 2 + 3 + 4 * 5 + 6");
        let args: Vec<String> = get_flattened_binary_args(expr, BinaryOperator::Plus)
            .into_iter()
            .map(|e| e.to_string())
            .collect();

        assert_eq!(args, vec!["1", "-2", "3", "4 * 5", "6"]);
    }

    #[test]
    fn unflatten_binary_expr_right() {
        let args: Vec<InternedExpr> = vec!["1", "2", "3", "4"].into_iter().map(parse).collect();
        let expr = unflatten_binary_expr(&args, BinaryOperator::Plus, UnflattenStrategy::Right);

        match expr.as_ref() {
            Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) => {
                assert_eq!(*op, BinaryOperator::Plus);

                assert_eq!(lhs.to_string(), "1");
                match rhs.as_ref() {
                    Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) => {
                        assert_eq!(*op, BinaryOperator::Plus);

                        assert_eq!(lhs.to_string(), "2");
                        match rhs.as_ref() {
                            Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) => {
                                assert_eq!(*op, BinaryOperator::Plus);
                                assert_eq!(lhs.to_string(), "3");
                                assert_eq!(rhs.to_string(), "4");
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn unflatten_binary_expr_left() {
        let args: Vec<InternedExpr> = vec!["1", "2", "3", "4"].into_iter().map(parse).collect();
        let expr = unflatten_binary_expr(&args, BinaryOperator::Plus, UnflattenStrategy::Left);

        match expr.as_ref() {
            Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) => {
                assert_eq!(*op, BinaryOperator::Plus);

                match lhs.as_ref() {
                    Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) => {
                        assert_eq!(*op, BinaryOperator::Plus);

                        match lhs.as_ref() {
                            Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) => {
                                assert_eq!(*op, BinaryOperator::Plus);
                                assert_eq!(lhs.to_string(), "1");
                                assert_eq!(rhs.to_string(), "2");
                            }
                            _ => unreachable!(),
                        }
                        assert_eq!(rhs.to_string(), "3");
                    }
                    _ => unreachable!(),
                }
                assert_eq!(rhs.to_string(), "4");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn unique_pats() {
        let parsed = parse_expression_pattern(scan("$a + _b * (#c - [$d]) / $a").tokens).0;
        let pats = super::unique_pats(&parsed);

        let mut pats: Vec<_> = pats
            .iter()
            .map(|p| match p.as_ref() {
                ExprPat::VarPat(v) | ExprPat::ConstPat(v) | ExprPat::AnyPat(v) => v,
                _ => unreachable!(),
            })
            .collect();
        pats.sort_by(|a, b| a.as_bytes()[1].cmp(&b.as_bytes()[1]));

        assert_eq!(pats, vec!["$a", "_b", "#c", "$d"]);
    }
}
