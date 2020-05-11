use super::pattern_match::match_rule;
use crate::grammar::*;
use crate::utils::hash;
use crate::{parse_expression_pattern, scan};

use core::fmt;
use std::collections::HashMap;
use std::rc::Rc;

/// A mapping between two expression patterns.
pub struct PatternMap {
    from: ExprPat,
    to: ExprPat,
}

pub enum Rule {
    PatternMap(PatternMap),
    Evaluate(fn(&Expr) -> Option<Expr>),
}

impl Rule {
    /// Converts a string representation of a rule to a `BuiltRule`.
    /// A rule's string form must be
    ///   "<expr> -> <expr>"
    /// Where <expr> is an expression pattern. See [`RuleSet`] for more details.
    ///
    /// [`RuleSet`]: crate::evaluator_rules::RuleSet
    pub fn from_str(rule: &str) -> Self {
        let split = rule.split(" -> ");
        let mut split = split
            .map(scan)
            .map(parse_expression_pattern)
            .map(|(expr, _)| expr);

        // Unofficially, rustc's expression evaluation order is L2R, but officially it is undefined.
        let from = split.next().unwrap();
        let to = split.next().unwrap();
        Self::PatternMap(PatternMap { from, to })
    }

    pub fn from_fn(f: fn(&Expr) -> Option<Expr>) -> Self {
        Self::Evaluate(f)
    }
}

impl Transformer<Rc<Expr>, Rc<Expr>> for Rule {
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
    /// ```rust, ignore
    /// "$x + 0 -> $x".try_apply("x + 0")  // Some(x)
    /// "$x + 0 -> $x".try_apply("x + 1")  // None
    /// "$x + 0 -> $x".try_apply("x")      // None
    /// ```
    fn transform(&self, target: Rc<Expr>) -> Rc<Expr> {
        fn fill(cache: &mut HashMap<u64, Rc<Expr>>, t: Rc<Expr>, r: Rc<Expr>) -> Rc<Expr> {
            Rc::clone(cache.entry(hash(t.as_ref())).or_insert_with(|| r))
        }

        fn transform(
            rule: &Rule,
            target: Rc<Expr>,
            cache: &mut HashMap<u64, Rc<Expr>>,
        ) -> Rc<Expr> {
            if let Some(result) = cache.get(&hash(target.as_ref())) {
                return Rc::clone(result);
            }

            // First, apply the rule recursively on the target's subexpressions.
            let partially_transformed = match target.as_ref() {
                Expr::Const(_) => Rc::clone(&target),
                Expr::Var(_) => Rc::clone(&target),
                Expr::BinaryExpr(binary_expr) => Expr::BinaryExpr(BinaryExpr {
                    op: binary_expr.op,
                    lhs: transform(rule, Rc::clone(&binary_expr.lhs), cache),
                    rhs: transform(rule, Rc::clone(&binary_expr.rhs), cache),
                })
                .into(),
                Expr::UnaryExpr(unary_expr) => Expr::UnaryExpr(UnaryExpr {
                    op: unary_expr.op,
                    rhs: transform(rule, Rc::clone(&unary_expr.rhs), cache),
                })
                .into(),
                Expr::Parend(expr) => {
                    let inner = transform(rule, Rc::clone(expr), cache);
                    Expr::Parend(inner).into()
                }
                Expr::Braced(expr) => {
                    let inner = transform(rule, Rc::clone(expr), cache);
                    Expr::Braced(inner).into()
                }
            };

            let transformed = match rule {
                Rule::PatternMap(PatternMap { from, to }) => {
                    match_rule(Rc::new(from.clone()), Rc::clone(&partially_transformed))
                        // If the rule was matched on the expression, we have replacements for rule
                        // patterns -> target subexpressions. Apply the rule by transforming the
                        // rule's RHS with the replacements.
                        .map(|repls| repls.transform(Rc::new(to.clone())))
                }
                Rule::Evaluate(f) => f(partially_transformed.as_ref()).map(Rc::new),
            }
            .unwrap_or(partially_transformed);

            fill(cache, target, transformed)
        }

        // Expr pointer -> transformed expression. Assumes that transient expressions of the same
        // value are reference counters pointing to the same underlying expression. This is done
        // via common subexpression elimination during parsing.
        let mut cache: HashMap<u64, Rc<Expr>> = HashMap::new();
        transform(self, target, &mut cache)
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fn_name<T>(_: T) -> &'static str {
            let name = std::any::type_name::<T>();
            name.split("::").last().unwrap()
        }
        match self {
            Self::PatternMap(PatternMap { from, to }) => {
                write!(f, "{} -> {}", from.to_string(), to.to_string())
            }
            Self::Evaluate(fun) => write!(f, "{}", fn_name(fun)),
        }
    }
}
