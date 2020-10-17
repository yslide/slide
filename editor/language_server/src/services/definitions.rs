use crate::ast;
use crate::shims::{to_offset, to_range};
use crate::ProgramInfo;

use libslide::collectors::collect_var_asgns;

use tower_lsp::lsp_types::*;

/// Returns all definitions of a variable in a program.
pub(crate) fn get_definitions(
    position: Position,
    program_info: &ProgramInfo,
    supports_link: bool,
) -> Option<GotoDefinitionResponse> {
    let &ProgramInfo {
        uri,
        original: program,
        source,
        ..
    } = &program_info;
    let position = to_offset(&position, &source);
    let tightest_expr = ast::get_tightest_expr(position, &program)?;
    let var = tightest_expr.get_var()?;

    let var_asgns = collect_var_asgns(&program);
    let asgns = var_asgns.get(&var)?;
    let definitions = if supports_link {
        let links = asgns.iter().map(|asgn| {
            let target_range = to_range(&asgn.lhs.span, &source);
            LocationLink {
                origin_selection_range: Some(to_range(&tightest_expr.span, &source)),
                target_uri: uri.clone(),
                target_range,
                target_selection_range: target_range,
            }
        });
        GotoDefinitionResponse::Link(links.collect())
    } else {
        let locs = asgns.iter().map(|asgn| {
            let range = to_range(&asgn.lhs.span, &source);
            Location {
                uri: uri.clone(),
                range,
            }
        });
        GotoDefinitionResponse::Array(locs.collect())
    };

    Some(definitions)
}
