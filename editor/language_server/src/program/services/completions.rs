//! Module `completions` provides code completions in a slide program.

use super::response::*;
use super::symbols::{fmt_symbol_info, fmt_var_symbol_definition};
use crate::Program;

use libslide::collectors::collect_var_asgns;

impl Program {
    /// Returns code completions for the context of an offset in a program.
    pub fn completions(&self, _offset: usize) -> Vec<ProgramCompletion> {
        let program = self.original_ast();
        // Currently we just return all variables as completion items.
        collect_var_asgns(&program)
            .into_iter()
            .map(|(var, asgns)| ProgramCompletion {
                label: var.to_string(),
                kind: ProgramCompletionKind::Variable,
                documentation: fmt_symbol_info(fmt_var_symbol_definition(Some(asgns.as_ref()))),
            })
            .collect()
    }
}
