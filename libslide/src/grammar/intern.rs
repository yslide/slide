//! Provides interners for slide [Grammar]s.
//! An interned slide [Grammar] is itself a [Grammar] for convenience of use.
//!
//! [Grammar]: super::Grammar

use crate::emit::{Emit, EmitConfig};
use crate::grammar::{Expr, ExprPat, Grammar};
use crate::Span;

use core::cmp::Ordering;
use lasso::{Rodeo, Spur};
use lazy_static::lazy_static;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::RwLock;

lazy_static! {
    /// Arena of interned strings.
    static ref INTERNED_STRS: RwLock<Rodeo<Spur>> = RwLock::new(Rodeo::default());
    /// A static reference to an empty string.
    static ref EMPTY_STR: InternedStr = InternedStr::intern("");
}

/// An interned [String][std::string::String] type.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct InternedStr(Spur);

impl InternedStr {
    /// Interns a string, or returns the existing interned reference if it already exists.
    pub(crate) fn intern<S: AsRef<str>>(s: S) -> Self {
        Self(
            INTERNED_STRS
                .write()
                .expect("Failed to write intern arena.")
                .get_or_intern(s),
        )
    }

    /// Gets the string interned at this reference.
    pub(crate) fn get(&self) -> String {
        unsafe {
            INTERNED_STRS
                .read()
                .expect("Failed to read intern arena.")
                .resolve_unchecked(&self.0)
                .to_owned()
        }
    }
}

impl std::fmt::Display for InternedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

/// Interns a string.
#[macro_export]
macro_rules! intern_str {
    ($str:expr) => {
        $crate::grammar::InternedStr::intern($str)
    };
}

/// Describes a reference-counted slide expression.
pub trait RcExpression
where
    Self: Deref + Ord + Clone,
{
    /// The type of expression held by Self.
    type Inner;

    /// Returns whether the expression is a statically-evaluatable constant.
    fn is_const(&self) -> bool;

    /// Returns whether the expression is a terminable variable (or variable-like).
    fn is_var(&self) -> bool;

    /// Paranthesizes `inner`.
    fn paren(inner: Self, span: Span) -> Self;

    /// Brackets `inner`.
    fn bracket(inner: Self, span: Span) -> Self;

    /// Creates an RcExpression from a [BinaryExpr](super::BinaryExpr).
    fn binary(expr: super::BinaryExpr<Self>, span: Span) -> Self;

    /// Creates an RcExpression from a [UnaryExpr](super::UnaryExpr).
    fn unary(expr: super::UnaryExpr<Self>, span: Span) -> Self;

    /// Returns an empty expression.
    fn empty(span: Span) -> Self;

    /// Returns the span of the expression.
    fn span(&self) -> Span;
}

