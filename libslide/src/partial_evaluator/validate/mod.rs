//! Validates that evaluated slide programs are well-formed. In some sense, validations are
//! post-evaluator linters.

mod incompatible_definitions;
use incompatible_definitions::*;

use super::EvaluatorContext;

use crate::diagnostics::Diagnostic;
use crate::evaluator_rules::Rule;
use crate::grammar::StmtList;

trait Validator<'a> {
    fn validate(
        stmt_list: &StmtList,
        source: &'a str,
        context: &EvaluatorContext,
        evaluator_rules: &[Rule],
    ) -> Vec<Diagnostic>;
}

macro_rules! register_validators {
    ($($validator:ident,)*) => {
        enum PEValidator {
            $($validator),*
        }

        impl PEValidator {
            fn validate<'a>(
                &self,
                stmt_list: &StmtList,
                source: &'a str,
                context: &EvaluatorContext,
                evaluator_rules: &[Rule],
            ) -> Vec<Diagnostic> {
                match self {
                    $(Self::$validator => $validator::validate(
                            stmt_list, source, context, evaluator_rules)),*
                }
            }
        }

        pub(super) fn validate<'a>(
            stmt_list: &StmtList,
            source: &'a str,
            context: &EvaluatorContext,
            evaluator_rules: &[Rule],
        ) -> Vec<Diagnostic> {
            [$(PEValidator::$validator),*]
                .iter()
                .flat_map(|v| v.validate(stmt_list, source, context, evaluator_rules))
                .collect()
        }
    }
}

register_validators! {
    IncompatibleDefinitionsValidator,
}
