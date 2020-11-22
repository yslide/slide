//! Module `definitions` serves definitions queries for a slide langauge server.

use super::local_response::*;
use crate::ast;
use crate::Program;

use libslide::collectors::collect_var_asgns;

impl Program {
    /// Returns all definitions of a variable in a program.
    pub fn get_definitions(
        &self,
        offset: usize,
        supports_link: bool,
    ) -> Option<LocalDefinitionResponse> {
        let uri = self.document_uri.as_ref();
        let program = self.original_ast();
        let tightest_expr = ast::get_tightest_expr(offset, &program)?;
        let var = tightest_expr.get_var()?;

        let var_asgns = collect_var_asgns(&program);
        let asgns = var_asgns.get(&var)?;
        let definitions = if supports_link {
            let links = asgns.iter().map(|asgn| {
                let target_span = asgn.lhs.span;
                LocalLocationLink {
                    origin_selection_span: tightest_expr.span,
                    target_uri: uri.clone(),
                    target_span,
                    target_selection_span: target_span,
                }
            });
            LocalDefinitionResponse::Link(links.collect())
        } else {
            let locs = asgns.iter().map(|asgn| {
                let span = asgn.lhs.span;
                LocalLocation {
                    uri: uri.clone(),
                    span,
                }
            });
            LocalDefinitionResponse::Array(locs.collect())
        };

        Some(definitions)
    }
}
