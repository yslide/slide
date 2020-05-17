mod types;
pub use types::*;

use crate::evaluator_rules::RuleSet;
use crate::grammar::*;
use crate::utils::hash;

use std::collections::HashSet;
use std::error::Error;
use std::rc::Rc;

/// Evaluates an expression to as simplified a form as possible.
/// The evaluation may be partial, as some values (like variables) may be unknown.
pub fn evaluate(expr: Stmt, ctxt: EvaluatorContext) -> Result<Expr, Box<dyn Error>> {
    let mut rule_set = RuleSet::default();
    for rule in ctxt.rule_blacklist {
        rule_set.remove(rule)
    }
    let built_rules = rule_set.build()?;

    let mut simplified_expr: Rc<Expr> = match expr {
        Stmt::Expr(expr) => expr.into(),
        // TODO: see below
        _ => todo!("Evaluation currently only handles expressions"),
    };

    // Try simplifying the expression with a rule set until the same expression is seen again,
    // meaning we can't simplify any further or are stuck in a cycle.
    let mut expr_hash = hash(&simplified_expr);
    let mut seen: HashSet<u64> = HashSet::new();
    while seen.insert(expr_hash) {
        for rule in &built_rules {
            simplified_expr = rule.transform(simplified_expr);
        }
        expr_hash = hash(&simplified_expr);
    }

    Ok((*simplified_expr).clone())
}

#[cfg(test)]
mod tests {
    use super::evaluate;
    use crate::evaluator_rules::RuleName;
    use crate::grammar::*;
    use crate::{parse_expression, scan, EvaluatorContext};

    fn parse(program: &str) -> Stmt {
        let tokens = scan(program);
        let (parsed, _) = parse_expression(tokens);
        parsed
    }

    macro_rules! partial_evaluator_tests {
        ($($name:ident: $program:expr => $result:expr)*) => {
        $(
            #[test]
            fn $name() {
                let parsed = parse($program);
                let evaluated = evaluate(parsed.clone(), EvaluatorContext::default()).unwrap();

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

        reorder_constants:              "1 + a"     => "a + 1"
        reorder_constants_nested:       "1 + a + 2" => "a + 3"
        reorder_constants_nested_left:  "a + 1 + 2" => "a + 3"
        reorder_constants_nested_right: "1 + 2 + a" => "a + 3"

        distribute_negation:            "-(a - b)"     => "b - a"
        distribute_negation_nested:     "1 + -(a - b)" => "1 + b - a"
        distribute_negation_with_eval:  "1 + -(2 - 3)" => "2"

        unwrap_parens_const:            "(1)"       => "1"
        unwrap_parens_var:              "(a)"       => "a"
        unwrap_parens_nested:           "(a) + (1)" => "a + 1"

        unwrap_braces_const:            "[1]"       => "1"
        unwrap_braces_var:              "[a]"       => "a"
        unwrap_braces_nested:           "[a] + [1]" => "a + 1"
    }

    #[test]
    fn remove_rule() {
        let parsed = parse("1 - 2 + 3 * 4");
        let ctxt = EvaluatorContext::default().with_blacklist([RuleName::Add].to_vec());
        let evaluated = evaluate(parsed, ctxt).unwrap();
        assert_eq!(evaluated.to_string(), "-1 + 12".to_string());
    }
}
