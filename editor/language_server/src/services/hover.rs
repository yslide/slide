use crate::shims::{to_offset, to_range};
use crate::ProgramInfo;

use collectors::collect_var_asgns;
use libslide::*;

use std::collections::HashSet;
use tower_lsp::lsp_types::*;

/// Returns hover info for an expression.
/// - If the expression is a variable,
///   - if the variable is defined, its simplified definition(s) are returned.
///   - if the variable is not defined, an "unknown" marker is returned.
/// - Otherwise, a simplified version of the hovered expression is returned.
pub fn get_hover_info(
    position: Position,
    program_info: impl std::ops::Deref<Target = ProgramInfo>,
    context: &ProgramContext,
) -> Option<Hover> {
    let position = to_offset(&position, &program_info.source);
    let tightest_expr = match get_tightest_expr(position, &program_info.original) {
        Some(expr) => expr,
        None => return None,
    };
    let range = Some(to_range(&tightest_expr.span, &program_info.source));

    // Now the fun part: actually figure out the hover result.
    let var_asgns = collect_var_asgns(&program_info.simplified);
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

fn get_tightest_expr(pos: usize, program: &StmtList) -> Option<&RcExpr> {
    let mut finder = ExprFinder {
        tightest: None,
        pos,
    };
    finder.visit(program);
    finder.tightest
}

struct ExprFinder<'a> {
    tightest: Option<&'a RcExpr>,
    pos: usize,
}
impl<'a> StmtVisitor<'a> for ExprFinder<'a> {
    fn visit_expr(&mut self, expr: &'a RcExpr) {
        // TODO: skip entire statements outside of position, and do not have to clone visitor impl.
        if expr.span.contains(self.pos) {
            self.tightest = Some(expr);
            match expr.as_ref() {
                Expr::Const(k) => self.visit_const(k, expr.span),
                Expr::Var(v) => self.visit_var(v, expr.span),
                Expr::BinaryExpr(b) => self.visit_binary(b),
                Expr::UnaryExpr(u) => self.visit_unary(u, expr.span),
                Expr::Parend(p) => self.visit_parend(p, expr.span),
                Expr::Bracketed(b) => self.visit_bracketed(b, expr.span),
            }
        }
    }
}
