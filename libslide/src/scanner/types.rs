// A definition of a token - used in the scanner
// Written by Luke Bhan, 2/19/2020
//
//
#[derive(PartialEq, Clone, Debug)]
pub enum TokenType{
    // Stores a floating point number as dp
    Float(f64),

    // Stores an int - signed
    Int(i64),

    // Identifies addition 
    Plus,

    // Identifies subtraction 
    Minus, 

    // Identifies multiplication
    Mult,

    // Identifies division 
    Div,

    // Identifies modulo
    Mod,

    // Identifies exponent
    Exp,

    // Identifies an equal sign
    Equal, 

    // open parentheses (
    OpenParen,

    // close parentheses )
    CloseParen, 

    // open bracket [
    OpenBracket, 
    
    // close bracket ]
    CloseBracket,

    // invalid token
    Invalid(String)
}

#[derive(PartialEq, Clone, Debug)]
pub struct Token{
    pub token: TokenType
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_1(){
        let t = Token{token: TokenType::Invalid("s".to_string())};
        let r =  Token{token: TokenType::Invalid("s".to_string())};
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
        let t = Token{token: TokenType::Float(25.25)};
        let r = Token{token: TokenType::Float(25.25)};
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
        let t = Token{token: TokenType::Float(25.025)};
        let r = Token{token: TokenType::Float(25.25)};
        assert_ne!(true, t == r);
    }
}

