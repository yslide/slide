use super::Parser;
use crate::grammar::*;
use crate::scanner::types::{Token, TokenType};
use crate::utils::{hash, PeekIter, StringUtils};

use std::collections::HashMap;
use std::rc::Rc;

pub fn parse(input: Vec<Token>) -> (Stmt, Vec<String>) {
    let mut parser = ExpressionParser::new(input);
    let parsed = parser.parse();
    let errors = parser.errors().iter().map(|e| e.to_string()).collect();
    (parsed, errors)
}

pub struct ExpressionParser {
    _input: PeekIter<Token>,
    _errors: Vec<String>,
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
    fn parse_pattern(&mut self, name: String) -> Expr {
        self._errors.push(format!(
            concat!(
                r#"The pattern "{name}" cannot be used in an expression. "#,
                r#"Consider using "{cut_name}" as a variable instead."#
            ),
            name = name,
            cut_name = name.to_string().substring(1, name.len() - 1),
        ));
        Expr::Var(name)
    }
}

impl Parser<Stmt> for ExpressionParser {
    type Expr = Expr;
    type Error = String;

    fn new(input: Vec<Token>) -> Self {
        Self {
            _input: PeekIter::new(input.into_iter()),
            _errors: vec![],
            seen: HashMap::new(),
        }
    }

    fn errors(&self) -> &Vec<Self::Error> {
        &self._errors
    }

    fn input(&mut self) -> &mut PeekIter<Token> {
        &mut self._input
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
        assert!(self.done());
        parsed
    }

    fn parse_float(&mut self, f: f64) -> Self::Expr {
        Self::Expr::Const(f)
    }

    fn parse_variable(&mut self, name: String) -> Self::Expr {
        Expr::Var(name)
    }

    fn parse_var_pattern(&mut self, name: String) -> Self::Expr {
        self.parse_pattern(name)
    }

    fn parse_const_pattern(&mut self, name: String) -> Self::Expr {
        self.parse_pattern(name)
    }

    fn parse_any_pattern(&mut self, name: String) -> Self::Expr {
        self.parse_pattern(name)
    }

    fn parse_open_paren(&mut self) -> Self::Expr {
        self.input().next(); // eat open paren
        Self::Expr::Parend(self.expr())
    }

    fn parse_open_bracket(&mut self) -> Self::Expr {
        self.input().next(); // eat open paren
        Self::Expr::Bracketed(self.expr())
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

    parser_error_tests! {
        parse_expression

        variable_pattern:        "$a"      => concat!(r#"The pattern "$a" cannot be used in an expression. "#,
                                                      r#"Consider using "a" as a variable instead."#)
        const_pattern:           "#a"      => concat!(r##"The pattern "#a" cannot be used in an expression. "##,
                                                      r#"Consider using "a" as a variable instead."#)
        any_pattern:             "_a"      => concat!(r##"The pattern "_a" cannot be used in an expression. "##,
                                                      r#"Consider using "a" as a variable instead."#)

        multiple_errors:         "_a + $a" => concat!(r##"The pattern "_a" cannot be used in an expression. "##,
                                                      r#"Consider using "a" as a variable instead."#,
                                                      "\n",
                                                      r##"The pattern "$a" cannot be used in an expression. "##,
                                                      r#"Consider using "a" as a variable instead."#)
    }

    #[test]
    fn common_subexpression_elimination() {
        let program = "1 * 2 + 1 * 2";
        let tokens = scan(program);
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
