pub mod types;

use crate::utils::PeekIter;
use rug::Float;
use types::*;

pub const FLOAT_PRECISION: u32 = 20;

pub fn scan<T: Into<String>>(input: T) -> Vec<Token> {
    let mut scanner = Scanner::new(input);
    scanner.scan();
    scanner.output
}

struct Scanner {
    input: PeekIter<char>,
    pub output: Vec<Token>,
}

impl Scanner {
    // instantiate a new scanner
    pub fn new<T: Into<String>>(input: T) -> Scanner {
        let chars: Vec<char> = input.into().chars().collect();

        Scanner {
            input: PeekIter::new(chars.into_iter()),
            output: Vec::new(),
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
                '$' => self.scan_var_pattern(),
                '#' => self.scan_const_pattern(),
                '_' => self.scan_any_pattern(),
                _ if c.is_alphabetic() => self.scan_var(),
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
        let mut float_str: String = self.input.collect_while(|c| c.is_digit(10));
        if let Some('.') = self.input.peek() {
            float_str.push('.');
            self.input.next();
            float_str.push_str(&self.input.collect_while::<_, String>(|c| c.is_digit(10)));
        }
        let tok = Token::new(TokenType::Float(Float::with_val(FLOAT_PRECISION, Float::parse(float_str).unwrap())));
        self.output.push(tok);
    }

    fn scan_var_str(&mut self) -> String {
        self.input.collect_while(|c| c.is_alphabetic())
    }

    fn scan_var(&mut self) {
        let var_name = self.scan_var_str();
        self.output.push(Token::new(TokenType::Variable(var_name)));
    }

    fn scan_var_pattern(&mut self) {
        let mut pat = match self.input.next() {
            Some(c @ '$') => c.to_string(),
            _ => unreachable!(),
        };
        pat.push_str(&self.scan_var_str());
        self.output
            .push(Token::new(TokenType::VariablePattern(pat)));
    }

    fn scan_const_pattern(&mut self) {
        let mut pat = match self.input.next() {
            Some(c @ '#') => c.to_string(),
            _ => unreachable!(),
        };
        pat.push_str(&self.scan_var_str());
        self.output.push(Token::new(TokenType::ConstPattern(pat)));
    }

    fn scan_any_pattern(&mut self) {
        let mut pat = match self.input.next() {
            Some(c @ '_') => c.to_string(),
            _ => unreachable!(),
        };
        pat.push_str(&self.scan_var_str());
        self.output.push(Token::new(TokenType::AnyPattern(pat)));
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

                let mut tokens = scan($program)
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
            invalid_numbers: "1.2.3", "1.2 Invalid(.) 3"
            invalid_tokens: "@", "Invalid(@)"
            invalid_tokens_mixed_with_valid: "=@/", "= Invalid(@) /"
            invalid_expressions: "1 + * 2", "1 + * 2"
        }
    }
}
