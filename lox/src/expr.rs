use crate::token;
use std::sync::atomic::{AtomicUsize, Ordering};

static ID_SRC: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone, Debug)]
pub enum Expr {
    Assign {
        id: usize,
        name: token::Token,
        value: Box<Expr>,
    },
    Binary {
        id: usize,
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Call {
        id: usize,
        callee: Box<Expr>,
        paren: token::Token,
        arguments: Vec<Expr>,
    },
    Grouping {
        id: usize,
        expression: Box<Expr>,
    },
    Literal {
        id: usize,
        value: token::Token,
    },
    Logical {
        id: usize,
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Unary {
        id: usize,
        operator: token::Token,
        right: Box<Expr>,
    },
    Variable {
        id: usize,
        name: token::Token,
    },
}

impl Expr {
    fn get_id() -> usize {
        ID_SRC.fetch_add(1, Ordering::Relaxed)
    }

    pub fn build_assign(name: token::Token, value: Expr) -> Self {
        Expr::Assign {
            id: Expr::get_id(),
            name,
            value: Box::new(value),
        }
    }

    pub fn build_binary(left: Expr, operator: token::Token, right: Expr) -> Self {
        Expr::Binary {
            id: Expr::get_id(),
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn build_call(callee: Expr, paren: token::Token, arguments: Vec<Expr>) -> Self {
        Expr::Call {
            id: Expr::get_id(),
            callee: Box::new(callee),
            paren,
            arguments,
        }
    }

    pub fn build_grouping(expression: Expr) -> Self {
        Expr::Grouping {
            id: Expr::get_id(),
            expression: Box::new(expression),
        }
    }

    pub fn build_literal(value: token::Token) -> Self {
        Expr::Literal {
            id: Expr::get_id(),
            value,
        }
    }

    pub fn build_logical(left: Expr, operator: token::Token, right: Expr) -> Self {
        Expr::Logical {
            id: Expr::get_id(),
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn build_unary(operator: token::Token, right: Expr) -> Self {
        Expr::Unary {
            id: Expr::get_id(),
            operator,
            right: Box::new(right),
        }
    }

    pub fn build_variable(name: token::Token) -> Self {
        Expr::Variable {
            id: Expr::get_id(),
            name,
        }
    }
}
