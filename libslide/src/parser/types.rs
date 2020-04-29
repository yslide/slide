use crate::scanner::types::{Token, TokenType};
use core::convert::TryFrom;
use core::fmt;

pub enum Stmt {
    Expr(Expr),
    Assignment(Assignment),
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
    BinaryExpr(BinaryExpr),
    UnaryExpr(UnaryExpr),
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
                BinaryExpr(binary_expr) => binary_expr.to_string(),
                UnaryExpr(unary_expr) => unary_expr.to_string(),
            }
        )
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
        match token.token_type {
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
        match token.token_type {
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
