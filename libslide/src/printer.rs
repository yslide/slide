use crate::parser::types::{BinaryExpr, Expr, UnaryExpr, Var};
use crate::visitor::Visitor;

pub fn print(expr: Expr) -> String {
    let mut printer = Printer;
    printer.visit_expr(expr)
}

struct Printer;

impl Visitor for Printer {
    type Result = String;

    fn visit_float(&mut self, item: f64) -> Self::Result {
        item.to_string()
    }

    fn visit_int(&mut self, item: i64) -> Self::Result {
        item.to_string()
    }

    fn visit_var(&mut self, item: Var) -> Self::Result {
        item.name
    }

    fn visit_binary_expr(&mut self, item: BinaryExpr) -> Self::Result {
        format!(
            "{} {} {}",
            self.visit_expr(*item.lhs),
            item.op.to_string(),
            self.visit_expr(*item.rhs)
        )
    }

    fn visit_unary_expr(&mut self, item: UnaryExpr) -> Self::Result {
        format!("{}{}", item.op.to_string(), self.visit_expr(*item.rhs))
    }
}

#[cfg(test)]
mod tests {
    macro_rules! printer_tests {
        ($($name:ident: $program:expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::scanner::scan;
                use crate::parser::parse;
                use crate::parser::types::Stmt;
                use crate::printer::print;

                let tokens = scan($program);
                let parsed = match *parse(tokens) {
                    Stmt::Expr(expr) => expr,
                    Stmt::Assignment(_) => unimplemented!(),
                };

                assert_eq!(print(parsed), $program.to_string())
            }
        )*
        }
    }

    printer_tests! {
        int:             "1"
        float:           "1.1"
        var:             "a"
        addition:        "1 + 2"
        subtraction:     "1 - 2"
        multiplication:  "1 * 2"
        division:        "1 / 2"
        modulo:          "1 % 2"
        exponent:        "1 ^ 2"
        sign_positive:   "+1"
        sign_negative:   "-1"

        nested_binary:   "1 + 2 * 3 + 4"
    }
}
