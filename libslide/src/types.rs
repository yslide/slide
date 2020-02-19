// A definition of a token - used in the scanner
// Written by Luke Bhan, 2/19/2020
//
//
enum TokenType{
    // Stores a floating point number as dp
    Num(f64),

    // Stores an int - signed
    Int(i64),

    // Stores an int - unsigned
    UInt(u64),

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
    End
}

// This will hold are token data
pub struct Token {
    token: TokenType,
    integer: i64,
    float: f64,
}




