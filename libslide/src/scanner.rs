mod types;
use types::TokenType;
use types::Token;

pub struct Scanner {
    input: String, 
    output: Vec<Token>
}

impl Scanner {
    pub fn new(input: &str) -> Scanner {
        Scanner{
            input: input.to_owned(),
            output: Vec::new()
        }
    }

   fn create_symbol_token(c: char) -> Token{
        let t: TokenType;
        match c {
            '+' => t = TokenType::Plus,
            '-' => t = TokenType::Minus,
            '*' => t = TokenType::Mult,
            '/' => t = TokenType::Div,
            '%' => t = TokenType::Mod,
            '^' => t = TokenType::Exp,
             _  => t = TokenType::Empty
        }
        let ret = Token(t, 0, 0.0);
        return ret;
    }

    fn iterate_digit(&mut self,mut i: usize,mut c: char) -> Token{
        let mut int_str = "".to_owned();
        let mut dec_str = "0.".to_owned();
        let ret: Token;
        while c.is_digit(10){
            int_str.push(c);
            i += 1;
            c = self.input.as_bytes()[i] as char;
        }
        if{c == '.'}{
            i += 1;
            c = self.input.as_bytes()[i] as char;
            while c.is_digit(10){
                dec_str.push(c);
                i += 1;
                c = self.input.as_bytes()[i] as char;
            }
            ret = Token(TokenType::Num,(int_str.parse::<i64>().unwrap()), (dec_str.parse::<f64>().unwrap()))
        }
        else {
            ret = Token(TokenType::Int, (int_str.parse::<i64>().unwrap()),0.0)
        }
        return ret;
    }

    pub fn scan(&mut self){
        let mut i: usize = 0;
        let mut c: char;
        let mut t: Token;
        while i < self.input.chars().count() {
            c = self.input.as_bytes()[i] as char;
            if !c.is_whitespace() {
                if c.is_digit(10) {
                    t = Scanner::iterate_digit(self, i, c);
                    self.output.push(t);
                }
                else{
                    t = Scanner::create_symbol_token(c);
                    if !t.is_empty() {
                        self.output.push(t);
                    }
                    else{
                        print!("Character invalid");
                    }
                    i += 1;
                }       
            }
        }
    }
    
}
        


            

        



        

