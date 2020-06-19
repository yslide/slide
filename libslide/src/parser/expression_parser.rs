use super::{extra_tokens_diag, Parser};
use crate::common::Span;
use crate::diagnostics::Diagnostic;
use crate::grammar::*;
use crate::scanner::types::{Token, TokenType};
use crate::utils::{hash, PeekIter, StringUtils};

use std::collections::HashMap;
use std::rc::Rc;

// pub struct ScanResult {
//     pub tokens: Vec<Token>,
//     pub diagnostics: Vec<Diagnostic>,
// }

pub fn parse(input: Vec<Token>) -> (Stmt, Vec<Diagnostic>) {
    let mut parser = ExpressionParser::new(input);
    (parser.parse(), parser.diagnostics)
}

pub struct ExpressionParser {
    _input: PeekIter<Token>,
    diagnostics: Vec<Diagnostic>,
    // We use an untyped hash here because we don't want to clone an Expr into the map in case it's
    // already there when using an entry API.
    // TODO: replace with Expr when raw_entry API is stabilized (see rust#56167)
    seen: HashMap<u64, Rc<Expr>>,
}

impl ExpressionParser {
    fn assignment(&mut self, var: String) -> Stmt {
        Stmt::Assignment(Assignment {
            var,
            rhs: self.expr(),
        })
    }
}

impl ExpressionParser {
    fn parse_pattern(&mut self, name: String, span: Span) -> Expr {
        self.push_diag(
            Diagnostic::span_err(
                span,
                "Patterns cannot be used in an expression",
                Some("unexpected pattern".into()),
            )
            .with_help(format!(
                r#"consider using "{cut_name}" as a variable"#,
                cut_name = name.substring(1, name.len() - 1)
            )),
        );
        Expr::Var(name)
    }
}

impl Parser<Stmt> for ExpressionParser {
    type Expr = Expr;

    fn new(input: Vec<Token>) -> Self {
        Self {
            _input: PeekIter::new(input.into_iter()),
            diagnostics: vec![],
            seen: HashMap::new(),
        }
    }

    fn input(&mut self) -> &mut PeekIter<Token> {
        &mut self._input
    }

    fn push_diag(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn parse(&mut self) -> Stmt {
        let mut next_2 = self.input().peek_map_n(2, |tok| tok.ty.clone());
        let parsed = match (next_2.pop_front(), next_2.pop_front()) {
            (Some(TokenType::Variable(name)), Some(TokenType::Equal)) => {
                self.input().next();
                self.input().next();
                self.assignment(name)
            }
            _ => Stmt::Expr((*self.expr()).clone()),
        };
        if !self.done() {
            let extra_tokens_diag = extra_tokens_diag(self.input());
            self.push_diag(extra_tokens_diag);
        }
        parsed
    }

    fn parse_float(&mut self, f: f64, _span: Span) -> Self::Expr {
        Self::Expr::Const(f)
    }

    fn parse_variable(&mut self, name: String, _span: Span) -> Self::Expr {
        Expr::Var(name)
    }

    fn parse_var_pattern(&mut self, name: String, span: Span) -> Self::Expr {
        self.parse_pattern(name, span)
    }

    fn parse_const_pattern(&mut self, name: String, span: Span) -> Self::Expr {
        self.parse_pattern(name, span)
    }

    fn parse_any_pattern(&mut self, name: String, span: Span) -> Self::Expr {
        self.parse_pattern(name, span)
    }

    fn finish_expr(&mut self, expr: Self::Expr) -> Rc<Self::Expr> {
        let p = self
            .seen
            .entry(hash(&expr))
            .or_insert_with(|| Rc::new(expr));
        Rc::clone(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan;

    parser_tests! {
        parse_expression

        variable:                "a"
        variable_in_op_left:     "a + 1"
        variable_in_op_right:    "1 + a"
        assignment_op:           "a = 5"
        assignment_op_expr:      "a = 5 + 2 ^ 3"
    }

    #[test]
    fn common_subexpression_elimination() {
        let program = "1 * 2 + 1 * 2";
        let tokens = scan(program).tokens;
        let (parsed, _) = parse(tokens);
        let (l, r) = match parsed {
            Stmt::Expr(Expr::BinaryExpr(BinaryExpr { lhs, rhs, .. })) => (lhs, rhs),
            _ => unreachable!(),
        };
        assert!(std::ptr::eq(l.as_ref(), r.as_ref())); // 1 * 2

        let (ll, lr, rl, rr) = match (l.as_ref(), r.as_ref()) {
            (
                Expr::BinaryExpr(BinaryExpr {
                    lhs: ll, rhs: lr, ..
                }),
                Expr::BinaryExpr(BinaryExpr {
                    lhs: rl, rhs: rr, ..
                }),
            ) => (ll, lr, rl, rr),
            _ => unreachable!(),
        };
        assert!(std::ptr::eq(ll.as_ref(), rl.as_ref())); // 1
        assert!(std::ptr::eq(lr.as_ref(), rr.as_ref())); // 2
    }
}
