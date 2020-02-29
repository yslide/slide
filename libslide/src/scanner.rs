mod types;
use  std::io;

pub struct Scanner {
    input: String, 
    output: Vec<token>
}

impl Scanner {
    pub fn new(input: &str) -> Scanner {
        Scanner{
            input: input.to_owned(),
            output: Vec::new()
        }
    }

    pub fn scan(&mut self){
        let mut i = 0;
        while i < self.input.chars().count() {
            c = self.input.as_bytest()[i] as char)
            if !(c.is_whitespace() {
                if c.is_digit() {
                    // insert digit code here
                }
                else{
                    t = create_symbol_token(c)
                    if !t.is_empty() {
                        self.output.push(c);
                    }
                    else{
                        print!("Character invalid");
                    }
                }       
            }
        }
    }
    fn create_symbol_token(c: char) -> Token{
        match c {
            '+' => let t: TokenType = TokenType::Plus,
            '-' => let t: TokenType = TokenType::Minus,
            '*' => let t: TokenType = TokenType::Mult,
            '/' => let t: TokenType = TokenType::Div,
            '%' => let t: TokenType = TokenType::Mod,
            '^' => let t: TokenType = TokenType::Exp,
             _  => let t: TokenType = TokenType::Empty
        }
        let ret = Token{token: t, integer: 0, float: 0.0};
        return ret;
    }

        

