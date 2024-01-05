use crate::{expr, token};
use std::collections::LinkedList;

#[derive(Clone, Debug)]
pub enum Stmt {
    Block {
        statements: LinkedList<Stmt>,
    },
    Expression {
        expression: expr::Expr,
    },
    Function {
        name: token::Token,
        params: LinkedList<token::Token>,
        body: LinkedList<Stmt>,
    },
    If {
        condition: expr::Expr,
        then_branch: Box<Stmt>,
        else_branch: Box<Option<Stmt>>,
    },
    Print {
        value: expr::Expr,
    },
    Return {
        keyword: token::Token,
        value: Option<expr::Expr>,
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
