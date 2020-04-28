use crate::parser::types::*;
use crate::scanner::TokenType;

trait Data {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result;
}

trait Visitor {
    type Result;
    fn visit_binop(&mut self, item: &BinOp) -> Self::Result;
    fn visit_unaryop(&mut self, item: &UnaryOp) -> Self::Result;
    fn visit_float(&mut self, item: &f64) -> Self::Result;
    fn visit_int(&mut self, item: &i64) -> Self::Result;
    fn visit_expr(&mut self, item: &Expr) -> Self::Result;
}

impl Data for f64 {
    fn accept<V: Visitor> (&self, visitor: &mut V) -> V::Result {
        visitor.visit_float(self)
    }
}

impl Data for i64 {
    fn accept<V: Visitor> (&self, visitor: &mut V) -> V::Result {
        visitor.visit_int(self)
    }
}

impl Data for BinOp {
    fn accept<V: Visitor> (&self, visitor: &mut V) -> V::Result {
        visitor.visit_binop(self)
    }
}

impl Data for UnaryOp {
    fn accept<V: Visitor> (&self, visitor: &mut V) -> V::Result {
        visitor.visit_unaryop(self)
    }
}

impl Data for Expr {
    fn accept<V: Visitor> (&self, visitor: &mut V) -> V::Result {
        visitor.visit_expr(self)
    }
}

struct Interpreter {
    input: Box<Expr>,
}

impl Visitor for Interpreter {
    type Result = f64;
    fn visit_expr(&mut self, item: &Expr) -> Self::Result {
        match item {
            BinOp => self.visit_binop(item),
            UnaryOp => self.visit_unaryop(item),
            Expr::Float(f) => self.visit_float(f),
            Expr::Int(i) => self.visit_int(i),
        }
    }

    fn visit_binop(&mut self, item: &BinOp) -> Self::Result {
        match item.op.token_type {
            TokenType::Plus => self.visit_expr(item.lhs) + self.visit_expr(item.rhs),
            TokenType::Mult => self.visit_expr(item.lhs) *  self.visit_expr(item.rhs),
            TokenType::Minus => self.visit_expr(item.lhs) - self.visit_expr(item.rhs),
            TokenType::Div => self.visit_expr(item.lhs) / self.visit_expr(item.rhs),
            TokenType::Exp => self.visit_expr(item.lhs).pow(self.visit_expr(item.rhs)),
            TokenType::Mod => self.visit_expr(item.lhs) % self.visit_expr(item.rhs),
        }
    }

    fn visit_unaryop(&mut self, item: &UnaryOp) -> Self::Result {
        match item.op.token_type {
            TokenType::Plus => self.visit_expr(item.rhs),
            TokenType::Minus => -1 * self.visit_expr(item.rhs),
        }
    }

    fn visit_float(&mut self, item: &f64) -> Self::Result {
        item
    }

    fn visit_int(&mut self, item: &i64) -> Self::Result {
        item
    }
}



