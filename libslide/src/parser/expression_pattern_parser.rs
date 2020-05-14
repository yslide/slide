use super::Parser;
use crate::grammar::*;
use crate::scanner::types::Token;
use crate::utils::{hash, PeekIter};

use std::collections::HashMap;
use std::rc::Rc;

pub fn parse(input: Vec<Token>) -> (Rc<ExprPat>, Vec<String>) {
    let mut parser = ExpressionPatternParser::new(input);
    let parsed = parser.parse();
    let errors = parser.errors().iter().map(|e| e.to_string()).collect();
    (parsed, errors)
}

pub struct ExpressionPatternParser {
    _input: PeekIter<Token>,
    _errors: Vec<String>,
    // We use an untyped hash here because we don't want to clone an Expr into the map in case it's
    // already there when using an entry API.
    // TODO: replace with Expr when raw_entry API is stabilized (see rust#56167)
    seen: HashMap<u64, Rc<ExprPat>>,
}

impl Parser<Rc<ExprPat>> for ExpressionPatternParser {
    type Expr = ExprPat;
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

    fn parse(&mut self) -> Rc<ExprPat> {
        let parsed = self.expr();
        assert!(self.done());
        parsed
    }

    fn parse_float(&mut self, f: f64) -> Self::Expr {
        Self::Expr::Const(f)
    }

    fn parse_variable(&mut self, name: String) -> Self::Expr {
        self._errors.push(format!(
            concat!(
                r#"The variable "{name}" cannot be used in an expression pattern. "#,
                r##"Consider using "${name}", "#{name}", or "_{name}" as a variable instead."##
            ),
            name = name,
        ));
        Self::Expr::VarPat(name)
    }

    fn parse_var_pattern(&mut self, name: String) -> Self::Expr {
        Self::Expr::VarPat(name)
    }

    fn parse_const_pattern(&mut self, name: String) -> Self::Expr {
        Self::Expr::ConstPat(name)
    }

    fn parse_any_pattern(&mut self, name: String) -> Self::Expr {
        Self::Expr::AnyPat(name)
    }

    fn parse_open_paren(&mut self) -> Self::Expr {
        self.input().next(); // eat open paren
        Self::Expr::Parend(self.expr())
    }

    fn parse_open_brace(&mut self) -> Self::Expr {
        self.input().next(); // eat open paren
        Self::Expr::Braced(self.expr())
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
        parse_expression_pattern

        pattern:                 "$a"
        pattern_in_op_left:      "$a + 1"
        pattern_in_op_right:     "1 + $a"
    }

    parser_error_tests! {
        parse_expression_pattern

        variable:                "a"     => concat!(r#"The variable "a" cannot be used in an expression pattern. "#,
                                                    r##"Consider using "$a", "#a", or "_a" as a variable instead."##)

        multiple_errors:         "a + b" => concat!(r#"The variable "a" cannot be used in an expression pattern. "#,
                                                    r##"Consider using "$a", "#a", or "_a" as a variable instead."##,
                                                    "\n",
                                                    r#"The variable "b" cannot be used in an expression pattern. "#,
                                                    r##"Consider using "$b", "#b", or "_b" as a variable instead."##)
    }

    #[test]
    fn common_subexpression_elimination() {
        let program = "$v * #c + $v * #c";
        let tokens = scan(program);
        let (parsed, _) = parse(tokens);
        let (l, r) = match (*parsed).clone() {
            ExprPat::BinaryExpr(BinaryExpr { lhs, rhs, .. }) => (lhs, rhs),
            _ => unreachable!(),
        };
        assert!(std::ptr::eq(l.as_ref(), r.as_ref())); // $v * #c

        let (ll, lr, rl, rr) = match (l.as_ref(), r.as_ref()) {
            (
                ExprPat::BinaryExpr(BinaryExpr {
                    lhs: ll, rhs: lr, ..
                }),
                ExprPat::BinaryExpr(BinaryExpr {
                    lhs: rl, rhs: rr, ..
                }),
            ) => (ll, lr, rl, rr),
            _ => unreachable!(),
        };
        assert!(std::ptr::eq(ll.as_ref(), rl.as_ref())); // 1
        assert!(std::ptr::eq(lr.as_ref(), rr.as_ref())); // 2
    }
}
