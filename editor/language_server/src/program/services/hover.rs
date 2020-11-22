//! Module `hover` provides hover services for a slide langauge server.

use super::response::*;
use crate::ast;
use crate::Program;

use collectors::collect_var_asgns;
use libslide::*;

use std::collections::HashSet;
use tower_lsp::lsp_types::*;

impl Program {
    /// Returns hover info for an expression.
    /// - If the expression is a variable,
    ///   - if the variable is defined, its simplified definition(s) are returned.
    ///   - if the variable is not defined, an "unknown" marker is returned.
    /// - Otherwise, a simplified version of the hovered expression is returned.
    pub fn get_hover_info(&self, offset: usize) -> Option<ProgramHoverResponse> {
        let program_ast = self.original_ast();
        let tightest_expr = ast::get_tightest_expr(offset, &program_ast)?;
        let span = tightest_expr.span;

        // Now the fun part: actually figure out the hover result.
        let simplified_ast = self.simplified_ast();
        let var_asgns = collect_var_asgns(&simplified_ast);
        let simplified = if let Some(var) = tightest_expr.get_var() {
            // A variable - get its definitions from its assignments.
            match var_asgns.get(&var) {
                Some(asgns) => fmt_asgn_definitions(asgns),
                None => "???".to_string(),
            }
        } else {
            // A subexpression - simplify it.
            // TODO: we only need to build rules once.
            let rules = build_rules(self.context.as_ref()).ok()?;
            evaluate_expr(tightest_expr.clone(), &rules, self.context.as_ref()).to_string()
        };
        let hover_info = fmt_hover_info(simplified);

        Some(ProgramHoverResponse {
            contents: HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                language: "math".to_string(),
                value: hover_info,
            })),
            span,
        })
    }
}

fn fmt_asgn_definitions(asgns: &[&Assignment]) -> String {
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

fn fmt_hover_info(simplified_vals: String) -> String {
    simplified_vals
        .lines()
        .map(|l| format!("= {}", l))
        .collect::<Vec<_>>()
        .join("\n")
}
