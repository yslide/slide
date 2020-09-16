//! Definitions of types used in the libslide scanner.

use crate::common::Span;
use core::fmt;

/// The type of a [Token][Token].
#[derive(PartialEq, Clone, Debug)]
pub enum TokenType {
    /// Stores a scanned number in double precision.
    Float(f64),

    /// + symbol
    Plus,

    /// - symbol
    Minus,

    /// * symbol
    Mult,

    /// / symbol
    Div,

    /// % symbol
    Mod,

    /// ^ symbol
    Exp,

    /// = symbol
    Equal,

    /// ( symbol
    OpenParen,

    /// ) symbol
    CloseParen,

    /// [ symbol
    OpenBracket,

    /// ] symbol
    CloseBracket,

    /// A variable name.
    Variable(String),

    /// A variable pattern, of form $name.
    VariablePattern(String),

    /// A constant pattern, of form #name.
    ConstPattern(String),

    /// An any pattern, of form _name.
    AnyPattern(String),

    /// An invalid token.
    Invalid(String),

    /// End of file.
    EOF,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TokenType::*;
        write!(
            f,
            "{}",
            match self {
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
                VariablePattern(s) => s.to_string(),
                ConstPattern(s) => s.to_string(),
                AnyPattern(s) => s.to_string(),
                Invalid(s) => s.to_string(),
                EOF => "end of file".into(),
            }
        )
    }
}

/// Describes a token in a slide program.
#[derive(PartialEq, Clone, Debug)]
pub struct Token {
    /// The type of the token.
    pub ty: TokenType,
    /// The source span of the token.
    pub span: Span,
    /// The full span of the token including its leading trivia.
    pub full_span: Span,
}

impl Token {
    /// Creates a new token.
    pub fn new<Sp1, Sp2>(ty: TokenType, span: Sp1, full_span: Sp2) -> Self
    where
        Sp1: Into<Span>,
        Sp2: Into<Span>,
    {
        Self {
            ty,
            span: span.into(),
            full_span: full_span.into(),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ty.to_string())
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
                    let tok = Token::new($ty, (0..0), (0..0));
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
            invalid: Invalid("@&@".into()), "@&@"
        }
    }
}
