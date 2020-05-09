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
pub fn match_rule(rule: ExprPat, target: Expr) -> Option<Replacements> {
    // TODO: Could we pass rule and target by reference?
    match (rule, target) {
        // The happiest path -- if a pattern matches an expression, return replacements for it!
        (pat @ ExprPat::VarPat(_), expr @ Expr::Var(_))
        | (pat @ ExprPat::ConstPat(_), expr @ Expr::Const(_))
        | (pat @ ExprPat::AnyPat(_), expr) => {
            let mut replacements = Replacements::default();
            replacements.insert(pat, expr);
            Some(replacements)
        }
        (ExprPat::Const(a), Expr::Const(b)) => {
            if (a - b).abs() > std::f64::EPSILON {
                // Constants don't match; rule can't be applied.
                return None;
            }
            // Constant values are... constant, so there is no need to replace them.
            Some(Replacements::default())
        }
        (ExprPat::BinaryExpr(rule), Expr::BinaryExpr(expr)) => {
            if rule.op != expr.op {
                return None;
            }
            // Expressions are of the same type; match the rest of the expression by recursing on
            // the arguments.
            let replacements_lhs: Replacements = match_rule(*rule.lhs, *expr.lhs)?;
            let replacements_rhs: Replacements = match_rule(*rule.rhs, *expr.rhs)?;
            Replacements::try_merge(replacements_lhs, replacements_rhs)
        }
        (ExprPat::UnaryExpr(rule), Expr::UnaryExpr(expr)) => {
            if rule.op != expr.op {
                return None;
            }
            // Expressions are of the same type; match the rest of the expression by recursing on
            // the argument.
            match_rule(*rule.rhs, *expr.rhs)
        }
        (ExprPat::Parend(rule), Expr::Parend(expr)) => match_rule(*rule, *expr),
        (ExprPat::Braced(rule), Expr::Braced(expr)) => match_rule(*rule, *expr),
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
        ExprPat, // rule pattern, like #a
        Expr,    // target expr,  like 10
    >,
}

impl Transformer<ExprPat, Expr> for Replacements {
    /// Transforms a pattern expression into an expression by replacing patterns with target
    /// expressions known by the [`Replacements`].
    ///
    /// This transformation can be used to apply a rule on an expression by transforming the RHS
    /// using patterns matched between the LHS of the rule and the target expression.
    ///
    /// [`Replacements`]: Replacements
    fn transform(&self, item: ExprPat) -> Expr {
        match item {
            ExprPat::VarPat(_) | ExprPat::ConstPat(_) | ExprPat::AnyPat(_) => {
                match self.map.get(&item) {
                    // We know about this pattern, replace it with its target expression.
                    Some(transformed) => transformed.clone(),

                    // A pattern can never be transformed to an expression if it is not replaced!
                    // TODO: Return an error rather than panicking
                    None => unreachable!(),
                }
            }

            ExprPat::Const(f) => Expr::Const(f),
            ExprPat::BinaryExpr(binary_expr) => BinaryExpr {
                op: binary_expr.op,
                lhs: self.transform(*binary_expr.lhs).into(),
                rhs: self.transform(*binary_expr.rhs).into(),
            }
            .into(),
            ExprPat::UnaryExpr(unary_expr) => UnaryExpr {
                op: unary_expr.op,
                rhs: self.transform(*unary_expr.rhs).into(),
            }
            .into(),
            ExprPat::Parend(expr) => Expr::Parend(self.transform(*expr).into()),
            ExprPat::Braced(expr) => Expr::Braced(self.transform(*expr).into()),
        }
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

    fn insert(&mut self, k: ExprPat, v: Expr) -> Option<Expr> {
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
            let a = ExprPat::VarPat("a".into());
            let b = ExprPat::VarPat("b".into());
            let c = ExprPat::VarPat("c".into());

            let mut left = Replacements::default();
            left.insert(a.clone(), Expr::Const(1.));
            left.insert(b.clone(), Expr::Const(2.));

            let mut right = Replacements::default();
            right.insert(b.clone(), Expr::Const(2.));
            right.insert(c.clone(), Expr::Const(3.));

            let merged = Replacements::try_merge(left, right).unwrap();
            assert_eq!(merged.map.len(), 3);
            assert_eq!(merged.map.get(&a).unwrap().to_string(), "1");
            assert_eq!(merged.map.get(&b).unwrap().to_string(), "2");
            assert_eq!(merged.map.get(&c).unwrap().to_string(), "3");
        }

        #[test]
        fn try_merge_overlapping_non_matching() {
            let a = ExprPat::VarPat("a".into());

            let mut left = Replacements::default();
            left.insert(a.clone(), Expr::Const(1.));

            let mut right = Replacements::default();
            right.insert(a, Expr::Const(2.));

            let merged = Replacements::try_merge(left, right);
            assert!(merged.is_none());
        }
    }

    mod match_rule {
        use super::*;
        use crate::{parse_expression, parse_expression_pattern, scan};

        macro_rules! match_rule_tests {
            ($($name:ident: $rule:expr => $target:expr => $expected_repls:expr)*) => {
            $(
                #[test]
                fn $name() {
                    let parse_rule = |prog: &str| -> ExprPat {
                        let (expr, _) = parse_expression_pattern(scan(prog));
                        expr
                    };
                    let parse_expr = |prog: &str| -> Expr {
                        match parse_expression(scan(prog)) {
                            (Stmt::Expr(expr), _) => expr,
                            _ => unreachable!(),
                        }
                    };

                    let parsed_rule = parse_rule($rule);
                    let parsed_target = parse_expr($target);

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
                        .map(|(r, t)| (parse_rule(r), parse_expr(t)));

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
