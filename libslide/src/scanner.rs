mod types;
use types::TokenType;
use types::Token;

pub struct Scanner {
    input: String, 
    pub output: Vec<Token>
}

impl Scanner {
    // instantiate a new scanner
    pub fn new<T: Into<String>>(input: T) -> Scanner {
        Scanner{
            input: input.into(),
            output: Vec::new()
        }
    }

    // matches token with symbol and creates it: private helper function 
   fn create_symbol_token(c: char) -> Token{
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
             _  => TokenType::Invalid(c.to_string())
        };
        let ret = Token{token_type: t};
        return ret;
   }

    // iterates through any digits to create a token of that value
    fn iterate_digit(&mut self,mut i: usize) -> (Token, usize){
        let mut int_str = "".to_owned();
        let mut dec_str = ".".to_owned();
        let ret: Token;
        // iterate through integer part
        while i < self.input.chars().count() && (self.input.as_bytes()[i] as char).is_digit(10){
            int_str.push(self.input.as_bytes()[i] as char);
            i += 1;
        }
        // iterate through decimal
        if i < self.input.chars().count() && (self.input.as_bytes()[i] as char)== '.'{
            i += 1;
            while i < self.input.chars().count() && (self.input.as_bytes()[i] as char).is_digit(10){
                dec_str.push(self.input.as_bytes()[i] as char);
                i += 1;
            }
            int_str.push_str(&dec_str);
            // turn integer and decmial strings into token
            ret = Token{token_type: TokenType::Float(int_str.parse::<f64>().unwrap())}
        }
        else{
            // turn integer string into token and default the float
            ret = Token{token_type: TokenType::Int(int_str.parse::<i64>().unwrap())}
        }
        return (ret, i);
    }

    pub fn scan(&mut self){
        let mut i: usize = 0;
        let mut c: char;
        let mut t: Token;
        let mut tuple: (Token, usize);
        // iterate through string
        while i < self.input.chars().count() {
            // ignore whitespace
            if !((self.input.as_bytes()[i] as char).is_whitespace()) {
                // check for digit and call correct helper function 
                if (self.input.as_bytes()[i] as char).is_digit(10) {
                    tuple = Scanner::iterate_digit(self, i);
                    i = tuple.1;
                    self.output.push(tuple.0);
                }
                else{
                    self.output.push(Scanner::create_symbol_token(self.input.as_bytes()[i] as char));
                    i += 1;
                }       
            }
            else{
                i += 1;
            }
        }
    }
    
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_empty_string_scan() {
        let mut s = Scanner::new("");
        s.scan();
        let result = Vec::<Token>::new();
        assert_eq!(true, result==s.output);
    }

    #[test]
    fn test_equal_string_scan() {
        let mut s = Scanner::new("=");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token_type: TokenType::Equal});
        assert_eq!(true, result==s.output);
    }

    #[test]
    fn test_equal_string_scan_with_whitespace(){
        let mut s = Scanner::new("     =             ");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token_type: TokenType::Equal});
        assert_eq!(true, result==s.output);
    }

    #[test]
    fn test_integer_scan(){
        let mut s = Scanner::new("2");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token_type: TokenType::Int(2)});
        assert_eq!(true, result==s.output);
    }

    #[test]
    fn test_multi_digit_integer_scan(){
        let mut s = Scanner::new("22233355567");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token_type: TokenType::Int(22233355567)});
        assert_eq!(true, result==s.output);
    }

    #[test]
    fn test_two_digit_integer_scan() {
        let mut s = Scanner::new("2 3 45 3");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token_type: TokenType::Int(2)});
        result.push(Token{token_type: TokenType::Int(3)});
        result.push(Token{token_type: TokenType::Int(45)});
        result.push(Token{token_type: TokenType::Int(3)});
        assert_eq!(true, result==s.output);
    }

    #[test]
    fn test_decimal_scan() {
        let mut s = Scanner::new("253.253");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token_type: TokenType::Float(253.253)});
        assert_eq!(true, result==s.output);
    }

    #[test]
    fn test_multi_decimal_scan() {
        let mut s = Scanner::new("2.2 3.3 33.44");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token_type: TokenType::Float(2.2)});
        result.push(Token{token_type: TokenType::Float(3.3)});
        result.push(Token{token_type: TokenType::Float(33.44)});
        assert_eq!(true, result==s.output);
    }

    #[test]
    fn test_decimal_integer_symbol_scan(){
        let mut s = Scanner::new("2.2 + 5 = 3.3 - 6.6 + 27 /( 2 ^ 5 ) * [2%2]");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token_type: TokenType::Float(2.2)});
        result.push(Token{token_type: TokenType::Plus});
        result.push(Token{token_type: TokenType::Int(5)});
        result.push(Token{token_type: TokenType::Equal});
        result.push(Token{token_type: TokenType::Float(3.3)});
        result.push(Token{token_type: TokenType::Minus});
        result.push(Token{token_type: TokenType::Float(6.6)});
        result.push(Token{token_type: TokenType::Plus});
        result.push(Token{token_type: TokenType::Int(27)});
        result.push(Token{token_type: TokenType::Div});
        result.push(Token{token_type: TokenType::OpenParen});
        result.push(Token{token_type: TokenType::Int(2)});
        result.push(Token{token_type: TokenType::Exp});
        result.push(Token{token_type: TokenType::Int(5)});
        result.push(Token{token_type: TokenType::CloseParen});
        result.push(Token{token_type: TokenType::Mult});
        result.push(Token{token_type: TokenType::OpenBracket});
        result.push(Token{token_type: TokenType::Int(2)});
        result.push(Token{token_type: TokenType::Mod});
        result.push(Token{token_type: TokenType::Int(2)});
        result.push(Token{token_type: TokenType::CloseBracket});
        assert_eq!(true, result==s.output);
    }    
} 
        
