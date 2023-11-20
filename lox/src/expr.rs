use crate::token;

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: token::Literal,
    },
    Unary {
        operator: token::Token,
        right: Box<Expr>,
    },
}

impl Expr {
    pub fn build_binary(left: Expr, operator: token::Token, right: Expr) -> Self {
        Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn build_grouping(expression: Expr) -> Self {
        Expr::Grouping {
            expression: Box::new(expression),
        }
    }

    pub fn build_literal(value: token::Literal) -> Self {
        Expr::Literal { value }
    }

    pub fn build_unary(operator: token::Token, right: Expr) -> Self {
        Expr::Unary {
            operator,
            right: Box::new(right),
        }
    }
}
