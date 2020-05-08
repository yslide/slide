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
        (Var(var), expr) => {
            let pat = VariablePattern::from_name(&var.name);
            use VariablePattern as Pat;
            match &expr {
                Const(_) => {
                    if pat & Pat::Const != Pat::Const {
                        return None;
                    }
                }
                Var(_) => {
                    if pat & Pat::Variable != Pat::Variable {
                        return None;
                    }
                }
                _ => {
                    // Only an Any pattern can match an expression that is not a const or var.
                    if pat & Pat::Any != Pat::Any {
                        return None;
                    }
                }
            }

            let mut replacements = Replacements::default();
            replacements.insert(Var(var), expr);
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
        (UnaryExpr(rule), UnaryExpr(expr)) => {
            if rule.op != expr.op {
                return None;
            }
            // Expressions are of the same type; match the rest of the expression by recursing on
            // the argument.
            match_rule(*rule.rhs, *expr.rhs)
        }
        (Parend(rule), Parend(expr)) => match_rule(*rule, *expr),
        (Braced(rule), Braced(expr)) => match_rule(*rule, *expr),
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
        use crate::{parse, scan, ScannerOptions};

        macro_rules! match_rule_tests {
            ($($name:ident: $rule:expr => $target:expr => $expected_repls:expr)*) => {
            $(
                #[test]
                fn $name() {
                    let parse = |prog: &str, is_rule: bool| -> Expr {
                        let mut scanner_options = ScannerOptions::default();
                        if is_rule {
                            scanner_options = scanner_options.set_is_var_char(|c| {
                                c.is_alphabetic() || c == '$' || c == '#' || c == '_'
                            });
                        }
                        match parse(scan(prog, scanner_options)) {
                            Stmt::Expr(expr) => expr,
                            _ => unreachable!(),
                        }
                    };

                    let parsed_rule = parse($rule, true);
                    let parsed_target = parse($target, false);

                    let repls = match_rule(parsed_rule, parsed_target);
                    let (repls, expected_repls): (Replacements, Vec<&str>) =
                        match (repls, $expected_repls) {
                            (None, expected_matches) => {
                                assert!(expected_matches.is_none());
                                return;
                            }
                            (Some(repl), expected_matches) => {
                                assert!(expected_matches.is_some());
                                (repl, expected_matches.unwrap())
                            }
                        };

                    let expected_repls = expected_repls
                        .into_iter()
                        .map(|m| m.split(": "))
                        .map(|mut i| (i.next().unwrap(), i.next().unwrap()))
                        .map(|(r, t)| (parse(r, true), parse(t, false)));

                    assert_eq!(expected_repls.len(), repls.map.len());

                    for (expected_pattern, expected_repl) in expected_repls {
                        assert_eq!(
                            expected_repl.to_string(),
                            repls.map.get(&expected_pattern).unwrap().to_string()
                        );
                    }
                }
            )*
            }
        }

        match_rule_tests! {
            consts:                     "0" => "0" => Some(vec![])
            consts_unmatched:           "0" => "1" => None

            variable_pattern:           "$a" => "x"     => Some(vec!["$a: x"])
            variable_pattern_on_const:  "$a" => "0"     => None
            variable_pattern_on_binary: "$a" => "x + 0" => None
            variable_pattern_on_unary:  "$a" => "+x"    => None

            const_pattern:              "#a" => "1"     => Some(vec!["#a: 1"])
            const_pattern_on_var:       "#a" => "x"     => None
            const_pattern_on_binary:    "#a" => "1 + x" => None
            const_pattern_on_unary:     "#a" => "+1"    => None

            any_pattern_on_variable:    "_a" => "x"     => Some(vec!["_a: x"])
            any_pattern_on_const:       "_a" => "1"     => Some(vec!["_a: 1"])
            any_pattern_on_binary:      "_a" => "1 + x" => Some(vec!["_a: 1 + x"])
            any_pattern_on_unary:       "_a" => "+(2)"  => Some(vec!["_a: +(2)"])

            binary_pattern:             "$a + #b" => "x + 0" => Some(vec!["$a: x", "#b: 0"])
            binary_pattern_wrong_op:    "$a + #b" => "x - 0" => None
            binary_pattern_partial:     "$a + #b" => "x + y" => None

            unary_pattern:              "+$a" => "+x" => Some(vec!["$a: x"])
            unary_pattern_wrong_op:     "+$a" => "-x" => None
            unary_pattern_partial:      "+$a" => "+1" => None

            parend:                     "($a + #b)" => "(x + 0)" => Some(vec!["$a: x", "#b: 0"])
            parend_on_braced:           "($a + #b)" => "[x + 0]" => None

            braced:                     "[$a + #b]" => "[x + 0]" => Some(vec!["$a: x", "#b: 0"])
            braced_on_parend:           "[$a + #b]" => "(x + 0)" => None
        }
    }
}
