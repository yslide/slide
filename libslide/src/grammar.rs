//! The primary libslide IR.

#[macro_use]
mod intern;
mod pattern;
mod transformer;
pub use intern::*;
pub use pattern::*;
pub use transformer::*;

use crate::emit::Emit;
use crate::scanner::types::{Token, TokenType};

use core::cmp::Ordering;
use core::convert::TryFrom;

/// Describes a top-level item in the libslide grammar.
pub trait Grammar
where
    Self: Emit,
{
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Expr(InternedExpr),
    Assignment(Assignment),
}

impl Grammar for Stmt {}

impl From<InternedExpr> for Stmt {
    fn from(expr: InternedExpr) -> Self {
        Stmt::Expr(expr)
    }
}

impl From<Assignment> for Stmt {
    fn from(asgn: Assignment) -> Self {
        Stmt::Assignment(asgn)
    }
}

#[derive(Clone, Debug)]
pub struct Assignment {
    pub var: String,
    pub rhs: InternedExpr,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    Const(f64),
    Var(String),
    BinaryExpr(BinaryExpr<InternedExpr>),
    UnaryExpr(UnaryExpr<InternedExpr>),
    /// An expression wrapped in parentheses
    Parend(InternedExpr),
    /// An expression wrapped in brackets
    Bracketed(InternedExpr),
}

impl Grammar for Expr {}

impl Expr {
    pub fn complexity(&self) -> u8 {
        1 + match self {
            Self::Const(_) => 0,
            Self::Var(_) => 0,
            Self::BinaryExpr(BinaryExpr { lhs, rhs, .. }) => lhs.complexity() + rhs.complexity(),
            Self::UnaryExpr(UnaryExpr { rhs, .. }) => rhs.complexity(),
            Self::Parend(expr) | Self::Bracketed(expr) => expr.complexity(),
        }
    }

    /// Gets the constant value stored in this expression, if any.
    pub fn get_const(&self) -> Option<f64> {
        match self {
            Self::Const(c) => Some(*c),
            _ => None,
        }
    }
}

impl Eq for Expr {}

impl PartialOrd for Expr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Expr {
    // For expression normalization.
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Var(a), Self::Var(b)) => a.cmp(b),
            (Self::Const(a), Self::Const(b)) => a.partial_cmp(b).unwrap(), // assume NaNs don't exist
            (Self::UnaryExpr(a), Self::UnaryExpr(b)) => a.cmp(b),
            (Self::BinaryExpr(a), Self::BinaryExpr(b)) => a.cmp(b),
            (Self::Parend(a), Self::Parend(b)) => a.cmp(b),
            (Self::Bracketed(a), Self::Bracketed(b)) => a.cmp(b),
            // Order: vars, consts, unary, binary, paren, brackets
            (Self::Const(_), Self::Var(_))
            | (Self::UnaryExpr(_), Self::Const(_))
            | (Self::UnaryExpr(_), Self::Var(_))
            | (Self::BinaryExpr(_), Self::UnaryExpr(_))
            | (Self::BinaryExpr(_), Self::Const(_))
            | (Self::BinaryExpr(_), Self::Var(_))
            | (Self::Parend(_), Self::BinaryExpr(_))
            | (Self::Parend(_), Self::UnaryExpr(_))
            | (Self::Parend(_), Self::Const(_))
            | (Self::Parend(_), Self::Var(_))
            | (Self::Bracketed(_), Self::Parend(_))
            | (Self::Bracketed(_), Self::BinaryExpr(_))
            | (Self::Bracketed(_), Self::UnaryExpr(_))
            | (Self::Bracketed(_), Self::Const(_))
            | (Self::Bracketed(_), Self::Var(_)) => Ordering::Greater,
            (Self::Var(_), _)
            | (Self::Const(_), _)
            | (Self::UnaryExpr(_), _)
            | (Self::BinaryExpr(_), _)
            | (Self::Parend(_), _) => Ordering::Less,
        }
    }
}

