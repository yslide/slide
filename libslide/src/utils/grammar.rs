use crate::grammar::*;
use crate::Span;

use std::collections::{HashSet, VecDeque};

pub fn get_symmetric_expressions(expr: RcExpr) -> Vec<RcExpr> {
    match expr.as_ref() {
        Expr::BinaryExpr(BinaryExpr { op, .. }) => match op {
            BinaryOperator::Plus | BinaryOperator::Mult => {
                let args = get_flattened_binary_args(expr.clone(), *op);
                let mut sym_args = Vec::with_capacity(args.len());
                let mut result = Vec::with_capacity((args.len() - 1) * 2);
                for i in 0..args.len() {
                    sym_args.push(args[i].clone());
                    for arg in args.iter().take(i) {
                        sym_args.push(arg.clone());
                    }
                    for arg in args.iter().skip(i + 1) {
                        sym_args.push(arg.clone());
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

pub fn get_flattened_binary_args(expr: RcExpr, parent_op: BinaryOperator) -> Vec<RcExpr> {
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
            let flattened_left = get_flattened_binary_args(child.lhs.clone(), child.op);
            let flattened_right = get_flattened_binary_args(child.rhs.clone(), child.op);
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
            let flattened_left = get_flattened_binary_args(child.lhs.clone(), child.op);
            let flattened_right = get_flattened_binary_args(child.rhs.clone(), child.op);
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
            let flattened_left = get_flattened_binary_args(child.lhs.clone(), child.op);
            let flattened_right = get_flattened_binary_args(child.rhs.clone(), child.op);
            insert_front!(args, flattened_left);
            insert_back!(args, flattened_right.into_iter().map(negate));
            args.into_iter().collect()
        }
        _ => vec![expr],
    }
}

fn negate(expr: RcExpr) -> RcExpr {
    let span = expr.span;
    match expr.as_ref() {
        // #a -> -#a
        Expr::Const(f) => rc_expr!(Expr::Const(-f), span),

        // $a -> -$a
        Expr::Var(_) => rc_expr!(
            Expr::UnaryExpr(UnaryExpr {
                op: UnaryOperator::SignNegative,
                rhs: expr,
            }),
            span
        ),

        // +_a => -_a
        Expr::UnaryExpr(UnaryExpr {
            op: UnaryOperator::SignPositive,
            rhs,
        }) => rc_expr!(
            Expr::UnaryExpr(UnaryExpr {
                op: UnaryOperator::SignPositive,
                rhs: rhs.clone(),
            }),
            span
        ),

        // -_a => _a
        Expr::UnaryExpr(UnaryExpr {
            op: UnaryOperator::SignNegative,
            rhs,
        }) => rhs.clone(),

        // _a <op> _b => -(_a <op> _b)
        // TODO: We could expand factorable expressions further:
        //       -(_a + _b) = -_a + -_b
        //       -(_a - _b) = -_a - -_b
        Expr::BinaryExpr(_) => rc_expr!(
            Expr::UnaryExpr(UnaryExpr {
                op: UnaryOperator::SignNegative,
                rhs: expr,
            }),
            span
        ),

        Expr::Parend(expr) | Expr::Bracketed(expr) => negate(expr.clone()),
    }
}

pub enum UnflattenStrategy {
    Left,  // (+ (+ 1 2) 3)
    Right, // (+ 1 (+ 2 3))
}

pub fn unflatten_binary_expr<E>(args: &[E], op: BinaryOperator, strategy: UnflattenStrategy) -> E
where
    E: RcExpression,
{
    fn _left<E: RcExpression>(args: &[E], op: BinaryOperator) -> E {
        let mut args = args.iter();
        let mut lhs = args.next().unwrap().clone();
        for rhs in args {
            let span = lhs.span().to(rhs.span());
            lhs = E::binary(
                BinaryExpr {
                    op,
                    lhs,
                    rhs: rhs.clone(),
                },
                span,
            )
        }
        lhs
    }

    fn _right<E: RcExpression>(args: &[E], op: BinaryOperator) -> E {
        let mut args = args.iter().rev();
        let mut rhs = args.next().unwrap().clone();
        for lhs in args {
            let span = lhs.span().to(rhs.span());
            rhs = E::binary(
                BinaryExpr {
                    op,
                    lhs: lhs.clone(),
                    rhs,
                },
                span,
            )
        }
        rhs
    }

    match strategy {
        UnflattenStrategy::Left => _left(args, op),
        UnflattenStrategy::Right => _right(args, op),
    }
}

/// Returns all unique pattern names in a pattern expression.
pub fn unique_pats(expr: &RcExprPat) -> HashSet<&str> {
    let mut collector = PatCollector::default();
    collector.visit(expr);
    collector.pats
}

// TODO: Put collectors like these in a "collectors" module.
#[derive(Default)]
struct PatCollector<'a> {
    pats: HashSet<&'a str>,
}

impl<'a> ExprPatVisitor<'a> for PatCollector<'a> {
    fn visit_var_pat(&mut self, var_pat: &'a str, _span: Span) {
        self.pats.insert(var_pat);
    }
    fn visit_const_pat(&mut self, const_pat: &'a str, _span: Span) {
        self.pats.insert(const_pat);
    }
    fn visit_any_pat(&mut self, any_pat: &'a str, _span: Span) {
        self.pats.insert(any_pat);
    }
}

pub fn normalize(expr: RcExpr) -> RcExpr {
    match expr.as_ref() {
        Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }) => {
            let partially_normalized = Expr::BinaryExpr(BinaryExpr {
                op: *op,
                lhs: normalize(lhs.clone()),
                rhs: normalize(rhs.clone()),
            });
            let mut flattened_args =
                get_flattened_binary_args(rc_expr!(partially_normalized, expr.span), *op);
            flattened_args.sort();
            unflatten_binary_expr(&flattened_args, *op, UnflattenStrategy::Left)
        }
        Expr::UnaryExpr(UnaryExpr { op, rhs }) => rc_expr!(
            Expr::UnaryExpr(UnaryExpr {
                op: *op,
                rhs: normalize(rhs.clone()),
            }),
            expr.span
        ),
        Expr::Parend(expr) => {
            let inner = normalize(expr.clone());
            let span = inner.span;
            rc_expr!(Expr::Parend(inner), span)
        }
        Expr::Bracketed(expr) => {
            let inner = normalize(expr.clone());
            let span = inner.span;
            rc_expr!(Expr::Bracketed(inner), span)
        }

        _ => expr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_expr, parse_expression_pattern, scan};

    #[test]
    fn get_symmetric_expressions_add() {
        let parsed = parse_expr!("1 - 2 + 3 + (x * 4) + 5");
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
        let parsed = parse_expr!("1 ^ 2 * 3 * (x - 4) * 5");
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
        let expr = parse_expr!("1 - 2 + 3 + 4 * 5 + 6");
        let args: Vec<String> = get_flattened_binary_args(expr, BinaryOperator::Plus)
            .into_iter()
            .map(|e| e.to_string())
            .collect();

        assert_eq!(args, vec!["1", "-2", "3", "4 * 5", "6"]);
    }

    #[test]
    fn unflatten_binary_expr_right() {
        let args: Vec<RcExpr> = vec!["1", "2", "3", "4"]
            .into_iter()
            .map(|s| parse_expr!(s))
            .collect();
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
        let args: Vec<RcExpr> = vec!["1", "2", "3", "4"]
            .into_iter()
            .map(|s| parse_expr!(s))
            .collect();
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

        let mut pats: Vec<_> = pats.into_iter().collect();
        pats.sort_by(|a, b| a.as_bytes()[1].cmp(&b.as_bytes()[1]));

        assert_eq!(pats, vec!["$a", "_b", "#c", "$d"]);
    }
}
