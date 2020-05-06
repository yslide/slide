mod options;
pub mod types;

pub use options::ScannerOptions;
use types::*;

pub fn scan<T: Into<String>>(input: T, scanner_options: ScannerOptions) -> Vec<Token> {
    let mut scanner = Scanner::new(input, scanner_options);
    scanner.scan();
    scanner.output
}

struct Scanner {
    input: Vec<char>,
    options: ScannerOptions,
    pub output: Vec<Token>,
}

impl Scanner {
    // instantiate a new scanner
    pub fn new<T: Into<String>>(input: T, options: ScannerOptions) -> Scanner {
        Scanner {
            input: input.into().chars().collect(),
            options,
            output: Vec::new(),
        }
    }

    // matches token with symbol and creates it: private helper function
    fn create_symbol_token(c: char) -> Token {
        let ty = match c {
            '+' => TokenType::Plus,
            '-' => TokenType::Minus,
            '*' => TokenType::Mult,
            '/' => TokenType::Div,
            '%' => TokenType::Mod,
            '^' => TokenType::Exp,
            '=' => TokenType::Equal,
            '(' => TokenType::OpenParen,
            ')' => TokenType::CloseParen,
            '[' => TokenType::OpenBracket,
            ']' => TokenType::CloseBracket,
            _ => TokenType::Invalid(c.to_string()),
        };
        Token { ty }
    }

    // iterates through any digits to create a token of that value
    fn iterate_digit(&mut self, mut i: usize) -> (Token, usize) {
        let mut float_str = String::new();
        // iterate through integer part
        while i < self.input.len() && (self.input[i]).is_digit(10) {
            float_str.push(self.input[i]);
            i += 1;
        }
        // iterate through decimal
        if i < self.input.len() && self.input[i] == '.' {
            i += 1;
            float_str.push('.');
            while i < self.input.len() && (self.input[i]).is_digit(10) {
                float_str.push(self.input[i]);
                i += 1;
            }
        }
        let num = Token {
            ty: TokenType::Float(float_str.parse::<f64>().unwrap()),
        };
        (num, i)
    }

    fn iterate_var(&mut self, mut i: usize) -> (Token, usize) {
        let mut var_str = String::new();
        while i < self.input.len() && self.options.is_var_char(self.input[i]) {
            var_str.push(self.input[i]);
            i += 1
        }
        let var = Token {
            ty: TokenType::Variable(var_str),
        };
        (var, i)
    }

    pub fn scan(&mut self) {
        let mut i: usize = 0;
        // iterate through string
        while i < self.input.len() {
            // ignore whitespace
            if !((self.input[i]).is_whitespace()) {
                // check for digit and call correct helper function
                if self.input[i].is_digit(10) {
                    let (num, new_idx) = self.iterate_digit(i);
                    i = new_idx;
                    self.output.push(num);
                } else if self.options.is_var_char(self.input[i]) {
                    let (var, new_idx) = self.iterate_var(i);
                    i = new_idx;
                    self.output.push(var);
                } else {
                    self.output
                        .push(Scanner::create_symbol_token(self.input[i]));
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        self.output.push(Token { ty: TokenType::EOF });
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
