use crate::scanner::types::Token;

pub enum Expr {
    Float(f64),
    Int(i64),
    BinOp(Box<BinOp>),
    // I added un op even though I havent implemented + or -
    UnaryOp(Box<UnaryOp>),
}

pub struct BinOp {
    pub item: Token,
    pub lhs: Expr,
    pub rhs: Expr
}

pub struct UnaryOp{
    pub item: Token, 
    pub rhs: Expr
}

pub struct Parser {
    input: Vec<Token>,
    index: usize,
}

impl Parser {
    pub fn new (input: Vec<Token>) -> Parser{
        Parser{
            input: input.into(),
            index: 0,
        }
    }

    pub fn parse(self) -> Expr {
        while self.index < self.input.len(){
            let mut cur = self.get_token();
            self.e1(cur);
            self.e1_tail(cur);
        }
        // here i need to return the head
    }

    fn e1(self, cur: Token) -> (){
        // self.e2(cur);
        // self.e2_tail(cur);
    }

    fn e1_tail(self, cur: Token) -> () {
        if(cur.token_type == TokenType::Plus){
            // add plus node to tree
        }
        else if {cur.token_type == TokenType::Minus){
            // add minus node to tree
        }
    }

    fn get_token(self) -> Token{
        self.index = self.index+1;
        return self.input[self.index-1];
    }
    
}
