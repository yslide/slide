pub mod types;
pub use types::*;

pub struct Scanner {
    input: Vec<char>,
    pub output: Vec<Token>,
}

impl Scanner {
    // instantiate a new scanner
    pub fn new<T: Into<String>>(input: T) -> Scanner {
        Scanner {
            input: input.into().chars().collect(),
            output: Vec::new(),
        }
    }

    // matches token with symbol and creates it: private helper function
    fn create_symbol_token(c: char) -> Token {
        let t = match c {
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
        return Token { token_type: t };
    }

    // iterates through any digits to create a token of that value
    fn iterate_digit(&mut self, mut i: usize) -> (Token, usize) {
        let mut int_str = "".to_owned();
        let mut dec_str = ".".to_owned();
        let ret: Token;
        // iterate through integer part
        while i < self.input.len() && (self.input[i]).is_digit(10) {
            int_str.push(self.input[i]);
            i += 1;
        }
        // iterate through decimal
        if i < self.input.len() && self.input[i] == '.' {
            i += 1;
            while i < self.input.len() && (self.input[i]).is_digit(10) {
                dec_str.push(self.input[i]);
                i += 1;
            }
            int_str.push_str(&dec_str);
            // turn integer and decmial strings into token
            ret = Token {
                token_type: TokenType::Float(int_str.parse::<f64>().unwrap()),
            }
        } else {
            // turn integer string into token and default the float
            ret = Token {
                token_type: TokenType::Int(int_str.parse::<i64>().unwrap()),
            }
        }
        return (ret, i);
    }

    fn iterate_var(&mut self, mut i: usize) -> (Token, usize) {
        let mut var_str = String::new();
        while i < self.input.len() && self.input[i].is_alphabetic() {
            var_str.push(self.input[i]);
            i += 1
        }
        let var = Token {
            token_type: TokenType::Variable(var_str),
        };
        return (var, i);
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
                } else if self.input[i].is_alphabetic() {
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

        self.output.push(Token {
            token_type: TokenType::EOF,
        });
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
                use crate::scanner::Scanner;

                let mut scanner = Scanner::new($program);
                scanner.scan();
                let mut tokens = scanner
                    .output
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