// TODO: We can do better than hashing to a string as well, but we'll save that til we have an
// arbitrary-precision numeric type.
#[allow(clippy::derive_hash_xor_eq)]
impl core::hash::Hash for Expr {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        use Expr::*;
        match self {
            // TODO: We can do better than hashing to a string as well, but we'll save that til we
            // have an arbitrary-precision numeric type.
            Const(f) => state.write(f.to_string().as_bytes()),
            Var(v) => v.hash(state),
            BinaryExpr(e) => e.hash(state),
            UnaryExpr(e) => e.hash(state),
            e @ Parend(_) => e.to_string().hash(state),
            e @ Bracketed(_) => e.to_string().hash(state),
        }
    }
}

impl From<f64> for Expr {
    fn from(f: f64) -> Self {
        Self::Const(f)
    }
}

impl From<BinaryExpr<InternedExpr>> for InternedExpr {
    fn from(binary_expr: BinaryExpr<InternedExpr>) -> Self {
        intern_expr!(Expr::BinaryExpr(binary_expr))
    }
}

impl From<UnaryExpr<InternedExpr>> for InternedExpr {
    fn from(unary_expr: UnaryExpr<InternedExpr>) -> Self {
        intern_expr!(Expr::UnaryExpr(unary_expr))
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub enum BinaryOperator {
    // Discrimant values exist to describe a formal ordering, and are grouped by tens to express
    // precedence.
    Plus = 1,
    Minus = 2,
    Mult = 10,
    Div = 11,
    Mod = 12,
    Exp = 20,
}

impl BinaryOperator {
    pub(crate) fn precedence(&self) -> u8 {
        (*self as u8) / 10
    }

    pub(crate) fn is_associative(&self) -> bool {
        use BinaryOperator::*;
        matches!(self, Plus | Mult | Exp)
    }
}

impl TryFrom<&Token> for BinaryOperator {
    type Error = ();

    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        use BinaryOperator::*;
        match token.ty {
            TokenType::Plus => Ok(Plus),
            TokenType::Minus => Ok(Minus),
            TokenType::Mult => Ok(Mult),
            TokenType::Div => Ok(Div),
            TokenType::Mod => Ok(Mod),
            TokenType::Exp => Ok(Exp),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct BinaryExpr<E: InternedExpression> {
    pub op: BinaryOperator,
    pub lhs: E,
    pub rhs: E,
}

macro_rules! mkop {
    ($($op_name:ident: $binop:path)*) => {
    $(
        pub fn $op_name<T, U>(lhs: T, rhs: U) -> Self
        where
            T: Into<E>,
            U: Into<E>,
        {
            Self {
                op: $binop,
                lhs: lhs.into(),
                rhs: rhs.into(),
            }
        }
    )*
    }
}

impl<E> BinaryExpr<E>
where
    E: InternedExpression,
{
    mkop! {
        mult: BinaryOperator::Mult
        div:  BinaryOperator::Div
        exp:  BinaryOperator::Exp
    }
}

impl<E> PartialOrd for BinaryExpr<E>
where
    E: InternedExpression,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for BinaryExpr<E>
where
    E: InternedExpression,
{
    fn cmp(&self, other: &Self) -> Ordering {
        if self.op != other.op {
            self.op.cmp(&other.op)
        } else if self.lhs != other.lhs {
            self.lhs.cmp(&other.lhs)
        } else {
            self.rhs.cmp(&other.rhs)
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Debug)]
pub enum UnaryOperator {
    SignPositive = 1,
    SignNegative = 2,
}

impl TryFrom<&Token> for UnaryOperator {
    type Error = ();

    fn try_from(token: &Token) -> Result<Self, Self::Error> {
        use UnaryOperator::*;
        match token.ty {
            TokenType::Plus => Ok(SignPositive),
            TokenType::Minus => Ok(SignNegative),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct UnaryExpr<E: InternedExpression> {
    pub op: UnaryOperator,
    pub rhs: E,
}

impl<E> UnaryExpr<E>
where
    E: InternedExpression,
{
    pub fn negate<T>(expr: T) -> Self
    where
        T: Into<E>,
    {
        Self {
            op: UnaryOperator::SignNegative,
            rhs: expr.into(),
        }
    }
}

impl<E> PartialOrd for UnaryExpr<E>
where
    E: InternedExpression,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for UnaryExpr<E>
where
    E: InternedExpression,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.op.cmp(&other.op) {
            Ordering::Equal => self.rhs.cmp(&other.rhs),
            ord => ord,
        }
    }
}
