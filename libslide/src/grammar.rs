use crate::scanner::types::{Token, TokenType};
use core::convert::TryFrom;
use core::fmt;

pub enum Stmt {
    Expr(Expr),
    Assignment(Assignment),
}

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
    pub var: Var,
    pub rhs: Box<Expr>,
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(= {} {})", self.var, self.rhs)
    }
}

pub enum Expr {
    Float(f64),
    Int(i64),
    Var(Var),
    BinaryExpr(BinaryExpr),
    UnaryExpr(UnaryExpr),
    /// An expression wrapped in parentheses
    Parend(Box<Expr>),
    /// An expression wrapped in braces
    Braced(Box<Expr>),
}

impl From<f64> for Expr {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<i64> for Expr {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<Var> for Expr {
    fn from(var: Var) -> Self {
        Self::Var(var)
    }
}

impl From<BinaryExpr> for Expr {
    fn from(binary_expr: BinaryExpr) -> Self {
        Self::BinaryExpr(binary_expr)
    }
}

impl From<UnaryExpr> for Expr {
    fn from(unary_expr: UnaryExpr) -> Self {
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
                Float(num) => num.to_string(),
                Int(num) => num.to_string(),
                Var(var) => var.to_string(),
                BinaryExpr(binary_expr) => binary_expr.to_string(),
                UnaryExpr(unary_expr) => unary_expr.to_string(),
                Parend(expr) | Braced(expr) => expr.to_string(),
            }
        )
    }
}

pub struct Var {
    pub name: String,
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name,)
    }
}

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

pub struct BinaryExpr {
    pub op: BinaryOperator,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl fmt::Display for BinaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.op.to_string(),
            self.lhs.to_string(),
            self.rhs.to_string(),
        )
    }
}

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

pub struct UnaryExpr {
    pub op: UnaryOperator,
    pub rhs: Box<Expr>,
}

impl fmt::Display for UnaryExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {})", self.op.to_string(), self.rhs.to_string(),)
    }
}
