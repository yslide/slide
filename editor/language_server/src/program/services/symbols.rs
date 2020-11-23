//! Module `symbols` provides information about symbols in a program.

use super::response::*;
use crate::Program;

use libslide::collectors::collect_var_asgns;
use libslide::*;

use std::collections::HashSet;

impl Program {
    /// Returns information about symbols (variables) in a slide program.
    ///
    /// If `matching` is specified, only symbols whose names including `matching` are returned.
    pub fn get_symbols(&self, matching: Option<&str>) -> Vec<ProgramSymbolInformation> {
        let ast = self.simplified_ast();
        let var_asgns = collect_var_asgns(&ast);
        var_asgns
            .into_iter()
            .filter(|(var, _)| match matching {
                Some(matcher) => var.as_ref().contains(&matcher),
                None => true,
            })
            .map(|(var, definitions)| ProgramSymbolInformation {
                name: var.to_string(),
                kind: ProgramSymbolKind::Variable,
                documentation: fmt_symbol_info(fmt_var_symbol_definition(Some(
                    definitions.as_ref(),
                ))),
                location: ProgramLocation {
                    uri: (*self.document_uri).clone(),
                    span: definitions[0].lhs.span,
                },
            })
            .collect()
    }
}

/// Pretty-formats a [variable](Expr::Var) symbol's definition given its assignment values.
pub fn fmt_var_symbol_definition(asgns: Option<&[&Assignment]>) -> String {
    match asgns {
        Some(asgns) => {
            let mut seen = HashSet::new();
            asgns
                .iter()
                .filter_map(|asgn| {
                    if seen.contains(&asgn.rhs) {
                        return None;
                    }
                    seen.insert(&asgn.rhs);
                    Some(asgn.rhs.to_string())
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        None => "???".to_owned(),
    }
}

/// Pretty-formats definition information for a symbol.
pub fn fmt_symbol_info(definition: String) -> String {
    definition
        .lines()
        .map(|l| format!("= {}", l))
        .collect::<Vec<_>>()
        .join("\n")
}
