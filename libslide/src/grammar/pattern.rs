pub use super::*;

#[derive(Clone)]
pub enum ExprPat {
    Const(f64),
    /// Pattern matching a variable
    VarPat(String),
    /// Pattern matching a constant
    ConstPat(String),
    /// Pattern matching any expression
    AnyPat(String),
    BinaryExpr(BinaryExpr<Self>),
    UnaryExpr(UnaryExpr<Self>),
    Parend(Box<Self>),
    Braced(Box<Self>),
}

impl Grammar for ExprPat {}
impl Expression for ExprPat {}

// TODO: We can't derive this because `f64` doesn't implement `Eq`.
// This should be fixed by moving to a arbitrary-precision numeric type.
impl Eq for ExprPat {}
impl PartialEq for ExprPat {
    fn eq(&self, other: &ExprPat) -> bool {
        use ExprPat::*;
        match (self, other) {
            (Const(x), Const(y)) => (x - y).abs() < std::f64::EPSILON,
            (VarPat(x), VarPat(y)) => x == y,
            (ConstPat(x), ConstPat(y)) => x == y,
            (AnyPat(x), AnyPat(y)) => x == y,
            (BinaryExpr(x), BinaryExpr(y)) => x == y,
            (UnaryExpr(x), UnaryExpr(y)) => x == y,
            (Parend(x), Parend(y)) => x == y,
            (Braced(x), Braced(y)) => x == y,
            _ => false,
        }
    }
}

// TODO: We can do better than hashing to a string as well, but we'll save that til we have an
// arbitrary-precision numeric type.
impl core::hash::Hash for ExprPat {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write(self.to_string().as_bytes())
    }
}

impl From<BinaryExpr<Self>> for ExprPat {
    fn from(binary_expr: BinaryExpr<Self>) -> Self {
        Self::BinaryExpr(binary_expr)
    }
}

impl From<UnaryExpr<Self>> for ExprPat {
    fn from(unary_expr: UnaryExpr<Self>) -> Self {
        Self::UnaryExpr(unary_expr)
    }
}

impl fmt::Display for ExprPat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ExprPat::*;
        write!(
            f,
            "{}",
            match self {
                Const(num) => num.to_string(),
                VarPat(var) | ConstPat(var) | AnyPat(var) => var.to_string(),
                BinaryExpr(binary_expr) => binary_expr.to_string(),
                UnaryExpr(unary_expr) => unary_expr.to_string(),
                Parend(expr) => format!("({})", expr.to_string()),
                Braced(expr) => format!("[{}]", expr.to_string()),
            }
        )
    }
}
