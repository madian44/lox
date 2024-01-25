pub mod function;
use crate::{expr, stmt, token};
use std::collections::LinkedList;

#[derive(Debug)]
pub enum Stmt {
    Block {
        statements: LinkedList<Stmt>,
    },
    Class {
        name: token::Token,
        superclass: Option<expr::Expr>,
        methods: LinkedList<Stmt>,
    },
    Expression {
        expression: expr::Expr,
    },
    Function {
        function: function::Function,
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

impl Stmt {
    pub fn new_function(
        name: token::Token,
        params: LinkedList<token::Token>,
        body: LinkedList<Stmt>,
    ) -> Self {
        stmt::Stmt::Function {
            function: function::Function::new(name, params, body),
        }
    }
}
