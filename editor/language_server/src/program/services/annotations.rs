//! Module `annotations` provides supplemental annotations over a slide program.

use super::response::*;
use crate::Program;

use libslide::evaluator_rules::Rule;
use libslide::visit::StmtVisitor;
use libslide::*;

impl Program {
    /// Returns relevant annotations to sit atop a slide program.
    pub fn annotations(&self) -> Option<Vec<ProgramAnnotation>> {
        let ast = self.original_ast();
        let mut collect = AnnotationsCollector {
            annotations: vec![],
            context: self.context.as_ref(),
            rules: &self.rules,
        };
        collect.visit_stmt_list(&ast);
        Some(collect.annotations)
    }
}

struct AnnotationsCollector<'a> {
    annotations: Vec<ProgramAnnotation>,
    context: &'a ProgramContext,
    rules: &'a [Rule],
}
impl<'a> StmtVisitor<'a> for AnnotationsCollector<'a> {
    fn visit_expr(&mut self, expr: &'a RcExpr) {
        libslide::visit::descend_expr(self, expr);
        if let Expr::BinaryExpr(..) = expr.as_ref() {
            let simpl = evaluate_expr(expr.clone(), &self.rules, self.context);
            if *expr != simpl {
                self.annotations.push(ProgramAnnotation {
                    span: expr.span,
                    annotation: simpl.to_string(),
                    action: ProgramActionRef {
                        title: simpl.to_string(),
                        handle: String::new(),
                    },
                })
            }
        }
    }
}
