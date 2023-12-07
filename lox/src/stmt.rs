use crate::{expr, token};
use std::collections::LinkedList;

pub enum Stmt {
    Expression {
        expression: expr::Expr,
    },
    Print {
        value: expr::Expr,
    },
    Var {
        name: token::Token,
        initialiser: Option<expr::Expr>,
    },
    Block {
        statements: LinkedList<Stmt>,
    },
}
