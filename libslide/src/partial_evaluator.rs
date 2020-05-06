use crate::evaluator_rules::*;
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
            simplified_expr = rule.try_apply(&simplified_expr).unwrap_or(simplified_expr);
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
        ($($name:ident: $program:expr, $result:expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::{parse, scan, ScannerOptions};
                use super::evaluate;

                let tokens = scan($program, ScannerOptions::default());
                let parsed = parse(tokens);
                let evaluated = evaluate(parsed);
                assert_eq!(evaluated.to_string(), $result.to_string());
            }
        )*
        }
    }

    partial_evaluator_tests! {
        var_plus_zero: "a + 0", "a"
    }
}
