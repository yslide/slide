use super::pattern_match::{MatchRule, PatternMatch};
use crate::grammar::*;
use crate::utils::{get_symmetric_expressions, hash, indent, unique_pats};
use crate::{parse_expression_pattern, scan};

use core::fmt;
use std::collections::HashMap;
use std::error::Error;

/// A mapping between two expression patterns.
#[derive(Clone, Debug)]
pub struct PatternMap {
    pub from: InternedExprPat,
    pub to: InternedExprPat,
}

impl fmt::Display for PatternMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}

#[derive(Debug)]
pub struct UnresolvedMapping {
    map: PatternMap,
    unresolved_pats: Vec<InternedExprPat>,
}

impl fmt::Display for UnresolvedMapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut missing_pats = self
            .unresolved_pats
            .iter()
            .map(|p| format!(r#""{}""#, p))
            .collect::<Vec<_>>();
        missing_pats.sort();
        let missing_pats = missing_pats.join(", ");

        write!(
            f,
            r#"Could not resolve pattern map
{}"{from} -> {to}"
Specifically, source "{from}" is missing pattern(s) {pats} present in target "{to}""#,
            indent("\n", 4),
            from = self.map.from,
            to = self.map.to,
            pats = missing_pats,
        )
    }
}

impl Error for UnresolvedMapping {}

impl PatternMap {
    /// Converts a string representation of a rule to a `PatternMap`.
    /// A rule's string form must be
    ///
    /// ```text
    /// "<expr> -> <expr>"
    /// ```
    ///
    /// Where `<expr>` is an expression pattern.
    pub fn from_str(rule: &str) -> Self {
        let split = rule.split(" -> ");
        let mut split = split
            .map(|toks| scan(toks).tokens)
            .map(parse_expression_pattern)
            .map(|(expr, _)| expr);

        // Unofficially, rustc's expression evaluation order is L2R, but officially it is undefined.
        let from = split.next().unwrap();
        let to = split.next().unwrap();
        Self { from, to }
    }

    /// Bootstraps a `PatternMap` rule with a one-pass application of a rule set, which may include
    /// the rule itself.
    ///
    /// This allows the rule to match evaluated contexts that cannot be represented by a
    /// `PatternMap` created from just a string form. For example, the rule
    ///
    /// ```text
    /// "-(_a - _b) -> _b - _a"
    /// ```
    ///
    /// in its raw form may only be applied to an expression with explicit parentheses. By
    /// bootstrapping this rule with a rule that removes explicit parentheses, the rule can be
    /// applied on expressions with implicit parentheses (i.e. of the prefix form `(- (- _a _b))`).
    pub fn bootstrap(&self, bootstrapping_rules: &[Rule]) -> Self {
        let mut bootstrapped = self.clone();
        for rule in bootstrapping_rules.iter() {
            bootstrapped.from = rule.transform(bootstrapped.from);
            bootstrapped.to = rule.transform(bootstrapped.to);
        }
        bootstrapped
    }

    /// Checks a `PatternMap` is resolvable, returning an error if it is not.
    pub fn validate(&self) -> Option<UnresolvedMapping> {
        let unresolved_pats: Vec<_> = unique_pats(&self.to)
            .difference(&unique_pats(&self.from))
            .map(|p| **p)
            .collect();

        if unresolved_pats.is_empty() {
            return None;
        }

        Some(UnresolvedMapping {
            map: self.clone(),
            unresolved_pats,
        })
    }
}

pub enum Rule {
    PatternMap(PatternMap),
    Evaluate(fn(InternedExpr) -> Option<InternedExpr>),
}

impl Rule {
    pub fn from_fn(f: fn(InternedExpr) -> Option<InternedExpr>) -> Self {
        Self::Evaluate(f)
    }

    pub fn from_str(s: &str) -> Self {
        Self::PatternMap(PatternMap::from_str(s))
    }
}

