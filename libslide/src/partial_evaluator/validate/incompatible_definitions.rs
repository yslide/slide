//! Detects incompatible variable definitions in a slide program.
//!
//! See the [`IncompatibleDefinitions`](crate::partial_evaluator::errors::IncompatibleDefinitions)
//! error for more details.

use super::Validator;

use crate::diagnostics::Diagnostic;
use crate::evaluator_rules::Rule;
use crate::grammar::*;
use crate::partial_evaluator::{evaluate_expr, EvaluatorContext};

use std::collections::HashMap;

/// Max number of definition pairs we generate diagnostics for.
///
/// We set a hard bound on this because generating information for a tremendous amount of
/// incompatible definitions is not very useful to a user, and is very costly because the partial
/// evaluator is run over each definition pair.
static MAX_DEFINITION_PAIRS: usize = 100;

#[derive(Default)]
pub(super) struct IncompatibleDefinitionsValidator<'a> {
    asgns: HashMap<InternedStr, Vec<&'a Assignment>>,
}

impl<'a> IncompatibleDefinitionsValidator<'a> {
    fn all_ordered_definition_pairs(self) -> Vec<(&'a Assignment, &'a Assignment)> {
        let mut definition_pairs = Vec::new();
        for (_name, asgns) in self.asgns.into_iter() {
            if asgns.len() < 2 {
                continue;
            }
            if definition_pairs.len() > MAX_DEFINITION_PAIRS {
                break;
            }
            let mut pairs = Vec::with_capacity((asgns.len() * asgns.len() - 1) / 2);
            for i in 0..asgns.len() {
                for j in i + 1..asgns.len() {
                    pairs.push((asgns[i], asgns[j]));
                }
            }
            definition_pairs.extend(pairs);
        }
        while definition_pairs.len() > MAX_DEFINITION_PAIRS {
            definition_pairs.pop();
        }
        definition_pairs.sort_by(|&(a, c), &(b, d)| {
            if a.span == b.span {
                c.span.cmp(&d.span)
            } else {
                a.span.cmp(&b.span)
            }
        });
        definition_pairs
    }

    fn validate(self, context: &EvaluatorContext, evaluator_rules: &[Rule]) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let definition_pairs = self.all_ordered_definition_pairs();
        for (def_a, def_b) in definition_pairs.into_iter() {
            let diff = rc_expr!(
                Expr::BinaryExpr(BinaryExpr::sub(def_a.rhs.clone(), def_b.rhs.clone())),
                crate::DUMMY_SP
            );
            let diff = evaluate_expr(diff, evaluator_rules, context);
            match diff.get_const() {
                None => continue,
                Some(e) if e == &0 => continue,
                Some(_) => diagnostics.push(IncompatibleDefinitions!(def_a.var, def_a, def_b)),
            }
        }
        diagnostics
    }
}

impl<'a> StmtVisitor<'a> for IncompatibleDefinitionsValidator<'a> {
    fn visit_asgn(&mut self, asgn: &'a Assignment) {
        self.asgns.entry(asgn.var).or_default().push(asgn);
    }
}

impl<'a> Validator<'a> for IncompatibleDefinitionsValidator<'a> {
    fn validate(
        stmt_list: &StmtList,
        _source: &'a str,
        context: &EvaluatorContext,
        evaluator_rules: &[Rule],
    ) -> Vec<Diagnostic> {
        let mut validator = IncompatibleDefinitionsValidator::default();
        validator.visit(stmt_list);
        validator.validate(context, evaluator_rules)
    }
}
