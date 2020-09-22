//! Tokenizes slide programs and produces lexing diagnostics.

#[macro_use]
mod errors;
pub use errors::ScanErrors;
use errors::*;

pub mod types;
use types::TokenType as TT;
pub use types::*;

use crate::common::Span;
use crate::diagnostics::{Diagnostic, DiagnosticRecord};
use strtod::strtod;

/// Describes the result of tokenizing a slide program.
pub struct ScanResult {
    /// Tokens of the program.
    pub tokens: Vec<Token>,
    /// Lexing diagnostics encountered while scanning the program.
    pub diagnostics: Vec<Diagnostic>,
}

/// Scans and tokenizes a string-like slide program.
pub fn scan<'a, T: Into<&'a str>>(input: T) -> ScanResult {
    let mut scanner = Scanner::new(input.into());
    scanner.scan();
    ScanResult {
        tokens: scanner.output,
        diagnostics: scanner.diagnostics,
    }
}

struct Scanner {
    pos: usize,
    input: Vec<char>,
    leading_trivia_start: usize,
    pub output: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Scanner {
    /// Instantiate a new scanner.
    pub fn new(input: &str) -> Scanner {
        Scanner {
            pos: 0,
            input: input.chars().collect(),
            leading_trivia_start: 0,
            output: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    #[inline]
    fn peek(&self) -> Option<&char> {
        self.input.get(self.pos)
    }

    #[inline]
    fn next(&mut self) -> Option<&char> {
        let ch = self.input.get(self.pos);
        self.pos += 1;
        ch
    }

    #[inline]
    fn push_diag(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn push_tok<S: Into<Span>>(&mut self, ty: TokenType, span: S) {
        let span = span.into();
        let full_span = (self.leading_trivia_start, span.hi);
        self.output.push(Token::new(ty, span, full_span));

        self.leading_trivia_start = span.hi;
    }

    fn collect_while(&mut self, pred: fn(&char) -> bool) -> String {
        let mut s = String::with_capacity(8);
        while let Some(true) = self.peek().map(pred) {
            s.push(*self.next().unwrap());
        }
        s
    }

    pub fn scan(&mut self) {
        // iterate through string
        while let Some(c) = self.peek() {
            match c {
                _ if c.is_whitespace() => self.scan_trivia(),
                _ if c.is_digit(10) => self.scan_num(),
                '$' => self.scan_var_pattern(),
                '#' => self.scan_const_pattern(),
                '_' => self.scan_any_pattern(),
                _ if c.is_alphabetic() => self.scan_var(),
                _ => self.scan_symbol(),
            }
        }

        self.push_tok(TT::EOF, (self.pos, self.pos + 1));
    }

    /// Scans leading trivia, including whitespace.
    fn scan_trivia(&mut self) {
        // For now, just skip all trivia.
        self.next();
    }

    /// Matches a symbol with a token and creates it.
    fn scan_symbol(&mut self) {
        use TokenType::*;
        let mut did_you_mean = None;
        let start = self.pos;
        let mut span = None;
        let ty = match self.next().unwrap() {
            '+' => Plus,
            '-' => Minus,
            '*' => Mult,
            '/' => Div,
            '%' => Mod,
            '^' => Exp,
            '=' => Equal,
            ':' => {
                if self.peek() == Some(&'=') {
                    self.next();
                    AssignDefine
                } else {
                    span = Some(start..start + 1);
                    self.collect_while(|c| c.is_whitespace());
                    if self.peek() == Some(&'=') {
                        did_you_mean = Some((":=", start..self.pos + 1));
                    }
                    Invalid(":".to_owned())
                }
            }
            '(' => OpenParen,
            ')' => CloseParen,
            '[' => OpenBracket,
            ']' => CloseBracket,
            c => Invalid(c.to_string()),
        };
        let span = span.unwrap_or_else(|| start..self.pos);

        if matches!(ty, Invalid(..)) {
            self.push_diag(InvalidToken!(span.clone(), did_you_mean));
        }
        self.push_tok(ty, span);
    }

    /// Scans through the content of a number to create a token of that value.
    fn scan_num(&mut self) {
        let start = self.pos;

        let mut float_str = self.collect_while(|c| c.is_digit(10));
        if let Some('.') = self.peek() {
            float_str.push(*self.next().unwrap());
            float_str.push_str(&self.collect_while(|c| c.is_digit(10)));
        }
        // TODO(https://github.com/rust-lang/rust/issues/31407): rustc's float parser may drop some
        // valid float literals. For now, use an external parser.
        let float = strtod(&float_str).unwrap();

        self.push_tok(TT::Float(float), (start, self.pos));
    }

    fn scan_var_str(&mut self) -> String {
        self.collect_while(|c| c.is_alphabetic())
    }

    fn scan_var(&mut self) {
        let start = self.pos;

        let var_name = self.scan_var_str();
        self.push_tok(TT::Variable(var_name), (start, self.pos));
    }

    fn scan_var_pattern(&mut self) {
        let start = self.pos;

        let mut pat = String::with_capacity(4);
        // Push the pattern prefix, which we already verified exists.
        pat.push(*self.next().unwrap());
        pat.push_str(&self.scan_var_str());

        self.push_tok(TT::VariablePattern(pat), (start, self.pos));
    }

    fn scan_const_pattern(&mut self) {
        let start = self.pos;

        let mut pat = String::with_capacity(4);
        // Push the pattern prefix, which we already verified exists.
        pat.push(*self.next().unwrap());
        pat.push_str(&self.scan_var_str());

        self.push_tok(TT::ConstPattern(pat), (start, self.pos));
    }

    fn scan_any_pattern(&mut self) {
        let start = self.pos;

        let mut pat = String::with_capacity(4);
        // Push the pattern prefix, which we already verified exists.
        pat.push(*self.next().unwrap());
        pat.push_str(&self.scan_var_str());

        self.push_tok(TT::AnyPattern(pat), (start, self.pos));
    }
}

#[cfg(test)]
mod tests {
    /// Tests the Scanner's output against a humanized string representation of the expected tokens.
    /// See [Token]'s impl of Display for more details.
    /// [Token]: src/scanner/types.rs
    macro_rules! scanner_tests {
        ($($name:ident: $program:expr, $format_str:expr)*) => {
        $(
            #[test]
            fn $name() {
                use crate::common::Span;
                use crate::scanner::scan;

                let mut tokens = scan($program).tokens;
                tokens.pop(); // EOF

                // First check if token string matches.
                let tokens_str = tokens
                    .iter()
                    .map(|tok| tok.to_string())
                    .collect::<Vec<_>>().join(" ");
                assert_eq!(tokens_str, $format_str);

                // Now check the token spans are correct.
                for token in tokens {
                    let Span {lo, hi} = token.span;
                    assert_eq!($program[lo..hi], token.to_string());
                }
            }
        )*
        }
    }

    mod scan {
        scanner_tests! {
            integer: "2", "2"
            float: "3.2", "3.2"
            plus: "+", "+"
            minus: "-", "-"
            mult: "*", "*"
            div: "/", "/"
            modulo: "%", "%"
            exp: "^", "^"
            equal: "=", "="
            open_paren: "(", "("
            close_paren: ")", ")"
            open_bracket: "[", "["
            close_bracket: "]", "]"
            variable_pattern: "$a", "$a"
            const_pattern: "#a", "#a"
            any_pattern: "_a", "_a"

            empty_string: "", ""
            skip_whitespace: "  =  ", "="

            multiple_integers: "1 2 3", "1 2 3"
            multiple_floats: "1.2 2.3 3.4", "1.2 2.3 3.4"
            multiple_numbers_mixed: "1 2.3 4", "1 2.3 4"

            expressions: "1 + 2 ^ 5", "1 + 2 ^ 5"

            variables: "a = 5", "a = 5"
            variables_cap: "ABcd = 5", "ABcd = 5"
        }
    }

    mod scan_invalid {
        scanner_tests! {
            invalid_numbers: "1.2.3", "1.2 . 3"
            invalid_tokens: "@", "@"
            invalid_tokens_mixed_with_valid: "=@/", "= @ /"
            invalid_expressions: "1 + * 2", "1 + * 2"
        }
    }

    #[test]
    fn leading_trivia() {
        let program = r#"1 + 2  +    3 -  

 4   ^ 5"#;
        let tokens = crate::scan(program).tokens;
        let toks_with_trivia = vec![
            "1", " +", " 2", "  +", "    3", " -", "  \n\n 4", "   ^", " 5",
        ];
        for (tok, str_with_trivia) in tokens.into_iter().zip(toks_with_trivia) {
            assert_eq!(tok.full_span.over(program), str_with_trivia);
        }
    }
}
