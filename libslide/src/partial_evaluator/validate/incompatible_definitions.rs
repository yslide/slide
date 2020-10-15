//! Detects incompatible variable definitions in a slide program.
//!
//! See the [`IncompatibleDefinitions`](super::super::errors::IncompatibleDefinitions)
//! error for more details.

use super::Validator;

use crate::diagnostics::Diagnostic;
use crate::evaluator_rules::Rule;
use crate::grammar::collectors::collect_var_asgns;
use crate::grammar::*;
use crate::partial_evaluator::compare::{cmp_eq, EqRelation};
use crate::ProgramContext;

use std::collections::{BTreeSet, HashMap};

/// Max number of definition pairs we generate diagnostics for.
///
/// We set a hard bound on this because generating information for a tremendous amount of
/// incompatible definitions is not very useful to a user, and is very costly because the partial
/// evaluator is run over each definition pair.
static MAX_DEFINITION_PAIRS: usize = 100;

fn all_ordered_definition_pairs(
    var_asgns: HashMap<InternedStr, Vec<&Assignment>>,
) -> Vec<(InternedStr, &Assignment, &Assignment)> {
    let mut definition_pairs = Vec::new();
    for (name, asgns) in var_asgns.into_iter() {
        if asgns.len() < 2 {
            continue;
        }
        if definition_pairs.len() > MAX_DEFINITION_PAIRS {
            break;
        }
        let mut pairs = Vec::with_capacity((asgns.len() * asgns.len() - 1) / 2);
        for i in 0..asgns.len() {
            for j in i + 1..asgns.len() {
                pairs.push((name, asgns[i], asgns[j]));
            }
        }
        definition_pairs.extend(pairs);
    }
    while definition_pairs.len() > MAX_DEFINITION_PAIRS {
        definition_pairs.pop();
    }
    definition_pairs.sort_by(|&(_, a, c), &(_, b, d)| {
        if a.span == b.span {
            c.span.cmp(&d.span)
        } else {
            a.span.cmp(&b.span)
        }
    });
    definition_pairs
}

fn validate(
    program: &StmtList,
    context: &ProgramContext,
    evaluator_rules: &[Rule],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let var_asgns = collect_var_asgns(&program);
    let definition_pairs = all_ordered_definition_pairs(var_asgns);
    for (name, def_a, def_b) in definition_pairs.into_iter() {
        diagnostics.push(
            match cmp_eq(&def_a.rhs, &def_b.rhs, evaluator_rules, context) {
                EqRelation::AlwaysEquivalent => continue,
                EqRelation::NeverEquivalent => IncompatibleDefinitions!(name, def_a, def_b),
                EqRelation::DependsOn(_) if !context.lint => continue,
                EqRelation::DependsOn(dep_vars) => {
                    let dep_vars = dep_vars // sort the vars deterministically
                        .into_iter()
                        .map(|v| v.to_string())
                        .collect::<BTreeSet<_>>();
                    MaybeIncompatibleDefinitions!(name, def_a, def_b, dep_vars)
                }
            },
        );
    }
    diagnostics
}

pub(super) struct IncompatibleDefinitionsValidator;
impl<'a> Validator<'a> for IncompatibleDefinitionsValidator {
    fn validate(
        stmt_list: &StmtList,
        _source: &'a str,
        context: &ProgramContext,
        evaluator_rules: &[Rule],
    ) -> Vec<Diagnostic> {
        validate(stmt_list, context, evaluator_rules)
    }
}
