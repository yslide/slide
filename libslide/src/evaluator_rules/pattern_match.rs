use crate::grammar::*;
use crate::utils::hash;

use std::collections::HashMap;

/// Represents pattern-matched replacements betwen a rule and a target expression.
///
/// The rhs of a rule may be transfomed with an instance of `PatternMatch` to obtain the result of a
/// rule applied on a target expression.
pub struct PatternMatch<E: RcExpression> {
    map: HashMap<
        u64, // pointer to rule pattern, like #a
        E,   // target expr,             like 10
    >,
}

impl<E: RcExpression> Default for PatternMatch<E> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

pub trait MatchRule<E: RcExpression> {
    /// Pattern matches a rule template against an expression. If successful, the results of the
    /// matching are returned as a mapping of rule to target expressions replacements.
    ///
    /// A sucessful pattern matching is one that matches the target expression wholly, abiding the
    /// expression pattern matching rules.
    fn match_rule(rule: RcExprPat, target: E) -> Option<PatternMatch<E>>;
}

impl MatchRule<RcExpr> for PatternMatch<RcExpr> {
    fn match_rule(rule: RcExprPat, target: RcExpr) -> Option<PatternMatch<RcExpr>> {
        match (rule.as_ref(), target.as_ref()) {
            // The happiest path -- if a pattern matches an expression, return replacements for it!
            (ExprPat::VarPat(_), Expr::Var(_))
            | (ExprPat::ConstPat(_), Expr::Const(_))
            | (ExprPat::AnyPat(_), _) => {
                let mut replacements = PatternMatch::default();
                replacements.insert(&rule, target);
                Some(replacements)
            }
            (ExprPat::Const(a), Expr::Const(b)) => {
                if (a - b).abs() > std::f64::EPSILON {
                    // Constants don't match; rule can't be applied.
                    return None;
                }
                // Constant values are... constant, so there is no need to replace them.
                Some(PatternMatch::default())
            }
            (ExprPat::BinaryExpr(rule), Expr::BinaryExpr(expr)) => {
                if rule.op != expr.op {
                    return None;
                }
                // Expressions are of the same type; match the rest of the expression by recursing on
                // the arguments.
                let replacements_lhs = Self::match_rule(rule.lhs.clone(), expr.lhs.clone())?;
                let replacements_rhs = Self::match_rule(rule.rhs.clone(), expr.rhs.clone())?;
                PatternMatch::try_merge(replacements_lhs, replacements_rhs)
            }
            (ExprPat::UnaryExpr(rule), Expr::UnaryExpr(expr)) => {
                if rule.op != expr.op {
                    return None;
                }
                // Expressions are of the same type; match the rest of the expression by recursing on
                // the argument.
                Self::match_rule(rule.rhs.clone(), expr.rhs.clone())
            }
            (ExprPat::Parend(rule), Expr::Parend(expr)) => {
                Self::match_rule(rule.clone(), expr.clone())
            }
            (ExprPat::Bracketed(rule), Expr::Bracketed(expr)) => {
                Self::match_rule(rule.clone(), expr.clone())
            }
            _ => None,
        }
    }
}

impl MatchRule<RcExprPat> for PatternMatch<RcExprPat> {
    fn match_rule(rule: RcExprPat, target: RcExprPat) -> Option<PatternMatch<RcExprPat>> {
        match (rule.as_ref(), target.as_ref()) {
            (ExprPat::VarPat(_), ExprPat::VarPat(_))
            | (ExprPat::ConstPat(_), ExprPat::ConstPat(_))
            | (ExprPat::AnyPat(_), _) => {
                let mut replacements = PatternMatch::default();
                replacements.insert(&rule, target);
                Some(replacements)
            }
            (ExprPat::Const(a), ExprPat::Const(b)) => {
                if (a - b).abs() > std::f64::EPSILON {
                    return None;
                }
                Some(PatternMatch::default())
            }
            (ExprPat::BinaryExpr(rule), ExprPat::BinaryExpr(expr)) => {
                if rule.op != expr.op {
                    return None;
                }
                let replacements_lhs = Self::match_rule(rule.lhs.clone(), expr.lhs.clone())?;
                let replacements_rhs = Self::match_rule(rule.rhs.clone(), expr.rhs.clone())?;
                PatternMatch::try_merge(replacements_lhs, replacements_rhs)
            }
            (ExprPat::UnaryExpr(rule), ExprPat::UnaryExpr(expr)) => {
                if rule.op != expr.op {
                    return None;
                }
                Self::match_rule(rule.rhs.clone(), expr.rhs.clone())
            }
            (ExprPat::Parend(rule), ExprPat::Parend(expr)) => {
                Self::match_rule(rule.clone(), expr.clone())
            }
            (ExprPat::Bracketed(rule), ExprPat::Bracketed(expr)) => {
                Self::match_rule(rule.clone(), expr.clone())
            }
            _ => None,
        }
    }
}

