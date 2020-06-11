pub use super::*;
use crate::scanner::FLOAT_PRECISION;

use rug::Float;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum ExprPat {
    Const(Float),
    /// Pattern matching a variable
    VarPat(String),
    /// Pattern matching a constant
    ConstPat(String),
    /// Pattern matching any expression
    AnyPat(String),
    BinaryExpr(BinaryExpr<Self>),
    UnaryExpr(UnaryExpr<Self>),
    Parend(Rc<Self>),
    Bracketed(Rc<Self>),
}

impl Grammar for ExprPat {
    fn s_form(&self) -> String {
        match self {
            Self::Const(konst) => konst.to_string(),
            Self::VarPat(pat) | Self::ConstPat(pat) | Self::AnyPat(pat) => pat.to_string(),
            Self::BinaryExpr(BinaryExpr { op, lhs, rhs }) => {
                format!("({} {} {})", op.to_string(), lhs.s_form(), rhs.s_form())
            }
            Self::UnaryExpr(UnaryExpr { op, rhs }) => {
                format!("({} {})", op.to_string(), rhs.s_form())
            }
            Self::Parend(inner) => format!("({})", inner.s_form()),
            Self::Bracketed(inner) => format!("[{}]", inner.s_form()),
        }
    }
}
impl Grammar for Rc<ExprPat> {
    fn s_form(&self) -> String {
        self.as_ref().s_form()
    }
}
impl Expression for ExprPat {
    #[inline]
    fn is_const(&self) -> bool {
        matches!(self, Self::Const(_))
    }
}

// TODO: We can't derive this because `f64` doesn't implement `Eq`.
// This should be fixed by moving to a arbitrary-precision numeric type.
impl Eq for ExprPat {}
impl PartialEq for ExprPat {
    fn eq(&self, other: &ExprPat) -> bool {
        use ExprPat::*;
        match (self, other) {
            (Const(x), Const(y)) => Float::with_val(FLOAT_PRECISION, x-y) < Float::with_val(FLOAT_PRECISION, std::f64::EPSILON),
            (VarPat(x), VarPat(y)) => x == y,
            (ConstPat(x), ConstPat(y)) => x == y,
            (AnyPat(x), AnyPat(y)) => x == y,
            (BinaryExpr(x), BinaryExpr(y)) => x == y,
            (UnaryExpr(x), UnaryExpr(y)) => x == y,
            (Parend(x), Parend(y)) => x == y,
            (Bracketed(x), Bracketed(y)) => x == y,
            _ => false,
        }
    }
}

// TODO: We can do better than hashing to a string as well, but we'll save that til we have an
// arbitrary-precision numeric type.
impl core::hash::Hash for ExprPat {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        use ExprPat::*;
        match self {
            // TODO: We can do better than hashing to a string as well, but we'll save that til we
            // have an arbitrary-precision numeric type.
            Const(f) => state.write(f.to_string().as_bytes()),
            VarPat(v) => v.hash(state),
            ConstPat(c) => c.hash(state),
            AnyPat(a) => a.hash(state),
            BinaryExpr(e) => e.hash(state),
            UnaryExpr(e) => e.hash(state),
            e @ Parend(_) => e.to_string().hash(state),
            e @ Bracketed(_) => e.to_string().hash(state),
        }
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
                Bracketed(expr) => format!("[{}]", expr.to_string()),
            }
        )
    }
}
