// A definition of a token - used in the scanner
// Written by Luke Bhan, 2/19/2020
//
//
#[derive(PartialEq)]
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

    // Identifies exponent - stored as a string
    Exp,

    // empty token
    Empty,
    End
}

// This will hold are token data
pub struct Token {
    token: TokenType,
    integer: i64,
    float: f64,
}

impl Token{
    pub fn new(token_i: TokenType, integer_i: i64, float_i: f64) -> Token {
        Token{
            token: token_i,
            integer: integer_i,
            float: float_i
        }
    }
    pub fn is_empty(&mut self) -> bool {
        if TokenType::Empty == self.token{
            return true;
        }
        else{
            return false;
        }
    }
}
        


