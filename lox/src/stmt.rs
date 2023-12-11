use crate::{expr, token};
use std::collections::LinkedList;

pub enum Stmt {
    Block {
        statements: LinkedList<Stmt>,
    },
    Expression {
        expression: expr::Expr,
    },
    If {
        condition: expr::Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    },
    Print {
        value: expr::Expr,
    },
    Var {
        name: token::Token,
        initialiser: Option<expr::Expr>,
    },
    While {
        condition: expr::Expr,
        body: Box<Stmt>,
    },
}
