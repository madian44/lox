use crate::interpreter::interpret_with_environment;
use crate::{
    interpreter::environment, interpreter::lox_type, interpreter::unwind, reporter, stmt, token,
};
use std::collections::{HashMap, LinkedList};
use std::fmt::{Debug, Display, Formatter};
use std::iter::zip;

pub struct Function {
    closure: environment::Environment,
    name: token::Token,
    params: LinkedList<token::Token>,
    body: LinkedList<stmt::Stmt>,
}

impl Function {
    pub fn build(
        closure: &environment::Environment,
        name: token::Token,
        params: LinkedList<token::Token>,
        body: LinkedList<stmt::Stmt>,
    ) -> Self {
        Self {
            closure: closure.clone(),
            name,
            params,
            body,
        }
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
        depths: &HashMap<usize, usize>,
        arguments: Vec<lox_type::LoxType>,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let closure = self.closure.clone();
        let mut environment = environment::Environment::new_with_enclosing(&closure);

        for (k, v) in zip(&self.params, arguments) {
            environment.define(&k.lexeme, v);
        }

        interpret_with_environment(reporter, depths, environment, &self.body)?;

        Ok(lox_type::LoxType::Nil)
    }
}
