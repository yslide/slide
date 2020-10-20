#![allow(warnings)]

use crate::ast;
use crate::shims::to_offset;
use crate::ProgramInfo;

use libslide::collectors::collect_var_asgns;
use libslide::RcExpression;

use std::collections::BTreeMap;
use tower_lsp::lsp_types::*;

/// Returns completions in context where a variable is expected.
/// Otherwise, no completions are provided.
pub(crate) fn get_completions(
    position: Position,
    program_info: &ProgramInfo,
) -> Option<CompletionResponse> {
    let position = to_offset(&position, &program_info.source);
    let tightest_expr = ast::get_tightest_expr(position, &program_info.original)?;
    if !tightest_expr.is_var() {
        return None;
    }

    let var_asgns = collect_var_asgns(&program_info.simplified)
        .into_iter()
        .collect::<BTreeMap<_, _>>() // sort by key name
        .into_iter()
        .map(|(key, _v)| CompletionItem {
            label: key.to_string(),
            kind: Some(CompletionItemKind::Variable),
            insert_text_format: Some(InsertTextFormat::PlainText),
            ..CompletionItem::default()
        })
        .collect();
    Some(CompletionResponse::Array(var_asgns))
}
