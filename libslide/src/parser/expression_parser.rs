use super::Parser;

use crate::grammar::*;
use crate::scanner::types::Token;
use crate::utils::{PeekIter, StringUtils};

pub struct ExpressionParser {
    _input: PeekIter<Token>,
    _errors: Vec<String>,
}

impl Parser for ExpressionParser {
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
        Var { name }.into()
    }

    fn parse_pattern(&mut self, name: String) -> Expr {
        self._errors.push(format!(
            concat!(
                r#"The pattern "{name}" cannot be used in an expression. "#,
                r#"Consider using "{cut_name}" as a variable instead."#
            ),
            name = name,
            cut_name = name.to_string().substring(1, name.len() - 1),
        ));
        Var { name }.into()
    }
}

#[cfg(test)]
mod tests {
    parser_tests! {
        Expression

        variable:                "a"
        variable_in_op_left:     "a + 1"
        variable_in_op_right:    "1 + a"
        assignment_op:           "a = 5"
        assignment_op_expr:      "a = 5 + 2 ^ 3"
    }

    parser_error_tests! {
        Expression

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
