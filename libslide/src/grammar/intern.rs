//! Provides interners for slide [Grammar]s.
//! An interned slide [Grammar] is itself a [Grammar] for convenience of use.
//!
//! [Grammar]: super::Grammar

use crate::emit::Emit;
use crate::grammar::{Expr, ExprPat, Grammar};
use crate::Span;

use core::cmp::Ordering;
use lasso::{Rodeo, Spur};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::RwLock;

/// Describes an interned slide expression.
pub trait InternedExpression
where
    Self: Copy + Deref + Ord,
{
    /// The type of expression interned by Self.
    type Inner;

    /// Returns whether the expression is a statically-evaluatable constant.
    fn is_const(&self) -> bool;

    /// Paranthesizes `inner`.
    fn paren(inner: Self, span: Span) -> Self;

    /// Brackets `inner`.
    fn bracket(inner: Self, span: Span) -> Self;

    /// Creates an InternedExpression from a [BinaryExpr](super::BinaryExpr).
    fn binary(expr: super::BinaryExpr<Self>, span: Span) -> Self;

    /// Creates an InternedExpression from a [UnaryExpr](super::UnaryExpr).
    fn unary(expr: super::UnaryExpr<Self>, span: Span) -> Self;

    /// Returns an empty expression.
    fn empty(span: Span) -> Self;

    /// Returns the span of the expression.
    fn span(&self) -> Span;
}

/// An interner for arbitrary, hashable, non-string types.
#[derive(Debug)]
struct Interner<T> {
    /// Why two intern tables? The idea is to hash a grammar to a string, intern the string in a
    /// [Rodeo](lasso::Rodeo), and keep a table mapping the interned string reference to the
    /// original grammar. Rodeo provides fast lookups, so the largest constraint here is creating
    /// the string hash from a grammar.
    ///
    /// Okay, but why not just a `Map<Grammar, Grammar>`? This doesn't quite work because then we
    /// need to store &Grammar on the interned struct, which needs a lifetime. The only lifetime
    /// that would work non-locally is `'static`, but that is longer than the lifetime of variables
    /// declared via lazy_static.  
    ///
    /// Okay, but why not just a `Map<String, Grammar>`? Because then we must store a string on the
    /// interned struct, and clone a string anytime we intern a value (even if it already exists).
    /// This is much more expensive than a reference or 32-bit [Spur](lasso::Spur).
    rodeo: Rodeo<Spur>,

    /// Two structurally equivalent expressions may have different spurs, so we have to make sure
    /// that the expression we dereference is the one corresponding to exactly that span. For
    /// example, in
    ///   1 + 2 * 3 and
    ///   2 * 3 + 1
    /// the interned value for `2 * 3` will have the same spur, but if we dereference them to the
    /// same underlying expression, the spans of `2` and `3` will be the same when they are not.
    ///
    /// Thus, we hash by both spur and the span of the interned expression.
    // TODO(ayazhafiz): this might not be enough. For example, if we start recording the location
    // of operators, then this will fail for
    //   1 +  3
    //   1  + 3
    // Plus, marking expressions as unique based of the span gets rid of most of the usefulness of
    // interning anyway. Maybe this can just become an `Rc` container.
    spur_map: HashMap<(Spur, Span), Box<T>>,
}

impl<T> Default for Interner<T> {
    fn default() -> Self {
        Self {
            rodeo: Rodeo::default(),
            spur_map: HashMap::new(),
        }
    }
}

