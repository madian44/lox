use crate::{stmt, token};
use std::collections::LinkedList;
use std::rc::Rc;

#[derive(Debug)]
struct InternalFunction {
    pub name: token::Token,
    pub params: LinkedList<token::Token>,
    pub body: LinkedList<stmt::Stmt>,
}

#[derive(Debug, Clone)]
pub struct Function {
    function: Rc<InternalFunction>,
}

impl Function {
    pub fn new(
        name: token::Token,
        params: LinkedList<token::Token>,
        body: LinkedList<stmt::Stmt>,
    ) -> Self {
        Self {
            function: Rc::new(InternalFunction { name, params, body }),
        }
    }

    pub fn name(&self) -> &token::Token {
        &self.function.name
    }

    pub fn params(&self) -> &LinkedList<token::Token> {
        &self.function.params
    }

    pub fn body(&self) -> &LinkedList<stmt::Stmt> {
        &self.function.body
    }
}
