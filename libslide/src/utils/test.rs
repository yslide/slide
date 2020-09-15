/// Parses a statement.
#[macro_export]
macro_rules! parse_stmt {
    ($expr:expr) => {{
        use crate::{parse_statement, scan};

        let tokens = scan($expr).tokens;
        let (parsed, _) = parse_statement(tokens);
        parsed
    }};
}

/// Parses an expression.
#[macro_export]
macro_rules! parse_expr {
    ($expr:expr) => {{
        use crate::grammar::*;
        match crate::parse_stmt!($expr) {
            Stmt::Expr(expr) => expr,
            _ => unreachable!(),
        }
    }};
}
