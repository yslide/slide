mod types;
use types::TokenType;
use types::Token;

pub struct Scanner {
    input: String, 
    pub output: Vec<Token>
}

impl Scanner {
    // instantiate a new scanner
    pub fn new(input: &str) -> Scanner {
        Scanner{
            input: input.to_owned(),
            output: Vec::new()
        }
    }

    // matches token with symbol and creates it: private helper function 
   fn create_symbol_token(c: char) -> Token{
        let t: TokenType;
        match c {
            '+' => t = TokenType::Plus,
            '-' => t = TokenType::Minus,
            '*' => t = TokenType::Mult,
            '/' => t = TokenType::Div,
            '%' => t = TokenType::Mod,
            '^' => t = TokenType::Exp,
            '=' => t = TokenType::Equal,
            '(' => t = TokenType::OpenParen, 
            ')' => t = TokenType::CloseParen, 
            '[' => t = TokenType::OpenBra, 
            ']' => t = TokenType::CloseBra,
             _  => t = TokenType::Empty
        }
        let ret = Token{token: t, ..Default::default()};
        return ret;
   }

    // iterates through any digits to create a token of that value: private helper method
    fn iterate_digit(&mut self,mut i: usize,mut c: char) -> (Token, usize){
        let mut int_str = "".to_owned();
        let mut dec_str = "0.".to_owned();
        let ret: Token;
        // iterate through integer part
        while c.is_digit(10) && i < self.input.chars().count() {
            int_str.push(c);
            i += 1;
            if i < self.input.chars().count() {
                c = self.input.as_bytes()[i] as char;
            }
        }
        // iterate through decimal
        if{c == '.'}{
            i += 1;
            c = self.input.as_bytes()[i] as char;
            while c.is_digit(10) && i < self.input.chars().count(){
                dec_str.push(c);
                i += 1;
                if i < self.input.chars().count() {
                    c = self.input.as_bytes()[i] as char;
                }
            }
            // turn integer and decmial strings into token
            ret = Token{token: TokenType::Num, integer: int_str.parse::<i64>().unwrap(), float: dec_str.parse::<f64>().unwrap()}
        }
        else{
            // turn integer string into token and default the float
            ret = Token{token:TokenType::Int, integer: int_str.parse::<i64>().unwrap(), ..Default::default()}
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
            c = self.input.as_bytes()[i] as char;
            // ignore whitespace
            if !c.is_whitespace() {
                // check for digit and call correct helper function 
                if c.is_digit(10) {
                    tuple = Scanner::iterate_digit(self, i, c);
                    i = tuple.1;
                    self.output.push(tuple.0);
                }
                else{
                    t = Scanner::create_symbol_token(c);
                    if !t.is_empty() {
                        self.output.push(t);
                    }
                    // throw error if token is not correct
                    else{
                        panic!("Character invalid");
                    }
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

    fn compare_vec<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool{
        let equal = a.iter().zip(b.iter()).filter(|&(a,b)| a == b).count();
        equal == a.len() && equal == b.len()
    }

    #[test]
    fn test_empty_string_scan() {
        let mut s = Scanner::new("");
        s.scan();
        let result = Vec::<Token>::new();
        assert_eq!(true, compare_vec::<Token>(&result, &s.output));
    }

    #[test]
    fn test_equal_string_scan() {
        let mut s = Scanner::new("=");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token: TokenType::Equal, ..Default::default()});
        assert_eq!(true, compare_vec::<Token>(&result, &s.output));
    }

    #[test]
    fn test_equal_string_scan_with_whitespace(){
        let mut s = Scanner::new("     =             ");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token: TokenType::Equal, ..Default::default()});
        assert_eq!(true, compare_vec::<Token>(&result, &s.output));
    }

    #[test]
    fn test_integer_scan(){
        let mut s = Scanner::new("2");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token: TokenType::Int, integer: 2, ..Default::default()});
        assert_eq!(true, compare_vec::<Token>(&result, &s.output));
    }

    #[test]
    fn test_multi_digit_integer_scan(){
        let mut s = Scanner::new("22233355567");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token: TokenType::Int, integer: 22233355567, ..Default::default()});
        assert_eq!(true, compare_vec::<Token>(&result, &s.output));
    }

    #[test]
    fn test_two_digit_integer_scan() {
        let mut s = Scanner::new("2 3 45 3");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token: TokenType::Int, integer: 2, ..Default::default()});
        result.push(Token{token: TokenType::Int, integer: 3, ..Default::default()});
        result.push(Token{token: TokenType::Int, integer: 45, ..Default::default()});
        result.push(Token{token: TokenType::Int, integer: 3, ..Default::default()});
        assert_eq!(true, compare_vec::<Token>(&result, &s.output));
    }

    #[test]
    fn test_decimal_scan() {
        let mut s = Scanner::new("253.253");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token: TokenType::Num, integer: 253, float: 0.253});
        assert_eq!(true, compare_vec::<Token>(&result, &s.output));
    }

    #[test]
    fn test_multi_decimal_scan() {
        let mut s = Scanner::new("2.2 3.3 33.44");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token: TokenType::Num, integer: 2, float: 0.2});
        result.push(Token{token: TokenType::Num, integer: 3, float: 0.3});
        result.push(Token{token: TokenType::Num, integer: 33, float: 0.44});
        assert_eq!(true, compare_vec::<Token>(&result, &s.output));
    }

    #[test]
    fn test_decimal_integer_symbol_scan(){
        let mut s = Scanner::new("2.2 + 5 = 3.3 - 6.6 + 27 /( 2 ^ 5 ) * [2%2]");
        s.scan();
        let mut result = Vec::<Token>::new();
        result.push(Token{token: TokenType::Num, integer: 2, float: 0.2});
        result.push(Token{token: TokenType::Plus, ..Default::default()});
        result.push(Token{token: TokenType::Int, integer: 5, ..Default::default()});
        result.push(Token{token: TokenType::Equal, ..Default::default()});
        result.push(Token{token: TokenType::Num, integer: 3, float: 0.3});
        result.push(Token{token: TokenType::Minus, ..Default::default()});
        result.push(Token{token: TokenType::Num, integer: 6, float: 0.6});
        result.push(Token{token: TokenType::Plus, ..Default::default()});
        result.push(Token{token: TokenType::Int, integer: 27, ..Default::default()});
        result.push(Token{token: TokenType::Div, ..Default::default()});
        result.push(Token{token: TokenType::OpenParen, ..Default::default()});
        result.push(Token{token: TokenType::Int, integer: 2, ..Default::default()});
        result.push(Token{token: TokenType::Exp, ..Default::default()});
        result.push(Token{token: TokenType::Int, integer: 5, ..Default::default()});
        result.push(Token{token: TokenType::CloseParen, ..Default::default()});
        result.push(Token{token: TokenType::Mult, ..Default::default()});
        result.push(Token{token: TokenType::OpenBra, ..Default::default()});
        result.push(Token{token: TokenType::Int, integer: 2, ..Default::default()});
        result.push(Token{token: TokenType::Mod, ..Default::default()});
        result.push(Token{token: TokenType::Int, integer: 2, ..Default::default()});
        result.push(Token{token: TokenType::CloseBra, ..Default::default()});
        assert_eq!(true, compare_vec::<Token>(&result, &s.output));
    }    
}


            

        



        

