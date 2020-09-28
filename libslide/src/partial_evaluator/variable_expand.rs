//! This module expands variables in expressions to their definition form.

#![allow(unused)] // TODO: remove

use crate::grammar::*;
use crate::Span;

use std::collections::HashMap;

trait VariableExpander<'a> {
    /// Creates a new VariableExpander from an expression to expand.
    fn new(expr: RcExpr) -> Self;

    /// Expands the variables matching the lhs of `asgn` with the assignment definition.
    /// Consumes and returns self, providing a chaining API.
    fn expand(self, asgn: &'a Assignment) -> Self;

    /// Consumes `self` and returns the owned expression with any variables expanded.
    fn finish(self) -> RcExpr;
}

/// Eagerly expands variables in an expression.
///
/// An `EagerVariableExpander` expands all matching variables on every call to
/// [`expand`](VariableExpander::expand).
///
/// For example, `"a + b".expand("a = b + 1").expand("b = c")` would expand to `"c + 1 + c"`.
/// Compare this to a [`LazyVariableExpander`](self::LazyVariableExpander).
///
/// Note, however, that an `EagerVariableExpander` will not try to expand variables within an
/// expanded variable definition on the same call to [`expand`](VariableExpander::expand).
///
/// For example, `"a".expand("a = a + a")` would expand to `"a + a"` rather than
/// `"a + a + a + a + ..."`.
struct EagerVariableExpander<'a> {
    expr: RcExpr,
    expand_def: Option<&'a Assignment>,
}

impl<'a> ExpressionTransformer<'a> for EagerVariableExpander<'a> {
    fn transform_var(&self, var: &'a InternedStr, span: Span) -> RcExpr {
        let asgn = self.expand_def.unwrap();
        if var == &asgn.var {
            asgn.rhs.clone()
        } else {
            rc_expr!(Expr::Var(*var), span)
        }
    }
}

impl<'a> VariableExpander<'a> for EagerVariableExpander<'a> {
    fn new(expr: RcExpr) -> Self {
        Self {
            expr,
            expand_def: None,
        }
    }

    fn expand(mut self, asgn: &'a Assignment) -> Self {
        self.expand_def = Some(asgn);
        self.expr = self.transform(&self.expr);
        self
    }

    fn finish(self) -> RcExpr {
        self.expr
    }
}

/// Lazily expands variables in an expression.
///
/// A `LazyVariableExpander` expands each variable exactly once, when
/// [`finish`](VariableExpander::finish) is called, and does not expand variables within an expanded
/// variable definition.
///
/// For example, `"a + b".expand("a = b + 1").expand("b = c")` would expand to `"b + 1 + c"`.
/// Compare this to an [`EagerVariableExpander`](self::EagerVariableExpander).
///
/// Furthermore, the variable definition used by a `LazyVariableExpander` is the last one given via
/// [`expand`](VariableExpander::expand).
///
/// For example, `"a + a".expand("a = 1").expand("a = 10")` would expand to `"10 + 10"`.
struct LazyVariableExpander<'a> {
    expr: RcExpr,
    expand_defs: HashMap<&'a InternedStr, &'a RcExpr>,
}

impl<'a> ExpressionTransformer<'a> for LazyVariableExpander<'a> {
    fn transform_var(&self, var: &'a InternedStr, span: Span) -> RcExpr {
        match self.expand_defs.get(var) {
            Some(&def) => def.clone(),
            None => rc_expr!(Expr::Var(*var), span),
        }
    }
}

impl<'a> VariableExpander<'a> for LazyVariableExpander<'a> {
    fn new(expr: RcExpr) -> Self {
        Self {
            expr,
            expand_defs: HashMap::new(),
        }
    }

    fn expand(mut self, asgn: &'a Assignment) -> Self {
        self.expand_defs.insert(&asgn.var, &asgn.rhs);
        self
    }

    fn finish(self) -> RcExpr {
        self.transform(&self.expr)
    }
}

#[cfg(test)]
mod test {
    use super::{EagerVariableExpander, LazyVariableExpander, VariableExpander};
    use crate::{parse_asgn, parse_expr};

    #[test]
    fn eager_variable_expander() {
        let expr = parse_expr!("a + b + a + 1 / 2");
        let a = parse_asgn!("a = b / 5");
        let b = parse_asgn!("b = c");

        let expanded = EagerVariableExpander::new(expr)
            .expand(&a)
            .expand(&b)
            .finish();

        assert_eq!(expanded.to_string(), "c / 5 + c + c / 5 + 1 / 2");
    }

    #[test]
    fn lazy_variable_expander() {
        let expr = parse_expr!("a + b + a + 1 / 2");
        let a = parse_asgn!("a = b / 5");
        let b = parse_asgn!("b = c");

        let expanded = LazyVariableExpander::new(expr)
            .expand(&a)
            .expand(&b)
            .finish();

        assert_eq!(expanded.to_string(), "b / 5 + c + b / 5 + 1 / 2");
    }
}
