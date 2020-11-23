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

/// The kind of a statement.
#[derive(Clone, Debug)]
pub enum StmtKind {
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
impl From<RcExpr> for StmtKind {
    fn from(expr: RcExpr) -> Self {
        StmtKind::Expr(expr)
    }
}

impl From<Assignment> for StmtKind {
    fn from(asgn: Assignment) -> Self {
        StmtKind::Assignment(asgn)
    }
}

/// A statement in a slide program.
#[derive(Clone, Debug)]
pub struct Stmt {
    /// The [kind](StmtKind) of the statement.
    pub kind: StmtKind,
    /// Vertical whitespace present before the statement.
    vw: usize,
}

impl Grammar for Stmt {}

impl Stmt {
    /// Creates a new `Stmt`.
    pub fn new(kind: StmtKind, vw: usize) -> Self {
        Self { kind, vw }
    }

    /// Update `self` with a fresh statement [kind](StmtKind), given functions for how a statement
    /// should be generated.
    pub fn update_with(
        self,
        expr_update: impl FnOnce(RcExpr) -> RcExpr,
        asgn_update: impl FnOnce(Assignment) -> Assignment,
    ) -> Self {
        let kind = match self.kind {
            StmtKind::Expr(expr) => expr_update(expr).into(),
            StmtKind::Assignment(asgn) => asgn_update(asgn).into(),
        };
        Self { kind, ..self }
    }

    /// Retrieve the number of vertical whitespace lines above this statement.
    pub fn vw(&self) -> usize {
        self.vw
    }

    /// Gets the span of the statement.
    pub fn span(&self) -> &Span {
        match &self.kind {
            StmtKind::Expr(e) => &e.span,
            StmtKind::Assignment(a) => &a.span,
        }
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

/// An assignment.
#[derive(Clone, Debug)]
pub struct Assignment {
    /// Left hand side of the assignment.
    pub lhs: RcExpr,
    /// The assignment operator.
    pub asgn_op: AssignmentOp,
    /// Right hand side of the assignment.
    pub rhs: RcExpr,

    /// Span of the entire assignment.
    pub span: Span,
}

impl Assignment {
    /// Redefines `self` with a definition-evaluating function `eval`.
    pub fn redefine_with(mut self, eval: impl FnOnce(RcExpr) -> RcExpr) -> Self {
        self.rhs = eval(self.rhs);
        self
    }
}

/// An expression.
#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    /// A constant.
    Const(f64),
    /// A variable.
    Var(InternedStr),
    /// A binary expression.
    BinaryExpr(BinaryExpr<RcExpr>),
    /// A unary expression.
    UnaryExpr(UnaryExpr<RcExpr>),
    /// An expression wrapped in parentheses.
    Parend(RcExpr),
    /// An expression wrapped in brackets.
    Bracketed(RcExpr),
}

impl Grammar for Expr {}

impl Expr {
    pub(crate) fn complexity(&self) -> u8 {
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

    /// Gets the variable value stored in this expression, if any.
    pub fn get_var(&self) -> Option<InternedStr> {
        match self {
            Self::Var(v) => Some(*v),
            _ => None,
        }
    }

    /// Returns `true` iff the expression is a variable.
    pub fn is_var(&self) -> bool {
        matches!(self, Self::Var(_))
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
