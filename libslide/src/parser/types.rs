use crate::scanner::Token;
use core::fmt;

pub enum Expr {
    Float(f64),
    Int(i64),
    BinOp(BinOp),
    // I added un op even though I havent implemented + or -
    UnaryOp(UnaryOp),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Expr::*;
        write!(
            f,
            "{}",
            match &self {
                Float(num) => num.to_string(),
                Int(num) => num.to_string(),
                BinOp(bin_op) => bin_op.to_string(),
                UnaryOp(unary_op) => unary_op.to_string(),
            }
        )
    }
}

pub struct BinOp {
    pub op: Token,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.op.to_string(),
            self.lhs.to_string(),
            self.rhs.to_string(),
        )
    }
}

pub struct UnaryOp {
    pub op: Token,
    pub rhs: Box<Expr>,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {})", self.op.to_string(), self.rhs.to_string(),)
    }
}

#[cfg(test)]
mod tests {
    mod format {
        use crate::scanner::types::*;

        macro_rules! format_tests {
            ($($name:ident: $expr:expr, $format_str:expr)*) => {
            $(
                #[test]
                fn $name() {
                    use crate::parser::types::*;
                    let expr = $expr;
                    assert_eq!(expr.to_string(), $format_str);
                }
            )*
            }
        }

        format_tests! {
            float: Expr::Float(1.3), "1.3"
            int: Expr::Int(10), "10"
            binary_op: Expr::BinOp(BinOp {
                op: Token {token_type: TokenType::Plus},
                lhs: Box::new(Expr::Int(1)),
                rhs: Box::new(Expr::Int(2))
            }), "(+ 1 2)"
            unary_op: Expr::UnaryOp(UnaryOp {
                op: Token {token_type: TokenType::Plus},
                rhs: Box::new(Expr::Int(1))
            }), "(+ 1)"
        }
    }
}