impl Transformer<InternedExpr, InternedExpr> for Rule {
    /// Attempts to apply a rule on a target expression by
    ///
    /// 1. Applying the rule recursively on the target's subexpression to obtain a
    ///    partially-transformed target expression.
    ///
    /// 2. Pattern matching the lhs of the rule with the partially-transformed target expression.
    ///   - If pattern matching is unsuccessful, no application is done and the original expression
    ///     is returned.
    ///
    /// 3. Expanding the rhs of the rule using the results of the pattern matching.
    ///
    /// Examples:
    ///
    /// ```ignore
    /// "$x + 0 -> $x".try_apply("x + 0")  // Some(x)
    /// "$x + 0 -> $x".try_apply("x + 1")  // None
    /// "$x + 0 -> $x".try_apply("x")      // None
    /// ```
    fn transform(&self, target: InternedExpr) -> InternedExpr {
        fn fill(
            cache: &mut HashMap<u64, InternedExpr>,
            t: InternedExpr,
            r: InternedExpr,
        ) -> InternedExpr {
            *cache.entry(hash(t.as_ref())).or_insert_with(|| r)
        }

        fn transform_inner(
            rule: &Rule,
            target: InternedExpr,
            cache: &mut HashMap<u64, InternedExpr>,
        ) -> InternedExpr {
            match target.as_ref() {
                Expr::Const(_) => target,
                Expr::Var(_) => target,
                Expr::BinaryExpr(binary_expr) => intern_expr!(
                    Expr::BinaryExpr(BinaryExpr {
                        op: binary_expr.op,
                        lhs: transform(rule, binary_expr.lhs, cache),
                        rhs: transform(rule, binary_expr.rhs, cache),
                    }),
                    target.span
                ),
                Expr::UnaryExpr(unary_expr) => intern_expr!(
                    Expr::UnaryExpr(UnaryExpr {
                        op: unary_expr.op,
                        rhs: transform(rule, unary_expr.rhs, cache),
                    }),
                    target.span
                ),
                Expr::Parend(expr) => {
                    let inner = transform(rule, *expr, cache);
                    intern_expr!(Expr::Parend(inner), target.span)
                }
                Expr::Bracketed(expr) => {
                    let inner = transform(rule, *expr, cache);
                    intern_expr!(Expr::Bracketed(inner), target.span)
                }
            }
        }

        fn transform(
            rule: &Rule,
            target: InternedExpr,
            cache: &mut HashMap<u64, InternedExpr>,
        ) -> InternedExpr {
            if let Some(result) = cache.get(&hash(target.as_ref())) {
                return *result;
            }

            let mut result = target;
            match rule {
                Rule::PatternMap(PatternMap { from, to }) => {
                    for target in get_symmetric_expressions(target) {
                        // First, apply the rule recursively on the target's subexpressions.
                        let partially_transformed = transform_inner(rule, target, cache);
                        if partially_transformed.complexity() < result.complexity() {
                            result = partially_transformed;
                        }

                        if let Some(transformed) =
                            PatternMatch::match_rule(*from, partially_transformed)
                                // If the rule was matched on the expression, we have replacements for rule
                                // patterns -> target subexpressions. Apply the rule by transforming the
                                // rule's RHS with the replacements.
                                .map(|repls| repls.transform(*to))
                        {
                            result = transformed;
                        }
                    }
                }
                Rule::Evaluate(f) => {
                    // First, apply the rule recursively on the target's subexpressions.
                    let partially_transformed = transform_inner(rule, target, cache);
                    result = f(partially_transformed).unwrap_or(partially_transformed);
                }
            }

            fill(cache, target, result)
        }

        let mut cache: HashMap<u64, InternedExpr> = HashMap::new();
        transform(self, target, &mut cache)
    }
}

impl Transformer<InternedExprPat, InternedExprPat> for Rule {
    /// Bootstraps a rule with another (or possibly the same) rule.
    fn transform(&self, target: InternedExprPat) -> InternedExprPat {
        // First, apply the rule recursively on the target's subexpressions.
        let og_span = target.span;
        let partially_transformed = match target.as_ref() {
            ExprPat::Const(_) | ExprPat::VarPat(_) | ExprPat::ConstPat(_) | ExprPat::AnyPat(_) => {
                target
            }
            ExprPat::BinaryExpr(binary_expr) => intern_expr_pat!(
                ExprPat::BinaryExpr(BinaryExpr {
                    op: binary_expr.op,
                    lhs: self.transform(binary_expr.lhs),
                    rhs: self.transform(binary_expr.rhs),
                }),
                og_span
            ),
            ExprPat::UnaryExpr(unary_expr) => {
                let rhs = self.transform(unary_expr.rhs);
                intern_expr_pat!(
                    ExprPat::UnaryExpr(UnaryExpr {
                        op: unary_expr.op,
                        rhs,
                    }),
                    og_span
                )
            }
            ExprPat::Parend(expr) => {
                let inner = self.transform(*expr);
                intern_expr_pat!(ExprPat::Parend(inner), og_span)
            }
            ExprPat::Bracketed(expr) => {
                let inner = self.transform(*expr);
                intern_expr_pat!(ExprPat::Bracketed(inner), og_span)
            }
        };

        match self {
            Rule::PatternMap(PatternMap { from, to }) => {
                PatternMatch::match_rule(*from, partially_transformed)
                    .map(|repls| repls.transform(*to))
            }
            // Only pattern map rules can be used for bootstrapping. Function rules should be exact.
            _ => unreachable!(),
        }
        .unwrap_or(partially_transformed)
    }
}

fn fn_name<T>(_: T) -> &'static str {
    let name = std::any::type_name::<T>();
    name.split("::").last().unwrap()
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::PatternMap(pm) => pm.to_string(),
                Self::Evaluate(fun) => fn_name(fun).to_string(),
            }
        )
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PatternMap(pm) => write!(f, "{:?}", pm),
            Self::Evaluate(fun) => write!(f, "{}", fn_name(fun)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_error() {
        let err = PatternMap::from_str("_a + $b / #c * 3 -> 1 + $b * #e / _f")
            .validate()
            .unwrap();

        assert_eq!(
            err.to_string(),
            r##"Could not resolve pattern map
    "_a + $b / #c * 3 -> 1 + $b * #e / _f"
Specifically, source "_a + $b / #c * 3" is missing pattern(s) "#e", "_f" present in target "1 + $b * #e / _f""##
        );
    }

    #[test]
    fn validate_ok() {
        assert!(PatternMap::from_str("_a + $b / #c -> _a + $b")
            .validate()
            .is_none());
    }
}
