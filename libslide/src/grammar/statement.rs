use super::*;
use crate::Span;

/// A list of statements in a slide program.
#[derive(Clone, Debug)]
pub struct StmtList {
    /// The list of statements.
    list: Vec<Stmt>,
}

impl Grammar for StmtList {}

impl StmtList {
    pub(crate) fn new(list: Vec<Stmt>) -> Self {
        Self { list }
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, Stmt> {
        self.list.iter()
    }
}

pub struct StmtListIterator {
    stmts: <Vec<Stmt> as IntoIterator>::IntoIter,
}

impl Iterator for StmtListIterator {
    type Item = Stmt;

    fn next(&mut self) -> Option<Self::Item> {
        self.stmts.next()
    }
}

impl IntoIterator for StmtList {
    type Item = Stmt;
    type IntoIter = StmtListIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            stmts: self.list.into_iter(),
        }
    }
}

/// A statement in a slide program.
#[derive(Clone, Debug)]
pub enum Stmt {
    /// An expression statement is a statement that consists solely of an expression. For example,
    /// the slide program
    ///
    /// ```text
    /// 1 + 1
    /// ```
    ///
    /// contains one statement, that statement also being an expression.
    Expr(RcExpr),
    /// An assignment binds some value to a variable. For example the statement
    ///
    /// ```text
    /// x = 1 + 1
    /// ```
    ///
    /// binds the expression "1 + 1" to "x".
    Assignment(Assignment),
}

impl Grammar for Stmt {}

impl From<RcExpr> for Stmt {
    fn from(expr: RcExpr) -> Self {
        Stmt::Expr(expr)
    }
}

impl From<Assignment> for Stmt {
    fn from(asgn: Assignment) -> Self {
        Stmt::Assignment(asgn)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AssignmentOp {
    /// =
    Equal(Span),
    /// :=
    AssignDefine(Span),
}

impl AssignmentOp {
    pub fn span(&self) -> &Span {
        match self {
            AssignmentOp::Equal(span) => span,
            AssignmentOp::AssignDefine(span) => span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Assignment {
    pub var: InternedStr,
    pub asgn_op: AssignmentOp,
    pub rhs: RcExpr,
    pub span: Span,
}

impl Assignment {
    /// Redefines the [`Assignment`][Assignment] with a definition-evaluating function `eval`.
    pub fn redefine_with(mut self, eval: impl FnOnce(RcExpr) -> RcExpr) -> Self {
        self.rhs = eval(self.rhs);
        self
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    Const(f64),
    Var(InternedStr),
    BinaryExpr(BinaryExpr<RcExpr>),
    UnaryExpr(UnaryExpr<RcExpr>),
    /// An expression wrapped in parentheses
    Parend(RcExpr),
    /// An expression wrapped in brackets
    Bracketed(RcExpr),
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
            (Self::Var(a), Self::Var(b)) => a.get().cmp(&b.get()),
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
