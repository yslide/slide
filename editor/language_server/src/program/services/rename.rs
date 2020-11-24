//! Module `rename` provides services for renaming variables, where possible.

use super::response::*;
use crate::ast::get_tightest_expr;
use crate::Program;

use libslide::visit::StmtVisitor;
use libslide::{InternedStr, Span};

impl Program {
    /// Returns the range and placeholder for an item to rename, if it can be renamed. If it cannot
    /// be renamed, a reason as to why is returned via an error.
    pub fn can_rename(
        &self,
        offset: usize,
    ) -> Result<ProgramCanRenameResponse, ProgramCannotRenameBecause> {
        let ast = self.original_ast();
        let expr = get_tightest_expr(offset, &ast)
            .ok_or(ProgramCannotRenameBecause::CursorNotOverVariable)?;
        if expr.is_var() {
            Ok(ProgramCanRenameResponse {
                span: expr.span,
                placeholder: "new variable".to_owned(),
            })
        } else {
            Err(ProgramCannotRenameBecause::CursorNotOverVariable)
        }
    }

    /// Retrieves edits to rename a variable across a program.
    pub fn get_rename_edits(&self, offset: usize, rename: String) -> Option<ProgramRenameResponse> {
        let ast = self.original_ast();
        // Only variables can be renamed.
        let orig_var_name = get_tightest_expr(offset, &ast)?.get_var()?;

        let mut collector = NamedVarCollector {
            name: orig_var_name,
            locations: vec![],
        };
        collector.visit_stmt_list(&ast);

        Some(ProgramRenameResponse {
            uri: (*self.document_uri).clone(),
            edits: collector
                .locations
                .into_iter()
                .map(|span| ProgramTextEdit {
                    span,
                    edit: rename.to_string(),
                })
                .collect(),
        })
    }
}

struct NamedVarCollector {
    name: InternedStr,
    locations: Vec<Span>,
}
impl<'a> StmtVisitor<'a> for NamedVarCollector {
    fn visit_var(&mut self, &var: &'a InternedStr, span: Span) {
        if var == self.name {
            self.locations.push(span);
        }
    }
}
