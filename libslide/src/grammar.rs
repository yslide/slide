mod pattern;
mod transformer;
pub use pattern::*;
pub use transformer::*;

use crate::scanner::types::{Token, TokenType};

use core::cmp::Ordering;
use core::convert::TryFrom;
use core::fmt;
use std::rc::Rc;

pub trait Grammar
where
    Self: fmt::Display + fmt::Debug,
{
    /// Returns the S-expression form of this expression.
    /// For example, 1 + 1 -> (+ 1 1).
    fn s_form(&self) -> String;
}
pub trait Expression
where
    Self: fmt::Display + From<BinaryExpr<Self>> + From<UnaryExpr<Self>> + Ord,
{
    /// Returns whether the expression is a statically-evaluatable constant.
    fn is_const(&self) -> bool;

    /// Paranthesizes `inner`.
    fn paren(inner: Rc<Self>) -> Self;

    /// Brackets `inner`.
    fn bracket(inner: Rc<Self>) -> Self;

    /// Returns an empty expression.
    fn empty() -> Self;
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Expr(Expr),
    Assignment(Assignment),
}

impl Grammar for Stmt {
    fn s_form(&self) -> String {
        match self {
            Self::Expr(expr) => expr.s_form(),
            Self::Assignment(Assignment { var, rhs }) => format!("(= {} {})", var, rhs.s_form()),
        }
    }
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

#[derive(Clone, Debug)]
pub struct Assignment {
    pub var: String,
    pub rhs: Rc<Expr>,
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.var, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    Const(f64),
    Var(String),
    BinaryExpr(BinaryExpr<Self>),
    UnaryExpr(UnaryExpr<Self>),
    /// An expression wrapped in parentheses
    Parend(Rc<Self>),
    /// An expression wrapped in brackets
    Bracketed(Rc<Self>),
}

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

impl Grammar for Expr {
    fn s_form(&self) -> String {
        match self {
            Self::Const(konst) => konst.to_string(),
            Self::Var(var) => var.to_string(),
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
impl Grammar for Rc<Expr> {
    fn s_form(&self) -> String {
        self.as_ref().s_form()
    }
}

impl Expression for Expr {
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
        // Variables must be named, so we can encode an unnamed variable as an empty expression.
        Expr::Var(String::new())
    }
}

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
                Bracketed(expr) => format!("[{}]", expr.to_string()),
            }
        )
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
    fn precedence(&self) -> u8 {
        (*self as u8) / 10
    }

    fn is_associative(&self) -> bool {
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

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct BinaryExpr<E: Expression> {
    pub op: BinaryOperator,
    pub lhs: Rc<E>,
    pub rhs: Rc<E>,
}

macro_rules! mkop {
    ($($op_name:ident: $binop:path)*) => {
    $(
        pub fn $op_name<T, U>(lhs: T, rhs: U) -> Self
        where
            T: Into<Rc<E>>,
            U: Into<Rc<E>>,
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
    E: Expression,
{
    mkop! {
        mult: BinaryOperator::Mult
        div:  BinaryOperator::Div
        exp:  BinaryOperator::Exp
    }
}

macro_rules! display_binary_expr {
    (<$expr:ident>) => {
        impl fmt::Display for BinaryExpr<$expr> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut result = String::with_capacity(128);
                use $expr::*;
                let format_arg = |arg: &Rc<$expr>, right_child: bool| match arg.as_ref() {
                    // We want to format items like
                    //    v--------- child op
                    //         v---- parent op
                    // (3 + 5) ^ 2 [1]
                    //  3 + 5  + 2
                    //  3 - 5  + 2
                    //  3 * 5  + 2
                    // and
                    //   v---------- parent op
                    //        v----- child op
                    // 2 +  3 + 5
                    // 2 - (3 + 5)
                    // 2 * (3 + 5)
                    //
                    // So the idea here is as follows:
                    // - if the child op precedence is less than the parent op, we must always
                    //   parenthesize it ([1])
                    // - if the op precedences are equivalent, then
                    //   - if the child is on the LHS, we can always unwrap it
                    //   - if the child is on the RHS, we parenthesize it unless the parent op is
                    //     associative
                    //
                    // I think this is enough, but maybe we're overlooking left/right
                    // associativity?
                    BinaryExpr(child) => {
                        if child.op.precedence() < self.op.precedence()
                            || (right_child
                                && child.op.precedence() == self.op.precedence()
                                && !self.op.is_associative())
                        {
                            format!("({})", child)
                        } else {
                            child.to_string()
                        }
                    }
                    expr => expr.to_string(),
                };
                result.push_str(&format!(
                    "{} {} {}",
                    format_arg(&self.lhs, false),
                    self.op,
                    format_arg(&self.rhs, true)
                ));
                f.write_str(&result)
            }
        }
    };
}

display_binary_expr!(<Expr>);
display_binary_expr!(<ExprPat>);

impl<E> PartialOrd for BinaryExpr<E>
where
    E: Expression,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for BinaryExpr<E>
where
    E: Expression,
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

#[derive(PartialEq, Eq, Clone, Hash, Debug)]
pub struct UnaryExpr<E: Expression> {
    pub op: UnaryOperator,
    pub rhs: Rc<E>,
}

impl<E> UnaryExpr<E>
where
    E: Expression,
{
    pub fn negate<T>(expr: T) -> Self
    where
        T: Into<Rc<E>>,
    {
        Self {
            op: UnaryOperator::SignNegative,
            rhs: expr.into(),
        }
    }
}

macro_rules! display_unary_expr {
    (<$expr:ident>) => {
        impl fmt::Display for UnaryExpr<$expr> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut result = String::with_capacity(128);
                use $expr::*;
                let format_arg = |arg: &Rc<$expr>| match arg.as_ref() {
                    BinaryExpr(l) => format!("({})", l),
                    expr => expr.to_string(),
                };
                result.push_str(&format!("{}{}", self.op, format_arg(&self.rhs)));
                f.write_str(&result)
            }
        }
    };
}

display_unary_expr!(<Expr>);
display_unary_expr!(<ExprPat>);

impl<E> PartialOrd for UnaryExpr<E>
where
    E: Expression,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<E> Ord for UnaryExpr<E>
where
    E: Expression,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.op.cmp(&other.op) {
            Ordering::Equal => self.rhs.as_ref().cmp(&other.rhs.as_ref()),
            ord => ord,
        }
    }
}
