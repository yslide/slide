//! Module `actions` provides code rewrite actions for a slide program.

use super::response::*;
use crate::ast::*;
use crate::Program;

use libslide::*;
use tower_lsp::lsp_types::Url;

impl Program {
    /// Determines rewrite actions applicable over a span in a program.
    pub fn actions(&self, span: Span) -> Vec<ProgramAction> {
        let mut actions = self.diagnostic_actions(span);
        if let Some(rewrite) = self.rewrite_action(span) {
            actions.push(rewrite);
        }
        actions
    }

    /// Retrieves program actions for diagnostics over a span in a program.
    fn diagnostic_actions(&self, for_span: Span) -> Vec<ProgramAction> {
        let diagnostics = self.diagnostics();
        diagnostics
            .iter()
            .filter(|ProgramDiagnostic { span, .. }| span.intersects(for_span))
            .filter_map(|diag| diagnostic2action(diag, self.document_uri.as_ref()))
            .collect()
    }

    /// Retrieves a rewrite action for the item covering the span, if there is such an item and it
    /// has any rewrite.
    fn rewrite_action(&self, span: Span) -> Option<ProgramAction> {
        let ast = self.original_ast();
        // TODO: only need to build rules once.
        let rules = build_rules(self.context.as_ref()).ok()?;
        let simplify_expr = |e| evaluate_expr(e, rules.as_ref(), self.context.as_ref());
        let (span, original, simplified) = match get_item_at_span(span, &ast)? {
            AstItem::Expr(e) => (e.span, e.to_string(), simplify_expr(e.clone()).to_string()),
            AstItem::Assignment(a) => (
                a.span,
                a.to_string(),
                a.clone().redefine_with(simplify_expr).to_string(),
            ),
        };

        if original == simplified {
            None
        } else {
            Some(ProgramAction {
                title: "Simplify".to_owned(),
                kind: ProgramActionKind::Rewrite,
                resolved_diagnostic: None,
                uri: self.document_uri.as_ref().clone(),
                edit: ProgramTextEdit {
                    span,
                    edit: simplified,
                },
                is_preferred: true,
            })
        }
    }
}

fn diagnostic2action(diag: &ProgramDiagnostic, document_uri: &Url) -> Option<ProgramAction> {
    let autofix = diag.autofix.as_ref()?;
    let edit = match &autofix.fix {
        diagnostics::Edit::Delete => "".to_owned(),
        diagnostics::Edit::Replace(s) => s.to_owned(),
    };
    let is_preferred = matches!(autofix.confidence, diagnostics::AutofixConfidence::ForSure);

    let action = ProgramAction {
        title: format!("Fix: {}", diag.title),
        kind: ProgramActionKind::DiagnosticFix,
        resolved_diagnostic: Some(diag.clone()),
        uri: document_uri.clone(),
        edit: ProgramTextEdit {
            span: diag.span,
            edit,
        },
        is_preferred,
    };
    Some(action)
}
