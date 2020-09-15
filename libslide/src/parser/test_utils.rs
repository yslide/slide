#![cfg(test)]

macro_rules! __parse {
    ($parser:ident, $inout:expr) => {{
        use crate::parser::$parser;
        use crate::scanner::scan;

        let inout: Vec<&str> = $inout.split(" => ").collect();
        let pin = inout[0];
        let pout = if inout.len() > 1 {
            inout[1].to_owned()
        } else {
            pin.to_owned()
        };
        let tokens = scan(pin).tokens;
        let (parsed, _) = $parser(tokens);
        (parsed, pin, pout)
    }};
}

macro_rules! __check_parsed {
    (expr: $inout:expr) => {
        use crate::grammar::*;
        use crate::parser::test_utils::verify_expr_spans;

        let (parsed, input, expected_out) = __parse!(parse_statement, $inout);
        assert_eq!(parsed.to_string(), expected_out);

        if input == expected_out {
            // We can automate verification of spans only if the input is in the same emit form as
            // the output.
            let inner_expr = match parsed {
                Stmt::Expr(inner) => inner,
                Stmt::Assignment(Assignment { rhs, .. }) => rhs,
            };
            verify_expr_spans(&inner_expr, &input);
        }
    };

    (expr_pat: $inout:expr) => {
        use crate::parser::test_utils::verify_expr_pat_spans;

        let (parsed, input, expected_out) = __parse!(parse_expression_pattern, $inout);
        assert_eq!(parsed.to_string(), expected_out);

        if input == expected_out {
            // We can automate verification of spans only if the input is in the same emit form as
            // the output.
            verify_expr_pat_spans(&parsed, &input);
        }
    };
}

macro_rules! common_parser_tests {
    ($($name:ident: $inout:expr)*) => {
    $(
        #[test]
        fn $name() {
            __check_parsed!(expr: $inout);
            __check_parsed!(expr_pat: $inout);
        }
    )*
    }
}

macro_rules! parser_tests {
    ($kind:ident $($name:ident: $program:expr)*) => {
    $(
        #[test]
        fn $name() {
            __check_parsed!($kind: $program);
        }
    )*
    }
}

use crate::grammar::*;
use crate::Span;

fn check_span(span: Span, input: &str, actual: String) {
    let expected_from_span = &input[span.lo..span.hi];
    assert_eq!(expected_from_span, actual, "Spans mismatch!");
}

pub fn verify_expr_spans(expr: &InternedExpr, input: &str) {
    check_span(expr.span, input, expr.to_string());
    match expr.as_ref() {
        Expr::BinaryExpr(BinaryExpr { lhs, rhs, .. }) => {
            verify_expr_spans(lhs, input);
            verify_expr_spans(rhs, input);
        }
        Expr::UnaryExpr(UnaryExpr { rhs, .. }) => {
            verify_expr_spans(rhs, input);
        }
        Expr::Parend(inner) | Expr::Bracketed(inner) => verify_expr_spans(inner, input),
        _ => (),
    }
}

pub fn verify_expr_pat_spans(expr: &InternedExprPat, input: &str) {
    check_span(expr.span, input, expr.to_string());
    match expr.as_ref() {
        ExprPat::BinaryExpr(BinaryExpr { lhs, rhs, .. }) => {
            verify_expr_pat_spans(lhs, input);
            verify_expr_pat_spans(rhs, input);
        }
        ExprPat::UnaryExpr(UnaryExpr { rhs, .. }) => {
            verify_expr_pat_spans(rhs, input);
        }
        ExprPat::Parend(inner) | ExprPat::Bracketed(inner) => verify_expr_pat_spans(inner, input),
        _ => (),
    }
}
