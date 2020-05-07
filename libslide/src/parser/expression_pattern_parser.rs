use super::Parser;

use crate::grammar::*;
use crate::scanner::types::Token;
use crate::utils::PeekIter;

pub struct ExpressionPatternParser {
    _input: PeekIter<Token>,
    _errors: Vec<String>,
}

impl Parser for ExpressionPatternParser {
    type Error = String;

    fn new(input: Vec<Token>) -> Self {
        Self {
            _input: PeekIter::new(input.into_iter()),
            _errors: vec![],
        }
    }

    fn errors(&self) -> &Vec<Self::Error> {
        &self._errors
    }

    fn input(&mut self) -> &mut PeekIter<Token> {
        &mut self._input
    }

    fn parse_variable(&mut self, name: String) -> Expr {
        self._errors.push(format!(
            concat!(
                r#"The variable "{name}" cannot be used in an expression pattern. "#,
                r##"Consider using "${name}", "#{name}", or "_{name}" as a variable instead."##
            ),
            name = name,
        ));
        Var { name }.into()
    }

    fn parse_pattern(&mut self, name: String) -> Expr {
        Var { name }.into()
    }
}

#[cfg(test)]
mod tests {
    parser_tests! {
        ExpressionPattern

        pattern:                 "$a"
        pattern_in_op_left:      "$a + 1"
        pattern_in_op_right:     "1 + $a"
    }

    parser_error_tests! {
        ExpressionPattern

        variable:                "a"     => concat!(r#"The variable "a" cannot be used in an expression pattern. "#,
                                                    r##"Consider using "$a", "#a", or "_a" as a variable instead."##)

        multiple_errors:         "a + b" => concat!(r#"The variable "a" cannot be used in an expression pattern. "#,
                                                    r##"Consider using "$a", "#a", or "_a" as a variable instead."##,
                                                    "\n",
                                                    r#"The variable "b" cannot be used in an expression pattern. "#,
                                                    r##"Consider using "$b", "#b", or "_b" as a variable instead."##)
    }
}
