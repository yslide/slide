use core::fmt;

use crate::grammar::*;
use crate::{parse_expression_pattern, scan};

use super::pattern_match::match_rule;

/// A mapping between two expression patterns.
pub struct PatternMap {
    from: ExprPat,
    to: ExprPat,
}

pub enum Rule {
    PatternMap(PatternMap),
    Evaluate(fn(Expr) -> Option<Expr>),
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

    pub fn from_fn(f: fn(Expr) -> Option<Expr>) -> Self {
        Self::Evaluate(f)
    }
}

impl Transformer<Expr, Expr> for Rule {
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
    fn transform(&self, target: Expr) -> Expr {
        // First, apply the rule recursively on the target's subexpressions.
        let partially_transformed = match target {
            expr @ Expr::Const(_) => expr,
            var @ Expr::Var(_) => var,
            Expr::BinaryExpr(binary_expr) => BinaryExpr {
                op: binary_expr.op,
                lhs: self.transform(*binary_expr.lhs).into(),
                rhs: self.transform(*binary_expr.rhs).into(),
            }
            .into(),
            Expr::UnaryExpr(unary_expr) => UnaryExpr {
                op: unary_expr.op,
                rhs: self.transform(*unary_expr.rhs).into(),
            }
            .into(),
            Expr::Parend(expr) => Expr::Parend(self.transform(*expr).into()),
            Expr::Braced(expr) => Expr::Braced(self.transform(*expr).into()),
        };

        match self {
            Rule::PatternMap(PatternMap { from, to }) => {
                let replacements = match match_rule(from.clone(), partially_transformed.clone()) {
                    Some(repl) => repl,
                    // Could not match the rule on the top-level of the expression; return the
                    // partially transformed expression.
                    None => return partially_transformed,
                };

                // If the rule was matched on the expression, we have replacements for rule
                // patterns -> target subexpressions. Apply the rule by transforming the rule's RHS
                // with the replacements.
                replacements.transform(to.clone())
            }
            Rule::Evaluate(f) => f(partially_transformed.clone()).unwrap_or(partially_transformed),
        }
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
