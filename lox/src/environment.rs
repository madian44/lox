use crate::{lox_type, runtime_error::RuntimeError, token};
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, lox_type::LoxType>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: &lox_type::LoxType) {
        self.values.insert(name, value.clone());
    }

    pub fn get(&self, name: &token::Token) -> Result<lox_type::LoxType, RuntimeError> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            _ => Err(RuntimeError {
                message: format!("Undefined variable '{}'", name.lexeme),
            }),
        }
    }
}
