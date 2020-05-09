use super::Parser;

use crate::grammar::*;
use crate::scanner::types::Token;
use crate::utils::PeekIter;

pub fn parse(input: Vec<Token>) -> (ExprPat, Vec<String>) {
    let mut parser = ExpressionPatternParser::new(input);
    let parsed = parser.parse();
    let errors = parser.errors().iter().map(|e| e.to_string()).collect();
    (*parsed, errors)
}

pub struct ExpressionPatternParser {
    _input: PeekIter<Token>,
    _errors: Vec<String>,
}

impl Parser<ExprPat> for ExpressionPatternParser {
    type Expr = ExprPat;
    type Error = String;

    fn new(input: Vec<Token>) -> Self {
        Self {
            _input: PeekIter::new(input.into_iter()),
            _errors: vec![],
        }
    }

    fn parse(&mut self) -> Box<ExprPat> {
        let parsed = Box::new(*self.expr());
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
}

#[cfg(test)]
mod tests {
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
}
