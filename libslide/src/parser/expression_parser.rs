use super::Parser;

use crate::grammar::*;
use crate::scanner::types::{Token, TokenType};
use crate::utils::{PeekIter, StringUtils};

pub fn parse(input: Vec<Token>) -> (Stmt, Vec<String>) {
    let mut parser = ExpressionParser::new(input);
    let parsed = parser.parse();
    let errors = parser.errors().iter().map(|e| e.to_string()).collect();
    (*parsed, errors)
}

pub struct ExpressionParser {
    _input: PeekIter<Token>,
    _errors: Vec<String>,
}

impl ExpressionParser {
    fn assignment(&mut self, var: String) -> Box<Stmt> {
        Box::new(Stmt::Assignment(Assignment {
            var,
            rhs: self.expr(),
        }))
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
        }
    }

    fn parse(&mut self) -> Box<Stmt> {
        let mut next_2 = self.input().peek_map_n(2, |tok| tok.ty.clone());
        let parsed = match (next_2.pop_front(), next_2.pop_front()) {
            (Some(TokenType::Variable(name)), Some(TokenType::Equal)) => {
                self.input().next();
                self.input().next();
                self.assignment(name)
            }
            _ => Box::new(Stmt::Expr(*self.expr())),
        };
        assert!(self.done());
        parsed
    }

    fn errors(&self) -> &Vec<Self::Error> {
        &self._errors
    }

    fn input(&mut self) -> &mut PeekIter<Token> {
        &mut self._input
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

    fn parse_open_brace(&mut self) -> Self::Expr {
        self.input().next(); // eat open paren
        Self::Expr::Braced(self.expr())
    }
}

#[cfg(test)]
mod tests {
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
}
