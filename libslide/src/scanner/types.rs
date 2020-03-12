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

    // Identifies exponent - stored as a char
    Exp,

    // Identifies an equal sign - stored as a char
    Equal, 

    // empty token
    Empty,
    End
}

// This will hold are token data
pub struct Token {
    pub token: TokenType,
    pub integer: i64,
    pub float: f64,
}

impl Token{
    pub fn is_empty(&mut self) -> bool {
        if TokenType::Empty == self.token{
            return true;
        }
        else{
            return false;
        }
    }
}
        


