pub mod types;

use crate::diagnostics::Diagnostic;
use types::TokenType as TT;
pub use types::*;

pub struct ScanResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

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
    pub output: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

macro_rules! tok {
    ($ty:expr, $pos:expr) => {
        Token::new($ty, $pos)
    };
}

impl Scanner {
    // instantiate a new scanner
    pub fn new(input: &str) -> Scanner {
        Scanner {
            pos: 0,
            input: input.chars().collect(),
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
                _ if c.is_whitespace() => {
                    self.next();
                }
                _ if c.is_digit(10) => self.scan_num(),
                '$' => self.scan_var_pattern(),
                '#' => self.scan_const_pattern(),
                '_' => self.scan_any_pattern(),
                _ if c.is_alphabetic() => self.scan_var(),
                _ => self.scan_symbol(),
            }
        }

        self.output.push(tok!(TT::EOF, (self.pos, self.pos + 1)));
    }

    // matches token with symbol and creates it: private helper function
    fn scan_symbol(&mut self) {
        use TokenType::*;
        let start = self.pos;
        let ty = match self.next().unwrap() {
            '+' => Plus,
            '-' => Minus,
            '*' => Mult,
            '/' => Div,
            '%' => Mod,
            '^' => Exp,
            '=' => Equal,
            '(' => OpenParen,
            ')' => CloseParen,
            '[' => OpenBracket,
            ']' => CloseBracket,
            c => Invalid(c.to_string()),
        };
        let span = start..self.pos;

        if matches!(ty, Invalid(..)) {
            let diag = Diagnostic::span_err(span.clone(), "Invalid token", None)
                .with_note(span.clone(), "token must be mathematically significant");
            self.push_diag(diag);
        }
        self.output.push(tok!(ty, span));
    }

    // iterates through any digits to create a token of that value
    fn scan_num(&mut self) {
        let start = self.pos;

        let mut float_str = self.collect_while(|c| c.is_digit(10));
        if let Some('.') = self.peek() {
            float_str.push(*self.next().unwrap());
            float_str.push_str(&self.collect_while(|c| c.is_digit(10)));
        }
        let float = float_str.parse::<f64>().unwrap();

        self.output.push(tok!(TT::Float(float), (start, self.pos)));
    }

    fn scan_var_str(&mut self) -> String {
        self.collect_while(|c| c.is_alphabetic())
    }

    fn scan_var(&mut self) {
        let start = self.pos;

        let var_name = self.scan_var_str();
        self.output
            .push(tok!(TT::Variable(var_name), (start, self.pos)));
    }

    fn scan_var_pattern(&mut self) {
        let start = self.pos;

        let mut pat = String::with_capacity(4);
        // Push the pattern prefix, which we already verified exists.
        pat.push(*self.next().unwrap());
        pat.push_str(&self.scan_var_str());

        self.output
            .push(tok!(TT::VariablePattern(pat), (start, self.pos)));
    }

    fn scan_const_pattern(&mut self) {
        let start = self.pos;

        let mut pat = String::with_capacity(4);
        // Push the pattern prefix, which we already verified exists.
        pat.push(*self.next().unwrap());
        pat.push_str(&self.scan_var_str());

        self.output
            .push(tok!(TT::ConstPattern(pat), (start, self.pos)));
    }

    fn scan_any_pattern(&mut self) {
        let start = self.pos;

        let mut pat = String::with_capacity(4);
        // Push the pattern prefix, which we already verified exists.
        pat.push(*self.next().unwrap());
        pat.push_str(&self.scan_var_str());

        self.output
            .push(tok!(TT::AnyPattern(pat), (start, self.pos)));
    }
}

#[cfg(test)]
mod tests {
    // Tests the Scanner's output against a humanized string representation of the expected tokens.
    // See [Token]'s impl of Display for more details.
    // [Token]: src/scanner/types.rs
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
}
