use crate::interpreter::interpret_with_environment;
use crate::{
    interpreter::environment, interpreter::lox_type, interpreter::unwind, reporter, stmt, token,
};
use std::collections::LinkedList;
use std::fmt::{Debug, Display, Formatter};
use std::iter::zip;

pub struct Function {
    name: token::Token,
    params: LinkedList<token::Token>,
    body: LinkedList<stmt::Stmt>,
}

impl Function {
    pub fn build(
        name: token::Token,
        params: LinkedList<token::Token>,
        body: LinkedList<stmt::Stmt>,
    ) -> Self {
        Self { name, params, body }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}

impl lox_type::Callable for Function {
    fn arity(&self) -> usize {
        self.params.len()
    }

    fn call(
        &self,
        reporter: &dyn reporter::Reporter,
        environment: &mut environment::Environment,
        arguments: Vec<lox_type::LoxType>,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        environment.new_frame();
        zip(&self.params, arguments).for_each(|(k, v)| environment.define(&k.lexeme, v));

        if let Err(err) = interpret_with_environment(reporter, environment, &self.body) {
            environment.pop_frame();
            return Err(err);
        }

        environment.pop_frame();

        Ok(lox_type::LoxType::Nil)
    }
}