impl Transformer<RcExprPat, RcExpr> for PatternMatch<RcExpr> {
    /// Transforms a pattern expression into an expression by replacing patterns with target
    /// expressions known by the [`PatternMatch`].
    ///
    /// This transformation can be used to apply a rule on an expression by transforming the RHS
    /// using patterns matched between the LHS of the rule and the target expression.
    ///
    /// [`PatternMatch`]: PatternMatch
    fn transform(&self, item: RcExprPat) -> RcExpr {
        fn transform(
            repls: &PatternMatch<RcExpr>,
            item: RcExprPat,
            cache: &mut HashMap<u64, RcExpr>,
        ) -> RcExpr {
            if let Some(result) = cache.get(&hash(item.as_ref())) {
                return result.clone();
            }

            let og_span = item.span;
            let transformed: RcExpr = match item.as_ref() {
                ExprPat::VarPat(_) | ExprPat::ConstPat(_) | ExprPat::AnyPat(_) => {
                    match repls.map.get(&hash(item.as_ref())) {
                        Some(transformed) => transformed.clone(),

                        // A pattern can only be transformed into an expression if it has an
                        // expression replacement. Patterns are be validated before transformation,
                        // so this branch should never be hit.
                        None => unreachable!(),
                    }
                }

                ExprPat::Const(f) => rc_expr!(Expr::Const(*f), og_span),
                ExprPat::BinaryExpr(binary_expr) => rc_expr!(
                    Expr::BinaryExpr(BinaryExpr {
                        op: binary_expr.op,
                        lhs: transform(repls, binary_expr.lhs.clone(), cache),
                        rhs: transform(repls, binary_expr.rhs.clone(), cache),
                    }),
                    og_span
                ),
                ExprPat::UnaryExpr(unary_expr) => rc_expr!(
                    Expr::UnaryExpr(UnaryExpr {
                        op: unary_expr.op,
                        rhs: transform(repls, unary_expr.rhs.clone(), cache),
                    }),
                    og_span
                ),
                ExprPat::Parend(expr) => {
                    let inner = transform(repls, expr.clone(), cache);
                    rc_expr!(Expr::Parend(inner), og_span)
                }
                ExprPat::Bracketed(expr) => {
                    let inner = transform(repls, expr.clone(), cache);
                    rc_expr!(Expr::Bracketed(inner), og_span)
                }
            };

            let result = cache
                .entry(hash(item.as_ref()))
                .or_insert_with(|| transformed);
            result.clone()
        }

        // Expr pointer -> transformed expression. Assumes that transient expressions of the same
        // value are reference counters pointing to the same underlying expression. This is done
        // via common subexpression elimination during parsing.
        let mut cache = HashMap::new();
        transform(self, item, &mut cache)
    }
}

impl Transformer<RcExprPat, RcExprPat> for PatternMatch<RcExprPat> {
    fn transform(&self, item: RcExprPat) -> RcExprPat {
        fn transform(
            repls: &PatternMatch<RcExprPat>,
            item: RcExprPat,
            cache: &mut HashMap<u64, RcExprPat>,
        ) -> RcExprPat {
            if let Some(result) = cache.get(&hash(item.as_ref())) {
                return result.clone();
            }

            let og_span = item.span;
            let transformed: RcExprPat = match item.as_ref() {
                ExprPat::VarPat(_) | ExprPat::ConstPat(_) | ExprPat::AnyPat(_) => {
                    match repls.map.get(&hash(item.as_ref())) {
                        Some(transformed) => transformed.clone(),
                        None => unreachable!(),
                    }
                }

                ExprPat::Const(f) => rc_expr_pat!(ExprPat::Const(*f), og_span),
                ExprPat::BinaryExpr(binary_expr) => rc_expr_pat!(
                    ExprPat::BinaryExpr(BinaryExpr {
                        op: binary_expr.op,
                        lhs: transform(repls, binary_expr.lhs.clone(), cache),
                        rhs: transform(repls, binary_expr.rhs.clone(), cache),
                    }),
                    og_span
                ),
                ExprPat::UnaryExpr(unary_expr) => rc_expr_pat!(
                    ExprPat::UnaryExpr(UnaryExpr {
                        op: unary_expr.op,
                        rhs: transform(repls, unary_expr.rhs.clone(), cache),
                    }),
                    og_span
                ),
                ExprPat::Parend(expr) => {
                    let inner = transform(repls, expr.clone(), cache);
                    rc_expr_pat!(ExprPat::Parend(inner), og_span)
                }
                ExprPat::Bracketed(expr) => {
                    let inner = transform(repls, expr.clone(), cache);
                    rc_expr_pat!(ExprPat::Bracketed(inner), og_span)
                }
            };

            let result = cache
                .entry(hash(item.as_ref()))
                .or_insert_with(|| transformed);
            result.clone()
        }

        // ExprPat pointer -> transformed expression. Assumes that transient expressions of the same
        // value are reference counters pointing to the same underlying expression. This is done
        // via common subexpression elimination during parsing.
        let mut cache = HashMap::new();
        transform(self, item, &mut cache)
    }
}

