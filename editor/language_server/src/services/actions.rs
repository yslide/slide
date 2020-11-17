use crate::ast;
use crate::shims::to_span;
use crate::ProgramInfo;

use libslide::*;

use tower_lsp::lsp_types::*;

/// Returns possible code actions at a location.
/// To actually execute the action, [`exec_action`](exec_action) should be used.
pub(crate) fn get_action_commands(
    range: Range,
    program_info: &ProgramInfo,
    // TODO: use this
    _existing_diagnostics: &[Diagnostic],
) -> Option<CodeActionResponse> {
    let span = to_span(&range, &program_info.source);
    // NB: we should actually consider the entire range here.
    let node = ast::get_tightest_expr(span.lo, &program_info.original)?;

    match node.as_ref() {
        Expr::Const(_) | Expr::Var(_) => None,
        Expr::BinaryExpr(_) | Expr::UnaryExpr(_) | Expr::Parend(_) | Expr::Bracketed(_) => {
            Some(vec![Act::simplify_expr()])
        }
    }
}

struct Act;
impl Act {
    fn simplify_expr() -> CodeActionOrCommand {
        let command = Cmd::SimplifyExpr.into_command();
        CodeActionOrCommand::CodeAction(CodeAction {
            // Use exec_action to get the actual edit.
            edit: None,
            title: command.title.clone(),
            kind: Some(CodeActionKind::REFACTOR_REWRITE),
            diagnostics: None,
            command: Some(command),
            is_preferred: Some(true),
        })
    }
}

enum Cmd {
    SimplifyExpr,
}
impl Cmd {
    fn into_command(&self) -> Command {
        match &self {
            Cmd::SimplifyExpr => Command {
                title: "Simplify expression".to_owned(),
                command: "slide::simplify_expr".to_owned(),
                arguments: None,
            },
        }
    }
}

#[allow(unused)]
pub(crate) fn exec_action() {
    todo!();
}