macro_rules! make_interner {
    ($($intern_macro:ident, $ty:ty, $interned_struct:ident, $interner:ident)*) => {$(
        /// An interned version of an expression.
        ///
        /// NB: interned expressions are equivalent if they point to the same underlying expression,
        /// even though two interned expressions may have different [span](crate::Span)s.
        #[derive(Debug, Clone)]
        pub struct $interned_struct {
            /// The underlying expression.
            expr: Rc<$ty>,
            /// The original span of this expression from an input source code.
            /// Even though the expression is interned, this span is distinct and serves as a
            /// backwards-mapping to where the expression originally came from.
            pub(crate) span: Span,
        }

        impl $interned_struct {
            /// Creates a new reference-counted expression at a span.
            pub(crate) fn new<Sp>(expr: $ty, span: Sp) -> Self
            where
                Sp: Into<Span>
            {
                Self {
                    expr: Rc::new(expr),
                    span: span.into(),
                }
            }
        }

        /// Interns an expression.
        #[macro_export]
        macro_rules! $intern_macro {
            ($expr: expr, $span: expr) => {
                $interned_struct::new($expr, $span)
            }
        }

        impl Grammar for $interned_struct {}

        impl core::hash::Hash for $interned_struct {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.expr.hash(state);
            }
        }

        impl PartialEq for $interned_struct {
            fn eq(&self, other: &Self) -> bool {
                self.expr.eq(&other.expr)
            }
        }

        impl Eq for $interned_struct {}

        impl Emit for $interned_struct {
            fn emit_pretty(&self, config: EmitConfig) -> String {
                self.as_ref().emit_pretty(config)
            }

            fn emit_s_expression(&self, config: EmitConfig) -> String {
                self.as_ref().emit_s_expression(config)
            }

            fn emit_latex(&self, config: EmitConfig) -> String {
                self.as_ref().emit_latex(config)
            }
        }

        impl core::fmt::Display for $interned_struct {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.emit_pretty(EmitConfig::default()))
            }
        }

        impl AsRef<$ty> for $interned_struct {
            fn as_ref(&self) -> &$ty {
                self.deref()
            }
        }

        impl Deref for $interned_struct {
            type Target = <Self as RcExpression>::Inner;

            fn deref(&self) -> &Self::Target {
                self.expr.deref()
            }
        }

        impl PartialOrd for $interned_struct {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $interned_struct {
            fn cmp(&self, other: &Self) -> Ordering {
                self.as_ref().cmp(&other.as_ref())
            }
        }
    )*};
}

make_interner! {
    rc_expr, Expr, RcExpr, EXPR_RODEO
    rc_expr_pat, ExprPat, RcExprPat, EXPR_PAT_RODEO
}

impl RcExpression for RcExpr {
    type Inner = Expr;

    #[inline]
    fn is_const(&self) -> bool {
        matches!(**self, Expr::Const(_))
    }

    #[inline]
    fn is_var(&self) -> bool {
        matches!(**self, Expr::Var(_))
    }

    #[inline]
    fn paren(inner: RcExpr, span: Span) -> Self {
        rc_expr!(Expr::Parend(inner), span)
    }

    #[inline]
    fn bracket(inner: RcExpr, span: Span) -> Self {
        rc_expr!(Expr::Bracketed(inner), span)
    }

    #[inline]
    fn binary(expr: super::BinaryExpr<Self>, span: Span) -> Self {
        rc_expr!(Expr::BinaryExpr(expr), span)
    }

    #[inline]
    fn unary(expr: super::UnaryExpr<Self>, span: Span) -> Self {
        rc_expr!(Expr::UnaryExpr(expr), span)
    }

    #[inline]
    fn empty(span: Span) -> Self {
        // Variables must be named, so we can encode an unnamed variable as an empty expression.
        rc_expr!(Expr::Var(*EMPTY_STR), span)
    }

    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl RcExpression for RcExprPat {
    type Inner = ExprPat;

    #[inline]
    fn is_const(&self) -> bool {
        matches!(**self, ExprPat::Const(_))
    }

    #[inline]
    fn is_var(&self) -> bool {
        matches!(**self, ExprPat::ConstPat(_) | ExprPat::VarPat(_) | ExprPat::AnyPat(_))
    }

    #[inline]
    fn paren(inner: RcExprPat, span: Span) -> Self {
        rc_expr_pat!(ExprPat::Parend(inner), span)
    }

    #[inline]
    fn bracket(inner: RcExprPat, span: Span) -> Self {
        rc_expr_pat!(ExprPat::Bracketed(inner), span)
    }

    #[inline]
    fn binary(expr: super::BinaryExpr<Self>, span: Span) -> Self {
        rc_expr_pat!(ExprPat::BinaryExpr(expr), span)
    }

    #[inline]
    fn unary(expr: super::UnaryExpr<Self>, span: Span) -> Self {
        rc_expr_pat!(ExprPat::UnaryExpr(expr), span)
    }

    #[inline]
    fn empty(span: Span) -> Self {
        // Patterns must be named, so we can encode an unnamed pattern as an empty expression.
        rc_expr_pat!(ExprPat::VarPat(String::new()), span)
    }

    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