impl<E: RcExpression + Eq> PatternMatch<E> {
    /// Merges two `PatternMatch`. If the `PatternMatch` are of incompatible state (i.e. contain
    /// different mappings), merging fails and nothing is returned.
    fn try_merge(left: PatternMatch<E>, right: PatternMatch<E>) -> Option<PatternMatch<E>> {
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

    fn insert(&mut self, k: &RcExprPat, v: E) -> Option<E> {
        self.map.insert(hash(k.as_ref()), v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_expr, parse_expression_pattern, scan};

    fn parse_rule(prog: &str) -> RcExprPat {
        parse_expression_pattern(scan(prog).tokens).program
    }

    mod replacements {
        use super::*;

        #[test]
        fn try_merge() {
            let a = rc_expr_pat!(ExprPat::VarPat("a".into()), crate::DUMMY_SP);
            let b = rc_expr_pat!(ExprPat::VarPat("b".into()), crate::DUMMY_SP);
            let c = rc_expr_pat!(ExprPat::VarPat("c".into()), crate::DUMMY_SP);

            let mut left: PatternMatch<RcExpr> = PatternMatch::default();
            left.insert(&a, rc_expr!(Expr::Const(1.), crate::DUMMY_SP));
            left.insert(&b, rc_expr!(Expr::Const(2.), crate::DUMMY_SP));

            let mut right: PatternMatch<RcExpr> = PatternMatch::default();
            right.insert(&b, rc_expr!(Expr::Const(2.), crate::DUMMY_SP));
            right.insert(&c, rc_expr!(Expr::Const(3.), crate::DUMMY_SP));

            let merged = PatternMatch::try_merge(left, right).unwrap();
            assert_eq!(merged.map.len(), 3);
            assert_eq!(merged.map.get(&hash(a.as_ref())).unwrap().to_string(), "1");
            assert_eq!(merged.map.get(&hash(b.as_ref())).unwrap().to_string(), "2");
            assert_eq!(merged.map.get(&hash(c.as_ref())).unwrap().to_string(), "3");
        }

        #[test]
        fn try_merge_overlapping_non_matching() {
            let a = rc_expr_pat!(ExprPat::VarPat("a".into()), crate::DUMMY_SP);

            let mut left: PatternMatch<RcExpr> = PatternMatch::default();
            left.insert(&a, rc_expr!(Expr::Const(1.), crate::DUMMY_SP));

            let mut right: PatternMatch<RcExpr> = PatternMatch::default();
            right.insert(&a, rc_expr!(Expr::Const(2.), crate::DUMMY_SP));

            let merged = PatternMatch::try_merge(left, right);
            assert!(merged.is_none());
        }
    }

    mod match_rule {
        use super::*;

        macro_rules! match_rule_tests {
            ($($name:ident: $rule:expr => $target:expr => $expected_repls:expr)*) => {
            $(
                #[test]
                fn $name() {
                    let parsed_rule = parse_rule($rule);
                    let parsed_target = parse_expr!($target);

                    let repls = PatternMatch::match_rule(parsed_rule, parsed_target);
                    let (repls, expected_repls): (PatternMatch<RcExpr>, Vec<&str>) =
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
                        .map(|(r, t)| (parse_rule(r), parse_expr!(t)));

                    assert_eq!(expected_repls.len(), repls.map.len());

                    for (expected_pattern, expected_repl) in expected_repls {
                        assert_eq!(
                            expected_repl.to_string(),
                            repls.map.get(&hash(expected_pattern.as_ref())).unwrap().to_string()
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
            parend_on_bracketed:           "($a + #b)" => "[x + 0]" => None

            bracketed:                     "[$a + #b]" => "[x + 0]" => Some(vec!["$a: x", "#b: 0"])
            bracketed_on_parend:           "[$a + #b]" => "(x + 0)" => None
        }
    }
}
