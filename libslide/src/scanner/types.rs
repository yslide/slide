// A definition of a token - used in the scanner
// Written by Luke Bhan, 2/19/2020
//
//

use core::fmt;

#[derive(PartialEq, Clone, Debug)]
pub enum TokenType {
    // Stores a floating point number as dp
    Float(f64),

    // Stores an int - signed
    Int(i64),

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

    // invalid token
    Invalid(String),

    // end of file
    EOF,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Token {
    pub token_type: TokenType,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenType::*;
        write!(
            f,
            "{}",
            match &self.token_type {
                Float(num) => num.to_string(),
                Int(num) => num.to_string(),
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
                Invalid(s) => format!("Invalid({})", s),
                EOF => format!("<EOF>"),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    mod format {
        use crate::scanner::types::*;

        macro_rules! format_tests {
        ($($name:ident: $token_type:expr, $format_str:expr)*) => {
        $(
            #[test]
            fn $name() {
                use TokenType::*;
                let tok = Token {token_type: $token_type};
                assert_eq!(tok.to_string(), $format_str);
            }
        )*
        }
    }

        format_tests! {
            float: Float(1.3), "1.3"
            int: Int(10), "10"
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
            invalid: Invalid("@&@".into()), "Invalid(@&@)"
        }
    }
}
