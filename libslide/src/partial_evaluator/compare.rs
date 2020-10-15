//! Module `compare` compares expressions for equality.

use crate::evaluator_rules::Rule;
use crate::grammar::collectors::collect_var_names;
use crate::grammar::{BinaryExpr, Expr, RcExpr};
use crate::partial_evaluator::evaluate_expr;
use crate::{InternedStr, ProgramContext};

use std::collections::HashSet;

/// Describes the equivalence relation between two expressions.
pub enum EqRelation {
    /// The expressions are always equivalent.
    AlwaysEquivalent,
    /// The expressions are never equivalent.
    NeverEquivalent,
    /// The equality of the two expressions depends on some variables.
    DependsOn(HashSet<InternedStr>),
}

/// Compares two expressions for equivalence, returning an [`EqRelation`](self::EqRelation).
pub fn cmp_eq(
    a: &RcExpr,
    b: &RcExpr,
    evaluator_rules: &[Rule],
    context: &ProgramContext,
) -> EqRelation {
    let diff = rc_expr!(
        Expr::BinaryExpr(BinaryExpr::sub(a.clone(), b.clone())),
        crate::DUMMY_SP
    );
    let diff = evaluate_expr(diff, evaluator_rules, context);
    match diff.get_const() {
        Some(e) if e.abs() <= std::f64::EPSILON => EqRelation::AlwaysEquivalent,

        // Difference is a non-zero constant; expressions are never equal.
        Some(_) => EqRelation::NeverEquivalent,

        // Difference is variable-dependent; equality is variable-dependent.
        None => EqRelation::DependsOn(collect_var_names(&diff)),
    }
}
