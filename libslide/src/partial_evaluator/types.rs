use crate::grammar::*;
use crate::printer::Print;
use core::fmt;

pub enum PEResult {
    Evaluated(f64),
    Unevaluated(Box<Expr>),
}

impl Print for PEResult {
    fn print(self) -> String {
        match self {
            Self::Evaluated(x) => x.to_string(),
            Self::Unevaluated(expr) => expr.print(),
        }
    }
}

impl From<Expr> for PEResult {
    fn from(unevaluated: Expr) -> Self {
        Self::Unevaluated(Box::new(unevaluated))
    }
}

impl From<f64> for PEResult {
    fn from(evaluated: f64) -> Self {
        Self::Evaluated(evaluated)
    }
}

impl PEResult {
    pub fn fold_binary<T, U>(lhs: T, rhs: U, op: BinaryOperator) -> PEResult
    where
        T: Into<PEResult>,
        U: Into<PEResult>,
    {
        use PEResult::*;
        let binary_fn = binary_operator_table(&op);
        match (lhs.into(), rhs.into()) {
            (Evaluated(x), Evaluated(y)) => binary_fn(x, y).into(),
            (Unevaluated(lhs), Unevaluated(rhs)) => {
                Expr::BinaryExpr(BinaryExpr { op, lhs, rhs }).into()
            }

            (Evaluated(x), y) => Self::fold_binary(Expr::Float(x), y, op),
            (x, Evaluated(y)) => Self::fold_binary(x, Expr::Float(y), op),
        }
    }

    pub fn fold_unary(rhs: PEResult, op: UnaryOperator) -> PEResult {
        use PEResult::*;
        let unary_fn = unary_operator_table(&op);
        match rhs {
            Evaluated(x) => unary_fn(x).into(),
            Unevaluated(rhs) => Expr::UnaryExpr(UnaryExpr { op, rhs }).into(),
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
