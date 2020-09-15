//! libslide's heavy-lifting optimizer, applying simplification rules on the libslide IR.

pub mod flatten;
mod types;
use flatten::flatten_expr;
pub use types::*;

use crate::evaluator_rules::{Rule, RuleSet};
use crate::grammar::*;
use crate::utils::{hash, normalize};

use std::collections::HashSet;
use std::error::Error;

/// Evaluates a statement to as simplified a form as possible.
/// The evaluation may be partial, as some values (like variables) may be unknown.
pub fn evaluate(expr: Stmt, ctxt: &EvaluatorContext) -> Result<Stmt, Box<dyn Error>> {
    let mut rule_set = RuleSet::default();
    for rule in &ctxt.rule_denylist {
        rule_set.remove(rule)
    }
    let built_rules = rule_set.build()?;

    Ok(match expr {
        Stmt::Expr(expr) => Stmt::Expr(evaluate_expr(expr, &built_rules, &ctxt)),
        Stmt::Assignment(Assignment { var, rhs: expr }) => Stmt::Assignment(Assignment {
            var,
            rhs: evaluate_expr(expr, &built_rules, &ctxt),
        }),
    })
}

/// Evaluates an expression to as simplified a form as possible.
/// The evaluation may be partial, as some values (like variables) may be unknown.
/// The returned expression is [normalized](crate::utils::normalize).
fn evaluate_expr(expr: InternedExpr, rules: &[Rule], ctxt: &EvaluatorContext) -> InternedExpr {
    let mut simplified_expr = expr;
    // Try simplifying the expression with a rule set until the same expression is seen again,
    // meaning we can't simplify any further or are stuck in a cycle.
    let mut expr_hash = hash(&simplified_expr);
    let mut seen: HashSet<u64> = HashSet::new();
    if ctxt.always_flatten {
        simplified_expr = flatten_expr(simplified_expr);
    }
    while seen.insert(expr_hash) {
        for rule in rules {
            simplified_expr = rule.transform(simplified_expr);
        }
        expr_hash = hash(&simplified_expr);
    }

    normalize(simplified_expr)
}

#[cfg(test)]
mod tests {
    use super::evaluate;
    use crate::evaluator_rules::RuleName;
    use crate::{parse_stmt, EvaluatorContext};

    macro_rules! partial_evaluator_tests {
        ($($name:ident: $program:expr => $result:expr)*) => {
        $(
            #[test]
            fn $name() {
                let parsed = parse_stmt!($program);
                let evaluated = evaluate(parsed.clone(), &EvaluatorContext::default()).unwrap();

                assert_eq!(evaluated.to_string(), $result.to_string());
            }
        )*
        }
    }

    partial_evaluator_tests! {
        add:                            "1 + 2"     => "3"
        add_nested_left:                "1 + 2 + a" => "a + 3"
        add_nested_right:               "a + 1 + 2" => "a + 3"
        add_nested_with_reorder:        "1 + a + 2" => "a + 3"

        sub:                            "1 - 2"     => "-1"
        sub_nested_left:                "1 - 2 - a" => "-1 - a"

        mult:                           "2 * 3"     => "6"
        mult_nested_left:               "2 * 3 * a" => "a * 6"

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
        posate_nested:                  "+(b + c)"     => "b + c"
        posate_nested_right:            "a + +(b + c)" => "a + b + c"
        posate_nested_prec:             "a * +(b + c)" => "a * (b + c)"

        negate:                         "-1"     => "-1"
        negate_nested:                  "1 + -2" => "-1"

        additive_identity_var:          "a + 0"       => "a"
        additive_identity_const:        "1 + 0"       => "1"
        additive_identity_any:          "(a * b) + 0" => "a * b"
        additive_identity_nested:       "(a + 0) + 0" => "a"
        additive_identity_with_reorder: "0 + a + 0"   => "a"

        additive_inverse_var:          "a - a"             => "0"
        additive_inverse_const:        "1 - 1"             => "0"
        additive_inverse_any:          "(a * b) - (a * b)" => "0"
        additive_inverse_nested:       "(a + 0) - a"       => "0"
        additive_inverse_with_reorder: "a + 0 - a"         => "0"

        subtractive_identity_var:          "a - 0"       => "a"
        subtractive_identity_const:        "1 - 0"       => "1"
        subtractive_identity_any:          "(a * b) - 0" => "a * b"
        subtractive_identity_nested:       "(a + 0) - 0" => "a"
        subtractive_identity_with_reorder: "0 - a - 0"   => "-a"

        reorder_constants:              "1 + a"     => "a + 1"
        reorder_constants_nested:       "1 + a + 2" => "a + 3"
        reorder_constants_nested_left:  "a + 1 + 2" => "a + 3"
        reorder_constants_nested_right: "1 + 2 + a" => "a + 3"

        distribute_negation:            "-(a - b)"     => "b - a"
        distribute_negation_nested:     "1 + -(a - b)" => "b + 1 - a"
        distribute_negation_with_eval:  "1 + -(2 - 3)" => "2"

        unwrap_parens_const:            "(1)"       => "1"
        unwrap_parens_var:              "(a)"       => "a"
        unwrap_parens_nested:           "(a) + (1)" => "a + 1"

        unwrap_brackets_const:            "[1]"       => "1"
        unwrap_brackets_var:              "[a]"       => "a"
        unwrap_brackets_nested:           "[a] + [1]" => "a + 1"

        flattened_addition:             "1 + 2 - b + 3 - b" => "6 - b - b"

        issue_92: "a + 1 - 1" => "a"
    }

    #[test]
    fn remove_rule() {
        let parsed = parse_stmt!("1 - 2 + 3 * 4");
        let ctxt = EvaluatorContext::default()
            .with_denylist([RuleName::Add].to_vec())
            .always_flatten(false);
        let evaluated = evaluate(parsed, &ctxt).unwrap();
        assert_eq!(evaluated.to_string(), "-1 + 12".to_string());
    }
}
