//! Module `rename` provides services for renaming variables, where possible.

use super::response::*;
use crate::ast::get_tightest_expr;
use crate::Program;

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
}
