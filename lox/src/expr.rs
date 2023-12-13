use crate::token;

#[derive(Debug)]
pub enum Expr {
    Assign {
        name: token::Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: token::Token,
        arguments: Vec<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: token::Token,
    },
    Logical {
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Unary {
        operator: token::Token,
        right: Box<Expr>,
    },
    Variable {
        name: token::Token,
    },
}

impl Expr {
    pub fn build_assign(name: token::Token, value: Expr) -> Self {
        Expr::Assign {
            name,
            value: Box::new(value),
        }
    }

    pub fn build_binary(left: Expr, operator: token::Token, right: Expr) -> Self {
        Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn build_call(callee: Expr, paren: token::Token, arguments: Vec<Expr>) -> Self {
        Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        }
    }

    pub fn build_grouping(expression: Expr) -> Self {
        Expr::Grouping {
            expression: Box::new(expression),
        }
    }

    pub fn build_literal(value: token::Token) -> Self {
        Expr::Literal { value }
    }

    pub fn build_logical(left: Expr, operator: token::Token, right: Expr) -> Self {
        Expr::Logical {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn build_unary(operator: token::Token, right: Expr) -> Self {
        Expr::Unary {
            operator,
            right: Box::new(right),
        }
    }
    pub fn build_variable(name: token::Token) -> Self {
        Expr::Variable { name }
    }
}
