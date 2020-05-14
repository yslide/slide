use crate::grammar::*;

use std::collections::VecDeque;
use std::rc::Rc;

pub fn get_symmetric_expressions(expr: Rc<Expr>) -> Vec<Rc<Expr>> {
    match expr.as_ref() {
        Expr::BinaryExpr(BinaryExpr { op, .. }) => match op {
            BinaryOperator::Plus | BinaryOperator::Mult => {
                let args = get_flattened_binary_args(Rc::clone(&expr), *op);
                let mut sym_args = Vec::with_capacity(args.len());
                let mut result = Vec::with_capacity((args.len() - 1) * 2);
                for i in 0..args.len() {
                    sym_args.push(Rc::clone(&args[i]));
                    for arg in args.iter().take(i) {
                        sym_args.push(Rc::clone(arg));
                    }
                    for arg in args.iter().skip(i + 1) {
                        sym_args.push(Rc::clone(arg));
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

fn get_flattened_binary_args(expr: Rc<Expr>, op: BinaryOperator) -> VecDeque<Rc<Expr>> {
    match expr.as_ref() {
        Expr::BinaryExpr(child) if child.op == op => {
            let mut args = VecDeque::with_capacity(2);
            for arg in get_flattened_binary_args(Rc::clone(&child.lhs), op)
                .into_iter()
                .rev()
            {
                args.push_front(arg);
            }
            args.append(&mut get_flattened_binary_args(Rc::clone(&child.rhs), op));
            args
        }
        _ => vec![expr].into_iter().collect(),
    }
}

enum UnflattenStrategy {
    Left,  // (+ (+ 1 2) 3)
    Right, // (+ 1 (+ 2 3))
}

fn unflatten_binary_expr<E>(
    args: &[Rc<E>],
    op: BinaryOperator,
    strategy: UnflattenStrategy,
) -> Rc<E>
where
    E: Expression,
{
    fn _left<E: Expression>(args: &[Rc<E>], op: BinaryOperator) -> Rc<E> {
        let mut args = args.iter();
        let mut lhs = Rc::clone(args.next().unwrap());
        for rhs in args {
            lhs = Rc::new(
                BinaryExpr {
                    op,
                    lhs,
                    rhs: Rc::clone(rhs),
                }
                .into(),
            );
        }
        lhs
    }

    fn _right<E: Expression>(args: &[Rc<E>], op: BinaryOperator) -> Rc<E> {
        let mut args = args.iter().rev();
        let mut rhs = Rc::clone(args.next().unwrap());
        for lhs in args {
            rhs = Rc::new(
                BinaryExpr {
                    op,
                    lhs: Rc::clone(lhs),
                    rhs,
                }
                .into(),
            );
        }
        rhs
    }

    match strategy {
        UnflattenStrategy::Left => _left(args, op),
        UnflattenStrategy::Right => _right(args, op),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_expression, scan};

    fn parse<T: Into<String>>(s: T) -> Rc<Expr> {
        let toks = scan(s);
        match parse_expression(toks) {
            (Stmt::Expr(expr), _) => Rc::new(expr),
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
                "1 - 2 + 3 + (x * 4) + 5",
                "1 - 2 + 3 + (x * 4) + 5",
                "3 + 1 - 2 + (x * 4) + 5",
                "3 + 1 - 2 + (x * 4) + 5",
                "(x * 4) + 1 - 2 + 3 + 5",
                "(x * 4) + 1 - 2 + 3 + 5",
                "5 + 1 - 2 + 3 + (x * 4)",
                "5 + 1 - 2 + 3 + (x * 4)",
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

        assert_eq!(args, vec!["1 - 2", "3", "4 * 5", "6"]);
    }

    #[test]
    fn unflatten_binary_expr_right() {
        let args: Vec<Rc<Expr>> = vec!["1", "2", "3", "4"].into_iter().map(parse).collect();
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
        let args: Vec<Rc<Expr>> = vec!["1", "2", "3", "4"].into_iter().map(parse).collect();
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
}
