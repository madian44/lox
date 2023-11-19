use crate::token;

pub enum Expr<'s> {
    Binary(Box<Expr<'s>>, token::Token<'s>, Box<Expr<'s>>),
    Grouping(Box<Expr<'s>>),
    Literal(token::Literal<'s>),
    Unary(token::Token<'s>, Box<Expr<'s>>),
}

impl<'s> Expr<'s> {
    pub fn binary(left: Expr<'s>, operator: token::Token<'s>, right: Expr<'s>) -> Expr<'s> {
        Expr::Binary(Box::new(left), operator, Box::new(right))
    }

    pub fn grouping(expression: Expr<'s>) -> Expr<'s> {
        Expr::Grouping(Box::new(expression))
    }

    pub fn literal(value: token::Literal<'s>) -> Expr<'s> {
        Expr::Literal(value)
    }

    pub fn unary(operator: token::Token<'s>, right: Expr<'s>) -> Expr<'s> {
        Expr::Unary(operator, Box::new(right))
    }
}
