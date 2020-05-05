use crate::grammar::*;
use crate::visitor::Visitor;
pub mod types;
use types::PEResult;

pub fn evaluate(expr: Stmt) -> PEResult {
    let mut partial_evaluator = PartialEvaluator;
    match expr {
        Stmt::Expr(expr) => partial_evaluator.visit_expr(expr),
        Stmt::Assignment(_) => todo!(),
    }
}

struct PartialEvaluator;

impl Visitor for PartialEvaluator {
    type Result = PEResult;

    fn visit_const(&mut self, item: f64) -> Self::Result {
        item.into()
    }

    fn visit_var(&mut self, item: Var) -> Self::Result {
        Expr::Var(item).into()
    }

    fn visit_binary_expr(&mut self, item: BinaryExpr) -> Self::Result {
        let lhs = self.visit_expr(*item.lhs);
        let rhs = self.visit_expr(*item.rhs);
        PEResult::fold_binary(lhs, rhs, item.op)
    }

    fn visit_unary_expr(&mut self, item: UnaryExpr) -> Self::Result {
        let rhs = self.visit_expr(*item.rhs);
        PEResult::fold_unary(rhs, item.op)
    }
}

#[cfg(test)]
mod tests {
    macro_rules! partial_evaluator_tests {
        ($($name:ident: $program:expr, $result:expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::scanner::scan;
                use crate::parser::parse;
                use crate::partial_evaluator::{evaluate, PEResult::*};

                let tokens = scan($program);
                let parsed = parse(tokens);
                let result = $result.to_string();
                match evaluate(parsed) {
                    Evaluated(r) => assert_eq!(r.to_string(), result),
                    Unevaluated(r) => assert_eq!(r.to_string(), result),
                }
            }
        )*
        }
    }

    mod evaluated {
        partial_evaluator_tests! {
            int:                     "1",                   1
            float:                   "1.1",                 1.1
            sign_positive:           "+1",                  1
            sign_positive_float:     "+1.1",                1.1
            sign_negative:           "-1",                  -1
            sign_negative_float:     "-1.1",                -1.1
            addition:                "1 + 2",               3
            addition_float:          "1.2 + 3.2",           4.4
            subtraction:             "1 - 2",               -1
            subtraction_float:       "1.1 - 2.2",           -1.1
            multiplication:          "1 * 2",               2
            multiplication_float:    "1.2 * 3.4",           4.08
            division:                "1 / 2",               0.5
            division_float:          "3.5 / 0.5",           7
            modulo:                  "8 % 3",               2
            modulo_float:            "3.75 % 0.5",          0.25
            exponent:                "2 ^ 3",               8
            exponent_float:          "6.25 ^ 0.5",          2.5
        }
    }

    mod unevaluated {
        partial_evaluator_tests! {
            var:                     "a",                   "a"
            sign_positive_var:       "+a",                  "+a"
            sign_negative_var:       "-a",                  "-a"
            addition_var:            "a + b",               "a + b"
            substraction_var:        "a - b",               "a - b"
            mult_var:                "a * b",               "a * b"
            div_var:                 "a / b",               "a / b"
            mod_var:                 "a % b",               "a % b"
            exp_var:                 "a ^ b",               "a ^ b"
            pe_left:                 "1 + 2 + a",           "3 + a"
            pe_right:                "a + 1 + 2",           "a + 1 + 2"
        }
    }
}
