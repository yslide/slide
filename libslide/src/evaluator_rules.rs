mod pattern_matching;

use crate::grammar::*;
use crate::{parse_expression_pattern, scan};
use pattern_matching::match_rule;

use core::fmt;

// TODO: will uncomment in a future iteration when we support removing rules.
// pub enum RuleName {
//     AdditiveIdentity = 0,
//     ReorderConstants = 1,
// }

static DEFAULT_RULESET: &[&str] = &[
    "_a + 0 -> _a",       // AdditiveIdentity
    "#a + $b -> $b + #a", // ReorderConstants
];

/// A set of expression-mapping rules.
///
/// Each rule is of the string form
///
///   "<expr> -> <expr>"
///
/// Where <expr> is any expression pattern. An expression pattern is similar to any other
/// expression, differing only in its pattern matching variables. The form of pattern matching
/// variables and the expressions they match are as follows:
///   
///   | pattern | matches        |
///   |:------- |:-------------- |
///   | _<name> | Any expression |
///   | #<name> | A constant     |
///   | $<name> | A variable     |
///
/// To apply a rule, the lhs of the rule is pattern matched on the target expression. If the
/// matching is sucessful, the rhs of the rule is expanded with the results of the matching.
///
/// For example, the rule
///   
///   "$a + 0 -> $a"
///
/// Applied on the expression "x + 0" would yield "x".
///
/// Note that rules are matched and applied on expression parse trees, not their string
/// representations. This ensures rule application is always exact and deterministic.
pub struct RuleSet {
    rules: Vec<String>,
}

impl Default for RuleSet {
    /// Constructs the default rule set.
    fn default() -> Self {
        Self {
            rules: DEFAULT_RULESET.iter().map(|s| (*s).to_string()).collect(),
        }
    }
}

impl RuleSet {
    /// Creates a list of `BuiltRule`s from the rule set.
    pub fn build(&self) -> Vec<BuiltRule> {
        self.rules.iter().map(BuiltRule::from).collect()
    }
}

/// Parsed form of a rule. Used for pattern matching.
pub struct BuiltRule {
    from: ExprPat,
    to: ExprPat,
}

impl From<&String> for BuiltRule {
    /// Converts a string representation of a rule to a `BuiltRule`.
    /// A rule's string form must be
    ///   "<expr> -> <expr>"
    /// Where <expr> is an expression pattern. See [`RuleSet`] for more details.
    ///
    /// [`RuleSet`]: crate::evaluator_rules::RuleSet
    fn from(rule: &String) -> Self {
        let split = rule.split(" -> ");
        let mut split = split
            .map(scan)
            .map(parse_expression_pattern)
            .map(|(expr, _)| expr);

        // Unofficially, rustc's expression evaluation order is L2R, but officially it is undefined.
        let from = split.next().unwrap();
        let to = split.next().unwrap();
        Self { from, to }
    }
}

impl Transformer<Expr, Expr> for BuiltRule {
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
    ///   "$x + 0 -> $x".try_apply("x + 0")  // Some(x)
    ///   "$x + 0 -> $x".try_apply("x + 1")  // None
    ///   "$x + 0 -> $x".try_apply("x")      // None
    ///
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

        let replacements = match match_rule(self.from.clone(), partially_transformed.clone()) {
            Some(repl) => repl,
            // Could not match the rule on the top-level of the expression; return the partially
            // transformed expression.
            None => return partially_transformed,
        };

        // If the rule was matched on the expression, we have replacements for rule patterns ->
        // target subexpressions. Apply the rule by transforming the rule's RHS with the replacements.
        replacements.transform(self.to.clone())
    }
}

impl fmt::Display for BuiltRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.from.to_string(), self.to.to_string(),)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_rules() {
        let rule_set = RuleSet::default();
        let built_rules = rule_set.build();
        let var_plus_zero = &built_rules[0];

        assert_eq!(var_plus_zero.to_string(), "_a + 0 -> _a");
    }
}
