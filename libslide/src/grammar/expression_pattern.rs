use super::*;

use rug::Rational;

/// A slide expression pattern.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ExprPat {
    /// A constant
    Const(Rational),
    /// Pattern matching a variable
    VarPat(String),
    /// Pattern matching a constant
    ConstPat(String),
    /// Pattern matching any expression
    AnyPat(String),
    /// A binary expression
    BinaryExpr(BinaryExpr<RcExprPat>),
    /// A unary expression
    UnaryExpr(UnaryExpr<RcExprPat>),
    /// A paranthesized expression
    Parend(RcExprPat),
    /// A bracketed expression
    Bracketed(RcExprPat),
}

impl Grammar for ExprPat {}

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
