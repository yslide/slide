use crate::ast;
use crate::shims::{to_offset, to_range};
use crate::Program;

use collectors::collect_var_asgns;
use libslide::*;

use std::collections::HashSet;
use tower_lsp::lsp_types::*;

/// Returns hover info for an expression.
/// - If the expression is a variable,
///   - if the variable is defined, its simplified definition(s) are returned.
///   - if the variable is not defined, an "unknown" marker is returned.
/// - Otherwise, a simplified version of the hovered expression is returned.
pub(crate) fn get_hover_info(
    position: Position,
    program: &Program,
    context: &ProgramContext,
) -> Option<Hover> {
    let position = to_offset(&position, &program.source);

    let program_ast = program.original_ast();
    let tightest_expr = ast::get_tightest_expr(position, &program_ast)?;
    let range = Some(to_range(&tightest_expr.span, &program.source));

    // Now the fun part: actually figure out the hover result.
    let simplified_ast = program.simplified_ast();
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
        let rules = build_rules(context).ok()?;
        evaluate_expr(tightest_expr.clone(), &rules, context).to_string()
    };
    let hover_info = fmt_hover_info(simplified);

    Some(Hover {
        contents: HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
            language: "slide".to_string(),
            value: hover_info,
        })),
        range,
    })
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
