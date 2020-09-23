explain_lint! {
    ///The homogenous assignment lint detects and warns on mixed use of assignment operators.
    ///
    ///For example, the following program uses both the equality and assign-define operators to
    ///assign variables:
    ///
    ///```text
    ///a = 1
    ///b := 1
    ///```
    ///
    ///This can be misleading or confusing, as these two operators are syntactically different (and
    ///semantically different in canonical mathematics notation), but are treated the same in slide.
    ///
    ///For this reason, it is suggested that exclusively `=` or `:=` are used for assignments in
    ///slide programs.
    L0004: HomogenousAssignmentLinter
}

use crate::linter::LintRule;

use crate::diagnostics::Diagnostic;
use crate::grammar::*;

pub struct HomogenousAssignmentLinter<'a> {
    source: &'a str,
    /// The assignment op kind we expect to see across the program, set to the first assignment op
    /// we see.
    asgn_op: Option<AssignmentOp>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> HomogenousAssignmentLinter<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            asgn_op: None,
            diagnostics: vec![],
        }
    }
}

impl<'a> StmtVisitor<'a> for HomogenousAssignmentLinter<'a> {
    fn visit_asgn_op(&mut self, asgn_op: &'a AssignmentOp) {
        match (self.asgn_op, asgn_op) {
            (Some(AssignmentOp::Equal(expected)), AssignmentOp::AssignDefine(actual))
            | (Some(AssignmentOp::AssignDefine(expected)), AssignmentOp::Equal(actual)) => {
                self.diagnostics.push(
                    Diagnostic::span_warn(
                        *actual,
                        "Mixed use of assignment operators",
                        Self::CODE,
                        format!(r#"expected "{}" here"#, expected.over(self.source)),
                    )
                    .with_spanned_note(
                        expected,
                        format!(
                            r#"first use of "{}" as an assignment operator here"#,
                            expected.over(self.source),
                        ),
                    ),
                )
            }
            (None, actual) => self.asgn_op = Some(*actual),
            _ => (),
        }
    }
}

impl<'a> LintRule<'a, StmtList> for HomogenousAssignmentLinter<'a> {
    fn lint(stmt_list: &StmtList, source: &'a str) -> Vec<Diagnostic> {
        let mut linter = Self::new(&source);
        linter.visit(stmt_list);
        linter.diagnostics
    }
}
