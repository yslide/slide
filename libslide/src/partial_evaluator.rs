use crate::evaluator_rules::RuleSet;
use crate::grammar::*;

use core::hash::{Hash, Hasher};
use std::collections::{hash_map::DefaultHasher, HashSet};

/// Evaluates an expression to as simplified a form as possible.
/// The evaluation may be partial, as some values (like variables) may be unknown.
pub fn evaluate(expr: Stmt) -> Expr {
    let rule_set = RuleSet::default();
    let built_rules = rule_set.build();

    let mut simplified_expr: Expr = match expr {
        Stmt::Expr(expr) => expr,
        _ => todo!("Evaluation currently only handles expressions"),
    };

    // Try simplifying the expression with a rule set until the same expression is seen again,
    // meaning we can't simplify any further or are stuck in a cycle.
    let mut expr_hash = hash_expr(&simplified_expr);
    let mut seen: HashSet<u64> = HashSet::new();
    while seen.insert(expr_hash) {
        for rule in &built_rules {
            simplified_expr = rule.transform(simplified_expr);
        }
        expr_hash = hash_expr(&simplified_expr);
    }

    simplified_expr
}

fn hash_expr(expr: &Expr) -> u64 {
    // There is no way to reset a hasher's state, so we create a new one each time.
    let mut hasher = DefaultHasher::new();
    expr.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    macro_rules! partial_evaluator_tests {
        ($($name:ident: $program:expr => $result:expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::{parse_expression, scan};
                use super::evaluate;

                let tokens = scan($program);
                let (parsed, _) = parse_expression(tokens);
                let evaluated = evaluate(parsed);
                assert_eq!(evaluated.to_string(), $result.to_string());
            }
        )*
        }
    }

    partial_evaluator_tests! {
        add:                            "1 + 2"     => "3"
        add_nested_left:                "1 + 2 + a" => "a + 3"
        // TODO: This doesn't work yet... we need to be smarter, maybe by rewritting associativity
        // or flipping constants between passes.
        // add_nested_right:               "a + 1 + 2" => "a + 3"
        // add_nested_with_reorder:        "1 + a + 2" => "a + 3"

        sub:                            "1 - 2"     => "-1"
        sub_nested_left:                "1 - 2 - a" => "-1 - a"

        mult:                           "2 * 3"     => "6"
        mult_nested_left:               "2 * 3 * a" => "6 * a"

        div:                            "6 / 2"     => "3"
        div_nested_left:                "6 / 2 / a" => "3 / a"
        div_associated:                 "6 / 2 / 3" => "1"

        modulo:                         "6 % 4"     => "2"
        modulo_nested_left:             "6 % 4 % a" => "2 % a"
        modulo_associated:              "9 % 5 % 5" => "4" // (9 % 5) % 5

        exp:                            "2 ^ 3"     => "8"
        exp_nested_left:                "2 ^ 3 ^ a" => "2 ^ 3 ^ a"
        exp_associated:                 "2 ^ 3 ^ 2" => "512"

        posate:                         "+1"           => "1"
        // TODO: unwrap parantheses further
        posate_nested_1:                "+(b + c)" => "(b + c)"
        posate_nested:                  "a + +(b + c)" => "a + (b + c)"

        negate:                         "-1"     => "-1"
        negate_nested:                  "1 + -2" => "-1"

        additive_identity_var:          "a + 0"       => "a"
        additive_identity_const:        "1 + 0"       => "1"
        additive_identity_any:          "(a * b) + 0" => "(a * b)"
        additive_identity_nested:       "(a + 0) + 0" => "a"
        additive_identity_with_reorder: "0 + a + 0"   => "a"

        reorder_constants:              "1 + a"     => "a + 1"
        reorder_constants_nested:       "1 + a + 2" => "a + 1 + 2"
        reorder_constants_nested_left:  "a + 1 + 2" => "a + 1 + 2"
        reorder_constants_nested_right: "1 + 2 + a" => "a + 3"

        unwrap_parens_const:            "(1)"       => "1"
        unwrap_parens_var:              "(a)"       => "a"
        unwrap_parens_nested:           "(a) + (1)" => "a + 1"

        unwrap_braces_const:            "[1]"       => "1"
        unwrap_braces_var:              "[a]"       => "a"
        unwrap_braces_nested:           "[a] + [1]" => "a + 1"
    }
}
