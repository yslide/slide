//! Provides interners for slide [Grammar]s.
//! An interned slide [Grammar] is itself a [Grammar] for convenience of use.
//!
//! [Grammar]: super::Grammar

use crate::emit::Emit;
use crate::grammar::{Expr, ExprPat, Grammar};

use core::cmp::Ordering;
use lasso::{Rodeo, Spur};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::RwLock;

/// Describes an interned slide expression.
pub trait InternedExpression
where
    Self: Copy + Deref + Ord + From<super::BinaryExpr<Self>> + From<super::UnaryExpr<Self>>,
{
    type Inner;

    /// Returns whether the expression is a statically-evaluatable constant.
    fn is_const(&self) -> bool;

    /// Paranthesizes `inner`.
    fn paren(inner: Self) -> Self;

    /// Brackets `inner`.
    fn bracket(inner: Self) -> Self;

    /// Returns an empty expression.
    fn empty() -> Self;
}

macro_rules! make_interner {
    ($($intern_macro:ident, $ty:ty, $interned_struct:ident, $rodeo:ident, $spur2expr:ident)*) => {$(
        lazy_static! {
            // Why two intern tables? The idea is to hash a grammar to a string, intern the string
            // in Rodeo, and keep a table mapping the interned string reference to the original
            // grammar. Rodeo provides fast lookups, so the largest constraint here is creating the
            // string hash from a grammar.
            //
            // Okay, but why not just a Map<Grammar, Grammar>? This doesn't quite work because then
            // we need to store &Grammar on the interned struct, which needs a lifetime. The only
            // lifetime that would work non-locally is 'static, but that is longer than the
            // lifetime of variables declared via lazy_static.
            //
            // Okay, but why not just a Map<String, Grammar>? Because then we must store a string
            // on the interned struct, and clone a string anytime we intern a value (even if it
            // already exists). This is much more expensive than a reference or 32-bit Spur.
            static ref $rodeo: RwLock<Rodeo<Spur>> = RwLock::new(Rodeo::default());
            static ref $spur2expr: RwLock<HashMap<Spur, $ty>> = RwLock::new(HashMap::new());
        }

        /// An interned version of an expression.
        #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
        pub struct $interned_struct(pub(crate) Spur);

        impl $interned_struct {
            /// Interns the expression, or returns the existing interned reference if it already
            /// exists.
            pub(crate) fn intern(expr: $ty) -> Self {
                let hash = expr.emit_s_expression();
                let mb = $rodeo
                    .read()
                    .expect("Failed to read intern rodeo; likely poisoned.")
                    .get(hash);
                match mb {
                    Some(spur) => Self(spur),
                    None => {
                        let spur = $rodeo
                            .write()
                            .expect("Failed to write to intern rodeo; likely poisoned.")
                            .get_or_intern(expr.emit_s_expression());
                        $spur2expr
                            .write()
                            .expect("Failed to write to intern reverse map; likely poisoned.")
                            .insert(spur, expr);
                        Self(spur)
                    }
                }
            }
        }

        /// Interns an expression.
        #[doc(hidden)]
        #[macro_export]
        macro_rules! $intern_macro {
            ($expr: expr) => {
                $interned_struct::intern($expr)
            }
        }

        impl Grammar for $interned_struct {}

        impl Emit for $interned_struct {
            fn emit_pretty(&self) -> String {
                self.as_ref().emit_pretty()
            }

            fn emit_s_expression(&self) -> String {
                self.as_ref().emit_s_expression()
            }

            fn emit_latex(&self) -> String {
                self.as_ref().emit_latex()
            }
        }

        impl core::fmt::Display for $interned_struct {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.emit_pretty(),)
            }
        }

        impl AsRef<$ty> for $interned_struct {
            fn as_ref(&self) -> &$ty {
                self.deref()
            }
        }

        impl Deref for $interned_struct {
            type Target = <Self as InternedExpression>::Inner;

            fn deref(&self) -> &<Self as InternedExpression>::Inner {
                let table = $spur2expr.read().expect("Failed to read intern reverse map; likely poisoned.");
                let val = table.get(&self.0).expect("BUG: intern key does not map to an interned value");
                // Safety:
                // - semantics: OK because cast to ptr, then deref ptr and return reference
                // - lifetimes: OK because values present in the intern tables last for the lifetime
                //   of the table, which is static.
                unsafe {
                    &*(val as *const $ty)
                }
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

        impl From<$ty> for $interned_struct {
            fn from(expr: $ty) -> Self {
                $intern_macro!(expr)
            }
        }
    )*};
}

make_interner! {
    intern_expr, Expr, InternedExpr, EXPR_RODEO, SPUR2EXPR
    intern_expr_pat, ExprPat, InternedExprPat, EXPR_PAT_RODEO, SPUR2EXPR_PAT
}

impl InternedExpression for InternedExpr {
    type Inner = Expr;

    #[inline]
    fn is_const(&self) -> bool {
        matches!(**self, Expr::Const(_))
    }

    #[inline]
    fn paren(inner: InternedExpr) -> Self {
        intern_expr!(Expr::Parend(inner))
    }

    #[inline]
    fn bracket(inner: InternedExpr) -> Self {
        intern_expr!(Expr::Bracketed(inner))
    }

    #[inline]
    fn empty() -> Self {
        // Variables must be named, so we can encode an unnamed variable as an empty expression.
        intern_expr!(Expr::Var(String::new()))
    }
}

impl InternedExpression for InternedExprPat {
    type Inner = ExprPat;

    #[inline]
    fn is_const(&self) -> bool {
        matches!(**self, ExprPat::Const(_))
    }

    #[inline]
    fn paren(inner: InternedExprPat) -> Self {
        intern_expr_pat!(ExprPat::Parend(inner))
    }

    #[inline]
    fn bracket(inner: InternedExprPat) -> Self {
        intern_expr_pat!(ExprPat::Bracketed(inner))
    }

    #[inline]
    fn empty() -> Self {
        // Patterns must be named, so we can encode an unnamed pattern as an empty expression.
        intern_expr_pat!(ExprPat::VarPat(String::new()))
    }
}
