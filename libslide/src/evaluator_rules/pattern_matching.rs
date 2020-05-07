use super::variables::VariablePattern;
use crate::grammar::*;
use std::collections::HashMap;

/// Pattern matches a rule template against an expression. If successful, the results of the
/// matching are returned as a mapping of rule to target expressions replacements.
///
/// A sucessful pattern matching is one that matches the target expression wholly, abiding the
/// expression pattern matching rules. See [`RuleSet`] for more details.
///
/// [mapping]: crate::evaluator_rules::pattern_matching::Replacements
/// [`RuleSet`]: crate::evaluator_rules::RuleSet
pub fn match_rule(rule: Expr, target: Expr) -> Option<Replacements> {
    // TODO: Could we pass rule and target by reference?
    use Expr::*;
    match (rule, target) {
        (Const(a), Const(b)) => {
            if (a - b).abs() > std::f64::EPSILON {
                // Constants don't match; rule can't be applied.
                return None;
            }
            // Constant values are... constant, so there is no need to replace them.
            Some(Replacements::default())
        }
        (Var(var), var_e @ Var(_)) => {
            if VariablePattern::from_name(&var.name) & VariablePattern::Variable == 0 {
                // Variable pattern does not match a variable; rule can't be applied.
                return None;
            }
            let mut replacements = Replacements::default();
            replacements.insert(Var(var), var_e);
            Some(replacements)
        }
        (BinaryExpr(rule), BinaryExpr(expr)) => {
            if rule.op != expr.op {
                return None;
            }
            // Expressions are of the same type; match the rest of the expression by recursing on
            // the arguments.
            let replacements_lhs: Replacements = match_rule(*rule.lhs, *expr.lhs)?;
            let replacements_rhs: Replacements = match_rule(*rule.rhs, *expr.rhs)?;
            Replacements::try_merge(replacements_lhs, replacements_rhs)
        }
        _ => None,
    }
}

#[derive(Default)]
/// Represents pattern-matched replacements betwen a rule and a target expression.
///
/// The rhs of a rule may be transfomed with an instance of `Replacements` to obtain the result of a
/// rule applied on a target expression.
pub struct Replacements {
    map: HashMap<
        Expr, // rule expr,   like #a
        Expr, // target expr, like 10
    >,
}

impl Transformer for Replacements {
    /// Transforms an expressions by replacing matching subexpressions with target expressions
    /// known by the `Replacements`.
    fn transform_expr(&self, item: Expr) -> Expr {
        // TODO: handle bad rule (item is a expression pattern but not in Replacements)
        if let Some(transformed) = self.map.get(&item) {
            return transformed.clone();
        }
        self.multiplex_transform_expr(item)
    }
}

impl Replacements {
    /// Merges two `Replacements`. If the `Replacements` are of incompatible state (i.e. contain
    /// different mappings), merging fails and nothing is returned.
    fn try_merge(left: Replacements, right: Replacements) -> Option<Replacements> {
        let mut replacements = left;
        for (from, to_r) in right.map.into_iter() {
            if let Some(to_l) = replacements.map.get(&from) {
                if to_r != *to_l {
                    // Replacement already exists and its value does not match exactly; bail out.
                    return None;
                }
                continue; // no need to insert replacement again
            }
            // Replacement is new, add it.
            replacements.map.insert(from, to_r);
        }
        Some(replacements)
    }

    fn insert(&mut self, k: Expr, v: Expr) -> Option<Expr> {
        self.map.insert(k, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod replacements {
        use super::*;

        #[test]
        fn try_merge() {
            let var_a = Expr::Var("a".into());
            let var_b = Expr::Var("b".into());
            let var_c = Expr::Var("c".into());

            let mut left = Replacements::default();
            left.insert(var_a.clone(), Expr::Const(1.));
            left.insert(var_b.clone(), Expr::Const(2.));

            let mut right = Replacements::default();
            right.insert(var_b.clone(), Expr::Const(2.));
            right.insert(var_c.clone(), Expr::Const(3.));

            let merged = Replacements::try_merge(left, right).unwrap();
            assert_eq!(merged.map.len(), 3);
            assert_eq!(merged.map.get(&var_a).unwrap().to_string(), "1");
            assert_eq!(merged.map.get(&var_b).unwrap().to_string(), "2");
            assert_eq!(merged.map.get(&var_c).unwrap().to_string(), "3");
        }

        #[test]
        fn try_merge_overlapping_non_matching() {
            let var_a = Expr::Var("a".into());

            let mut left = Replacements::default();
            left.insert(var_a.clone(), Expr::Const(1.));

            let mut right = Replacements::default();
            right.insert(var_a, Expr::Const(2.));

            let merged = Replacements::try_merge(left, right);
            assert!(merged.is_none());
        }
    }

    mod match_rule {
        use super::*;

        fn var(s: &str) -> Expr {
            Expr::Var(Var {
                name: s.to_string(),
            })
        }
        fn const_e(n: f64) -> Expr {
            Expr::Const(n)
        }

        // TODO: more tests?
        #[test]
        fn match_rule_has_match() {
            let replacements = match_rule(
                Expr::BinaryExpr(BinaryExpr {
                    op: BinaryOperator::Plus,
                    lhs: var("$a").into(),
                    rhs: const_e(0.).into(),
                }),
                Expr::BinaryExpr(BinaryExpr {
                    op: BinaryOperator::Plus,
                    lhs: var("x").into(),
                    rhs: const_e(0.).into(),
                }),
            )
            .unwrap();

            assert_eq!(replacements.map.len(), 1);
            assert_eq!(replacements.map.get(&var("$a")).unwrap().to_string(), "x");
        }

        #[test]
        fn match_rule_no_match() {
            let replacements = match_rule(const_e(0.), const_e(1.));

            assert!(replacements.is_none());
        }
    }
}
