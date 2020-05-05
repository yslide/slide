// A definition of a token - used in the scanner
// Written by Luke Bhan, 2/19/2020
//
//

use core::fmt;

#[derive(PartialEq, Clone, Debug)]
pub enum TokenType {
    // Stores a floating point number as dp
    Float(f64),

    // Identifies addition
    Plus,

    // Identifies subtraction
    Minus,

    // Identifies multiplication
    Mult,

    // Identifies division
    Div,

    // Identifies modulo
    Mod,

    // Identifies exponent
    Exp,

    // Identifies an equal sign
    Equal,

    // open parentheses (
    OpenParen,

    // close parentheses )
    CloseParen,

    // open bracket [
    OpenBracket,

    // close bracket ]
    CloseBracket,

    // variable name
    Variable(String),

    // invalid token
    Invalid(String),

    // end of file
    EOF,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Token {
    pub ty: TokenType,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenType::*;
        write!(
            f,
            "{}",
            match &self.ty {
                Float(num) => num.to_string(),
                Plus => "+".into(),
                Minus => "-".into(),
                Mult => "*".into(),
                Div => "/".into(),
                Mod => "%".into(),
                Exp => "^".into(),
                Equal => "=".into(),
                OpenParen => "(".into(),
                CloseParen => ")".into(),
                OpenBracket => "[".into(),
                CloseBracket => "]".into(),
                Variable(s) => s.to_string(),
                Invalid(s) => format!("Invalid({})", s),
                EOF => "<EOF>".into(),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    mod format {
        use crate::scanner::types::*;

        macro_rules! format_tests {
        ($($name:ident: $ty:expr, $format_str:expr)*) => {
        $(
            #[test]
            fn $name() {
                use TokenType::*;
                let tok = Token {ty: $ty};
                assert_eq!(tok.to_string(), $format_str);
            }
        )*
        }
    }

        format_tests! {
            float: Float(1.3), "1.3"
            plus: Plus, "+"
            minus: Minus, "-"
            mult: Mult, "*"
            div: Div, "/"
            modulo: Mod, "%"
            exp: Exp, "^"
            equal: Equal, "="
            open_paren: OpenParen, "("
            close_paren: CloseParen, ")"
            open_bracket: OpenBracket, "["
            close_bracket: CloseBracket, "]"
            variable: Variable("ab".into()), "ab"
            invalid: Invalid("@&@".into()), "Invalid(@&@)"
        }
    }
}
