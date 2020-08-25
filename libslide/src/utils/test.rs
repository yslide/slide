/// Parses an expression.
#[macro_export]
#[doc(hidden)]
macro_rules! parse_expr {
    ($expr:expr) => {{
        use crate::grammar::*;
        use crate::{parse_expression, scan};

        let tokens = scan($expr).tokens;
        let (parsed, _) = parse_expression(tokens);
        match parsed {
            Stmt::Expr(expr) => expr,
            _ => unreachable!(),
        }
    }};
}