macro_rules! make_interner {
    ($($intern_macro:ident, $ty:ty, $interned_struct:ident, $interner:ident)*) => {$(
        lazy_static! {
            static ref $interner: RwLock<Interner<$ty>> = RwLock::new(Interner::default());
        }

        /// An interned version of an expression.
        ///
        /// NB: interned expressions are equivalent if they point to the same underlying expression,
        /// even though two interned expressions may have different [span](crate::Span)s.
        #[derive(Debug, Copy, Clone)]
        pub struct $interned_struct {
            /// Pointer to an expression in the intern map.
            spur: Spur,
            /// The original span of this expression from an input source code.
            /// Even though the expression is interned, this span is distinct and serves as a
            /// backwards-mapping to where the expression originally came from.
            pub(crate) span: Span,
        }

        impl $interned_struct {
            /// Interns the expression, or returns the existing interned reference if it already
            /// exists.
            pub(crate) fn intern<Sp>(expr: $ty, span: Sp) -> Self
            where
                Sp: Into<Span>
            {
                let span = span.into();
                let hash = expr.emit_s_expression();
                let mb = {
                    let interner_lk = $interner.read().expect("Failed to read interner; likely poisoned.");
                    interner_lk.rodeo.get(hash)
                };
                let spur = match mb {
                    Some(spur) => {
                        let expr_with_span_exists = {
                            let interner_lk = $interner.read().expect("Failed to read interner; likely poisoned.");
                            interner_lk.spur_map.get(&(spur, span)).is_some()
                        };
                        if !expr_with_span_exists {
                            // No need to acquire an RAII lock here because after insertion, the
                            // interner is in a well-formed state. Reads before the write will lead
                            // to a subsequent write (OK), and reads after the write will be as
                            // expected.
                            let mut interner_lk = $interner.write().expect("Failed to read intern reverse map; likely poisoned.");
                            interner_lk.spur_map.insert((spur, span), Box::new(expr));
                        }
                        spur
                    }
                    None => {
                        let hash = expr.emit_s_expression();
                        // Lock up the interner to prevent a race condition.
                        let mut interner_lk = $interner.write().expect("Failed to get rodeo");
                        let spur = interner_lk.rodeo.get_or_intern(hash);
                        interner_lk.spur_map.insert((spur, span), Box::new(expr));
                        spur
                    }
                };
                Self {spur, span}
            }
        }

        /// Interns an expression.
        #[macro_export]
        macro_rules! $intern_macro {
            ($expr: expr, $span: expr) => {
                $interned_struct::intern($expr, $span)
            }
        }

        impl Grammar for $interned_struct {}

        impl core::hash::Hash for $interned_struct {
            fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
                self.spur.hash(state);
            }
        }

        impl PartialEq for $interned_struct {
            fn eq(&self, other: &Self) -> bool {
                self.spur.eq(&other.spur)
            }
        }

        impl Eq for $interned_struct {}

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

            fn deref(&self) -> &Self::Target {
                let interner_lk = $interner.read().expect("Failed to read interner; likely poisoned.");
                let interned_p = interner_lk.spur_map.get(&(self.spur, self.span)).unwrap_or_else(|| panic!(
                    // Constructing this formatted string is decently expensive, so avoid it unless
                    // we actually can't find the interned value.
                    "BUG: intern key does not map to an interned value.
                    Self: {:?}
                    Rodeo: {:?}
                    Spur Map: {:?}", self, interner_lk.rodeo, interner_lk.spur_map
                ));
                let inner: &Self::Target = &**interned_p;
                // Safety:
                //
                // - semantics: OK because cast to ptr, then deref ptr and return reference
                //
                // - lifetimes: OK because values present in the intern tables last for the lifetime
                //   of the table, which is (nearly) static.
                //
                //   Note that the reference to the boxed expression does not truly have a static
                //   lifetime (and hence the need for an unsafe cast). This is because `lazy_static`
                //   is not actually `'static`; however, once interned, an expression persists for
                //   the remaining lifetime of the program, so its lifetime is "relatively static"
                //   (at least as long as that of any of its users).
                //
                // - invalidation of reference: OK because the reference to the expression we return
                //   is never reallocated. We may worry that if we return a reference to an
                //   expression that is a value of a k-v pair in a hashmap, if the hashmap is
                //   reallocated while we are using the reference, whatever we read is junk.
                //
                //   However, the reference to the expression returned is a boxed expression pointed
                //   to from the value of the k-v pair, and so will never be reallocated.  As an
                //   illustration, the k-v pairs can be seen as
                //
                //   (spur, span) -> expr pointer -> Box<Expr>
                //   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^              -- k-v pair belonging to a hash map,
                //                                                may be reallocated (e.g. if the
                //                                                map is resized)
                //                                   ^^^^^^^^^ -- "static" address in memory
                //
                //   And the reallocation model can be seen as
                //
                //   ------ Hash map -----
                //   |   |   |   | k |   |     (before reallocation)
                //   |   |   |   | v |   |
                //   --------------|------
                //                 \
                //          /-------\-> expr -- return a reference to this
                //         /
                //   ------|--------------
                //   |   | v |   |   |   |
                //   |   | k |   |   |   |     (after reallocation)
                //   ------ Hash map -----
                //
                //   so even if the map is reallocated, our reference is valid.
                unsafe {
                    &*(inner as *const $ty)
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
    )*};
}

make_interner! {
    intern_expr, Expr, InternedExpr, EXPR_RODEO
    intern_expr_pat, ExprPat, InternedExprPat, EXPR_PAT_RODEO
}

impl InternedExpression for InternedExpr {
    type Inner = Expr;

    #[inline]
    fn is_const(&self) -> bool {
        matches!(**self, Expr::Const(_))
    }

    #[inline]
    fn paren(inner: InternedExpr, span: Span) -> Self {
        intern_expr!(Expr::Parend(inner), span)
    }

    #[inline]
    fn bracket(inner: InternedExpr, span: Span) -> Self {
        intern_expr!(Expr::Bracketed(inner), span)
    }

    #[inline]
    fn binary(expr: super::BinaryExpr<Self>, span: Span) -> Self {
        intern_expr!(Expr::BinaryExpr(expr), span)
    }

    #[inline]
    fn unary(expr: super::UnaryExpr<Self>, span: Span) -> Self {
        intern_expr!(Expr::UnaryExpr(expr), span)
    }

    #[inline]
    fn empty(span: Span) -> Self {
        // Variables must be named, so we can encode an unnamed variable as an empty expression.
        intern_expr!(Expr::Var(String::new()), span)
    }

    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}

impl InternedExpression for InternedExprPat {
    type Inner = ExprPat;

    #[inline]
    fn is_const(&self) -> bool {
        matches!(**self, ExprPat::Const(_))
    }

    #[inline]
    fn paren(inner: InternedExprPat, span: Span) -> Self {
        intern_expr_pat!(ExprPat::Parend(inner), span)
    }

    #[inline]
    fn bracket(inner: InternedExprPat, span: Span) -> Self {
        intern_expr_pat!(ExprPat::Bracketed(inner), span)
    }

    #[inline]
    fn binary(expr: super::BinaryExpr<Self>, span: Span) -> Self {
        intern_expr_pat!(ExprPat::BinaryExpr(expr), span)
    }

    #[inline]
    fn unary(expr: super::UnaryExpr<Self>, span: Span) -> Self {
        intern_expr_pat!(ExprPat::UnaryExpr(expr), span)
    }

    #[inline]
    fn empty(span: Span) -> Self {
        // Patterns must be named, so we can encode an unnamed pattern as an empty expression.
        intern_expr_pat!(ExprPat::VarPat(String::new()), span)
    }

    #[inline]
    fn span(&self) -> Span {
        self.span
    }
}
