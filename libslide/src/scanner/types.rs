// A definition of a token - used in the scanner
// Written by Luke Bhan, 2/19/2020
//
//
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TokenType{
    // Stores a floating point number as dp
    Num(f64),

    // Stores an int - signed
    Int(i64),

    // Identifies addition - stored as a char
    Plus,

    // Identifies subtraction - stored as a char
    Minus, 

    // Identifies multiplication - stored as a char
    Mult,

    // Identifies division - stored as a char
    Div,

    // Identifies modulo - stored as a char
    Mod,

    // Identifies exponent - stored as a char
    Exp,

    // Identifies an equal sign - stored as a char
    Equal, 

    // open parentheses (
    OpenParen,

    // close parentheses )
    CloseParen, 

    // open bracket [
    OpenBra, 
    
    // close bracket ]
    CloseBra,

    // empty token
    Empty
}

// This will hold are token data
#[derive(Copy, Clone, Debug)]
pub struct Token {
    pub token: TokenType,
}

impl Token{
    pub fn is_empty(self) -> bool {
        if TokenType::Empty == self.token{
            return true;
        }
        else{
            return false;
        }
    }
}

impl Default for Token{
    fn default() -> Token{
        Token {token: TokenType::Empty}
    }
}

impl PartialEq for Token{
    fn eq(&self, other: &Self) -> bool{
        self.token == other.token
    }
}
        
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_1() {
        let t: Token = Default::default();
        assert_eq!(true, t.is_empty());
    }    

    #[test]
    fn test_empty_2() {
        let t = Token{token: TokenType::Exp};
        assert_ne!(true, t.is_empty());
    }

    #[test]
    fn test_eq_1(){
        let t: Token = Default::default();
        let r: Token = Default::default();
        assert_eq!(true, t == r);
    }

    #[test]
    fn test_eq_2() {
        let t = Token{token: TokenType::Plus};
        let r = Token{token: TokenType::Plus};
        assert_eq!(true, t == r);
    }

    #[test] 
    fn test_eq_3() {
        let t = Token{token: TokenType::Num(25.25)};
        let r = Token{token: TokenType::Num(25.25)};
        assert_eq!(true, t == r);
    }

    #[test]
    fn test_neq_1(){
        let t = Token{token: TokenType::Plus};
        let r = Token{token: TokenType::Minus};
        assert_ne!(true, t == r);
    }

    #[test]
    fn test_neq_2(){
        let t = Token{token: TokenType::Num(25.025)};
        let r = Token{token: TokenType::Num(25.25)};
        assert_ne!(true, t == r);
    }
}

