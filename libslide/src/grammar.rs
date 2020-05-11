mod pattern;
mod transformer;
pub use pattern::*;
pub use transformer::*;

use crate::scanner::types::{Token, TokenType};

use core::convert::TryFrom;
use core::fmt;
use std::rc::Rc;

pub trait Grammar {}
pub trait Expression
where
    Self: fmt::Display + From<BinaryExpr<Self>> + From<UnaryExpr<Self>>,
{
}

pub enum Stmt {
    Expr(Expr),
    Assignment(Assignment),
}

impl Grammar for Stmt {}

impl From<Expr> for Stmt {
    fn from(expr: Expr) -> Self {
        Stmt::Expr(expr)
    }
}

impl From<Assignment> for Stmt {
    fn from(asgn: Assignment) -> Self {
        Stmt::Assignment(asgn)
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Stmt::*;
        write!(
            f,
            "{}",
            match self {
                Expr(expr) => expr.to_string(),
                Assignment(asgn) => asgn.to_string(),
            }
        )
    }
}

pub struct Assignment {
    pub var: String,
    pub rhs: Rc<Expr>,
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.var, self.rhs)
    }
}

#[derive(Clone, PartialEq)]
pub enum Expr {
    Const(f64),
    Var(String),
    BinaryExpr(BinaryExpr<Self>),
    UnaryExpr(UnaryExpr<Self>),
    /// An expression wrapped in parentheses
    Parend(Rc<Self>),
    /// An expression wrapped in braces
    Braced(Rc<Self>),
}

impl Eq for Expr {}

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
            e @ Braced(_) => e.to_string().hash(state),
        }
    }
}

impl Grammar for Expr {}
impl Grammar for Rc<Expr> {}
impl Expression for Expr {}

impl From<f64> for Expr {
    fn from(f: f64) -> Self {
        Self::Const(f)
    }
}

impl From<BinaryExpr<Self>> for Expr {
    fn from(binary_expr: BinaryExpr<Self>) -> Self {
        Self::BinaryExpr(binary_expr)
    }
}

impl From<UnaryExpr<Self>> for Expr {
    fn from(unary_expr: UnaryExpr<Self>) -> Self {
        Self::UnaryExpr(unary_expr)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        write!(
            f,
            "{}",
            match self {
                Const(num) => num.to_string(),
                Var(var) => var.to_string(),
                BinaryExpr(binary_expr) => binary_expr.to_string(),
                UnaryExpr(unary_expr) => unary_expr.to_string(),
                Parend(expr) => format!("({})", expr.to_string()),
                Braced(expr) => format!("[{}]", expr.to_string()),
            }
        )
    }
}

#[derive(PartialEq, Clone, Copy, Hash)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Mult,
    Div,
    Mod,
    Exp,
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

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BinaryOperator::*;
        write!(
            f,
            "{}",
            match self {
                Plus => "+",
                Minus => "-",
                Mult => "*",
                Div => "/",
                Mod => "%",
                Exp => "^",
            }
        )
    }
}

#[derive(PartialEq, Clone, Hash)]
pub struct BinaryExpr<E: Expression> {
    pub op: BinaryOperator,
    pub lhs: Rc<E>,
    pub rhs: Rc<E>,
}

impl<E: Expression> fmt::Display for BinaryExpr<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.lhs.to_string(),
            self.op.to_string(),
            self.rhs.to_string(),
        )
    }
}

#[derive(PartialEq, Clone, Copy, Hash)]
pub enum UnaryOperator {
    SignPositive,
    SignNegative,
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

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use UnaryOperator::*;
        write!(
            f,
            "{}",
            match self {
                SignPositive => "+",
                SignNegative => "-",
            }
        )
    }
}

#[derive(PartialEq, Clone, Hash)]
pub struct UnaryExpr<E: Expression> {
    pub op: UnaryOperator,
    pub rhs: Rc<E>,
}

impl<E: Expression> fmt::Display for UnaryExpr<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.op.to_string(), self.rhs.to_string(),)
    }
}
