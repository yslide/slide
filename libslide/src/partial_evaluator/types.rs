use crate::parser::types::*;
use core::fmt;

pub enum PEResult {
    Evaluated(f64),
    Unevaluated(Box<Expr>),
}

impl PEResult {
    pub fn from_expr(expr: Expr) -> Self {
        Self::Unevaluated(Box::new(expr))
    }

    pub fn fold_binary(lhs: PEResult, rhs: PEResult, op: BinaryOperator) -> PEResult {
        use PEResult::*;
        let binary_fn = binary_operator_table(&op);
        match (lhs, rhs) {
            (Evaluated(x), Evaluated(y)) => Evaluated(binary_fn(x, y)),
            (Unevaluated(lhs), Unevaluated(rhs)) => {
                PEResult::from_expr(Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }))
            }

            (Evaluated(x), y) => Self::fold_binary(PEResult::from_expr(Expr::Float(x)), y, op),
            (x, Evaluated(y)) => Self::fold_binary(x, PEResult::from_expr(Expr::Float(y)), op),
        }
    }

    pub fn fold_unary(rhs: PEResult, op: UnaryOperator) -> PEResult {
        use PEResult::*;
        let unary_fn = unary_operator_table(&op);
        match rhs {
            Evaluated(x) => Evaluated(unary_fn(x)),
            Unevaluated(rhs) => PEResult::from_expr(Expr::UnaryExpr(UnaryExpr { op, rhs })),
        }
    }
}

fn binary_operator_table(op: &BinaryOperator) -> Box<dyn Fn(f64, f64) -> f64 + 'static> {
    match op {
        BinaryOperator::Plus => Box::new(|x, y| x + y),
        BinaryOperator::Minus => Box::new(|x, y| x - y),
        BinaryOperator::Mult => Box::new(|x, y| x * y),
        BinaryOperator::Div => Box::new(|x, y| x / y),
        BinaryOperator::Mod => Box::new(|x, y| x % y),
        BinaryOperator::Exp => Box::new(|x, y| x.powf(y)),
    }
}

fn unary_operator_table(op: &UnaryOperator) -> Box<dyn Fn(f64) -> f64 + 'static> {
    match op {
        UnaryOperator::SignPositive => Box::new(|x| x),
        UnaryOperator::SignNegative => Box::new(|x| -x),
    }
}

impl fmt::Display for PEResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PEResult::*;
        write!(
            f,
            "{}",
            match self {
                Evaluated(f) => f.to_string(),
                Unevaluated(b) => b.to_string(),
            }
        )
    }
}
