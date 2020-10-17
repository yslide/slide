//! The primary libslide IR.

#[macro_use]
mod mem;
pub mod collectors;
mod expression_pattern;
mod statement;
mod transformer;
pub mod visit;
pub use expression_pattern::*;
pub use mem::*;
pub use statement::*;
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
pub struct BinaryExpr<E: RcExpression> {
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
    E: RcExpression,
{
    mkop! {
        sub: BinaryOperator::Minus
        mult: BinaryOperator::Mult
        div:  BinaryOperator::Div
        exp:  BinaryOperator::Exp
    }
}

impl<E> PartialOrd for BinaryExpr<E>
where
    E: RcExpression,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for BinaryExpr<E>
where
    E: RcExpression,
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
pub struct UnaryExpr<E: RcExpression> {
    pub op: UnaryOperator,
    pub rhs: E,
}

impl<E> UnaryExpr<E>
where
    E: RcExpression,
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
    E: RcExpression,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for UnaryExpr<E>
where
    E: RcExpression,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.op.cmp(&other.op) {
            Ordering::Equal => self.rhs.cmp(&other.rhs),
            ord => ord,
        }
    }
}
