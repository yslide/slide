//! Module `format` provides services for formatting a program.

use super::response::*;
use crate::ast::{get_item_at_span, AstItem};
use crate::Program;

use libslide::{Emit, EmitConfig, Span};

impl Program {
    /// Formats the program.
    pub fn format(&self, config: EmitConfig) -> ProgramTextEdit {
        ProgramTextEdit {
            span: (0, self.end - self.start).into(),
            edit: self.original_ast().emit_pretty(config),
        }
    }

    /// Formats a span in the program, if the span exactly includes something that can be formatted.
    pub fn format_span(&self, span: Span, config: EmitConfig) -> Option<ProgramTextEdit> {
        let edit = match get_item_at_span(span, &self.original_ast())? {
            AstItem::Assignment(asgn) => asgn.emit_pretty(config),
            AstItem::Expr(expr) => expr.emit_pretty(config),
        };
        Some(ProgramTextEdit { span, edit })
    }
}
