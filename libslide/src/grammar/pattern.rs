pub use super::*;

use std::rc::Rc;

#[derive(Clone, Debug)]
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
    Parend(Rc<Self>),
    Bracketed(Rc<Self>),
}

impl Grammar for ExprPat {}
impl Grammar for Rc<ExprPat> {}

impl Expression for ExprPat {
    #[inline]
    fn is_const(&self) -> bool {
        matches!(self, Self::Const(_))
    }

    #[inline]
    fn paren(inner: Rc<Self>) -> Self {
        Self::Parend(inner)
    }

    #[inline]
    fn bracket(inner: Rc<Self>) -> Self {
        Self::Bracketed(inner)
    }

    #[inline]
    fn empty() -> Self {
        // Patterns must be named, so we can encode an unnamed pattern as an empty expression.
        ExprPat::VarPat(String::new())
    }
}

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

impl PartialOrd for ExprPat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExprPat {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::VarPat(a), Self::VarPat(b))
            | (Self::VarPat(a), Self::ConstPat(b))
            | (Self::VarPat(a), Self::AnyPat(b))
            | (Self::ConstPat(a), Self::VarPat(b))
            | (Self::ConstPat(a), Self::ConstPat(b))
            | (Self::ConstPat(a), Self::AnyPat(b))
            | (Self::AnyPat(a), Self::VarPat(b))
            | (Self::AnyPat(a), Self::ConstPat(b))
            | (Self::AnyPat(a), Self::AnyPat(b)) => a.cmp(b),
            (Self::Const(a), Self::Const(b)) => a.partial_cmp(b).unwrap(), // assume NaNs don't exist
            (Self::UnaryExpr(a), Self::UnaryExpr(b)) => a.cmp(b),
            (Self::BinaryExpr(a), Self::BinaryExpr(b)) => a.cmp(b),
            (Self::Parend(a), Self::Parend(b)) => a.cmp(b),
            (Self::Bracketed(a), Self::Bracketed(b)) => a.cmp(b),
            // Order: vars, consts, unary, binary, paren, brackets
            (Self::Const(_), Self::VarPat(_))
            | (Self::Const(_), Self::ConstPat(_))
            | (Self::Const(_), Self::AnyPat(_))
            | (Self::UnaryExpr(_), Self::Const(_))
            | (Self::UnaryExpr(_), Self::VarPat(_))
            | (Self::UnaryExpr(_), Self::ConstPat(_))
            | (Self::UnaryExpr(_), Self::AnyPat(_))
            | (Self::BinaryExpr(_), Self::UnaryExpr(_))
            | (Self::BinaryExpr(_), Self::Const(_))
            | (Self::BinaryExpr(_), Self::VarPat(_))
            | (Self::BinaryExpr(_), Self::ConstPat(_))
            | (Self::BinaryExpr(_), Self::AnyPat(_))
            | (Self::Parend(_), Self::BinaryExpr(_))
            | (Self::Parend(_), Self::UnaryExpr(_))
            | (Self::Parend(_), Self::Const(_))
            | (Self::Parend(_), Self::VarPat(_))
            | (Self::Parend(_), Self::ConstPat(_))
            | (Self::Parend(_), Self::AnyPat(_))
            | (Self::Bracketed(_), Self::Parend(_))
            | (Self::Bracketed(_), Self::BinaryExpr(_))
            | (Self::Bracketed(_), Self::UnaryExpr(_))
            | (Self::Bracketed(_), Self::Const(_))
            | (Self::Bracketed(_), Self::VarPat(_))
            | (Self::Bracketed(_), Self::ConstPat(_))
            | (Self::Bracketed(_), Self::AnyPat(_)) => Ordering::Greater,
            (Self::VarPat(_), _)
            | (Self::ConstPat(_), _)
            | (Self::AnyPat(_), _)
            | (Self::Const(_), _)
            | (Self::UnaryExpr(_), _)
            | (Self::BinaryExpr(_), _)
            | (Self::Parend(_), _) => Ordering::Less,
        }
    }
}
