// A definition of a token - used in the scanner
// Written by Luke Bhan, 2/19/2020
//
//
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TokenType{
    // Stores a floating point number as dp
    Num,

    // Stores an int - signed
    Int,

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
    pub integer: i64,
    pub float: f64,
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
        Token {token: TokenType::Empty, integer: Default::default(), float: Default::default()}
    }
}

impl PartialEq for Token{
    fn eq(&self, other: &Self) -> bool{
        self.token == other.token && self.integer == other.integer && self.float == other.float
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
        let t = Token{token: TokenType::Exp, ..Default::default()};
        assert_ne!(true, t.is_empty());
    }

    #[test]
    fn test_default_int(){
        let t: Token = Default::default();
        assert_eq!(0, t.integer);
    }

    #[test]
    fn test_default_float(){
        let t: Token = Default::default();
        assert_eq!(0.0, t.float);
    }

    #[test]
    fn test_eq_1(){
        let t: Token = Default::default();
        let r: Token = Default::default();
        assert_eq!(true, t == r);
    }

    #[test]
    fn test_eq_2() {
        let t = Token{token: TokenType::Plus, ..Default::default()};
        let r = Token{token: TokenType::Plus, ..Default::default()};
        assert_eq!(true, t == r);
    }

    #[test] 
    fn test_eq_3() {
        let t = Token{token: TokenType::Num, integer: 25, float: 0.25};
        let r = Token{token: TokenType::Num, integer: 25, float: 0.25};
        assert_eq!(true, t == r);
    }

    #[test]
    fn test_neq_1(){
        let t = Token{token: TokenType::Plus, ..Default::default()};
        let r = Token{token: TokenType::Minus, ..Default::default()};
        assert_ne!(true, t == r);
    }

    #[test]
    fn test_neq_2(){
        let t = Token{token: TokenType::Num, integer: 25, float: 0.025};
        let r = Token{token: TokenType::Num, integer: 25, float: 0.25};
        assert_ne!(true, t == r);
    }
}

