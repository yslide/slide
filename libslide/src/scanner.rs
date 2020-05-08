mod options;
pub mod types;

pub use crate::utils::PeekIter;
pub use options::ScannerOptions;
use types::*;

pub use std::vec::IntoIter;

pub fn scan<T: Into<String>>(input: T, scanner_options: ScannerOptions) -> Vec<Token> {
    let mut scanner = Scanner::new(input, scanner_options);
    scanner.scan();
    scanner.output
}

struct Scanner {
    input: PeekIter<char>,
    options: ScannerOptions,
    pub output: Vec<Token>,
}

impl Scanner {
    // instantiate a new scanner
    pub fn new<T: Into<String>>(input: T, options: ScannerOptions) -> Scanner {
        let program = input.into();
        let chars: Vec<char> = program.chars().collect();
        let mut output = Vec::new();
        output.reserve(program.len() / 3);

        Scanner {
            input: PeekIter::new(chars.into_iter()),
            options,
            output,
        }
    }

    pub fn scan(&mut self) {
        // iterate through string
        while let Some(c) = self.input.peek() {
            match c {
                _ if c.is_whitespace() => {
                    self.input.next();
                }
                _ if c.is_digit(10) => self.scan_num(),
                _ if self.options.is_var_char(*c) => self.scan_var(),
                _ => self.scan_symbol(),
            }
        }

        self.output.push(Token::new(TokenType::EOF));
    }

    // matches token with symbol and creates it: private helper function
    fn scan_symbol(&mut self) {
        use TokenType::*;
        let ty = match self.input.next().unwrap() {
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
        self.output.push(Token::new(ty));
    }

    // iterates through any digits to create a token of that value
    fn scan_num(&mut self) {
        let mut float_str: String = self.input.collect_until(|c| c.is_digit(10));
        if let Some('.') = self.input.peek() {
            float_str.push('.');
            self.input.next();
            float_str.push_str(&self.input.collect_until::<_, String>(|c| c.is_digit(10)));
        }
        let tok = Token::new(TokenType::Float(float_str.parse::<f64>().unwrap()));
        self.output.push(tok);
    }

    fn scan_var(&mut self) {
        let options = self.options;
        let var_str: String = self.input.collect_until(|c| options.is_var_char(*c));
        let tok = Token::new(TokenType::Variable(var_str));
        self.output.push(tok);
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
                use crate::scanner::scan;
                use crate::scanner::ScannerOptions;

                let mut tokens = scan($program, ScannerOptions::default())
                    .into_iter()
                    .map(|tok| tok.to_string())
                    .collect::<Vec<_>>();
                tokens.pop();
                assert_eq!(tokens.join(" "), $format_str);
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
            invalid_numbers: "1.2.3", "1.2 Invalid(.) 3"
            invalid_tokens: "@", "Invalid(@)"
            invalid_tokens_mixed_with_valid: "=@/", "= Invalid(@) /"
            invalid_expressions: "1 + * 2", "1 + * 2"
        }
    }
}
