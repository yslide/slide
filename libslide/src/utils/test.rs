#[macro_export]
macro_rules! parse_expr {
    ($expr:expr) => {{
        use crate::grammar::*;
        use crate::{parse_expression, scan};

        let tokens = scan($expr).tokens;
        let (parsed, _) = parse_expression(tokens);
        match parsed {
            Stmt::Expr(expr) => Rc::new(expr),
            // TODO: see below
            _ => unreachable!(),
        }
    }};
}
