use crate::{lox_type, runtime_error::RuntimeError, token};
use std::collections::{HashMap, LinkedList};

pub struct Frame {
    values: HashMap<String, lox_type::LoxType>,
}

impl Frame {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    fn define(&mut self, name: &str, value: &lox_type::LoxType) {
        self.values.insert(name.to_string(), value.clone());
    }

    fn get(&self, name: &token::Token) -> Option<lox_type::LoxType> {
        self.values.get(&name.lexeme).cloned()
    }

    fn assign(&mut self, name: &token::Token, value: &lox_type::LoxType) -> bool {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value.clone());
            true
        } else {
            false
        }
    }
}

pub struct Environment {
    frames: LinkedList<Frame>,
}

impl Environment {
    pub fn new() -> Self {
        let mut frames = LinkedList::new();
        frames.push_front(Frame::new());
        Self { frames }
    }

    pub fn new_frame(&mut self) {
        self.frames.push_front(Frame::new());
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop_front();
    }

    pub fn define(&mut self, name: &str, value: &lox_type::LoxType) {
        self.frames.front_mut().unwrap().define(name, value)
    }

    pub fn assign(
        &mut self,
        name: &token::Token,
        value: &lox_type::LoxType,
    ) -> Result<(), RuntimeError> {
        for frame in &mut self.frames {
            if frame.assign(name, value) {
                return Ok(());
            }
        }
        Err(RuntimeError {
            message: format!("Undefined variable '{}'", name.lexeme),
        })
    }

    pub fn get(&self, name: &token::Token) -> Result<lox_type::LoxType, RuntimeError> {
        for frame in &self.frames {
            if let Some(value) = frame.get(name) {
                return Ok(value);
            }
        }

        Err(RuntimeError {
            message: format!("Undefined variable '{}'", name.lexeme),
        })
    }
}
#[cfg(test)]
mod test {

    use super::*;
    use crate::{location, lox_type, token};

    #[test]
    fn test_environment() {
        let mut environment = Environment::new();
        let blank_location = location::FileLocation::new(0, 0);
        let a_token = token::Token::new(
            token::TokenType::Identifier,
            "a",
            blank_location,
            blank_location,
            None,
        );

        let result = environment.get(&a_token);
        assert_eq!(
            result,
            Err(RuntimeError {
                message: "Undefined variable 'a'".to_string()
            })
        );

        let a_initial_value = lox_type::LoxType::String("a value".to_string());
        let result = environment.assign(&a_token, &a_initial_value);
        assert_eq!(
            result,
            Err(RuntimeError {
                message: "Undefined variable 'a'".to_string()
            })
        );

        environment.define(&a_token.lexeme, &a_initial_value);
        let result = environment.get(&a_token);
        assert_eq!(result, Ok(a_initial_value));

        let a_updated_value = lox_type::LoxType::String("a value (updated)".to_string());
        let result = environment.assign(&a_token, &a_updated_value);
        assert_eq!(result, Ok(()));

        let result = environment.get(&a_token);
        assert_eq!(result, Ok(a_updated_value));
    }

    #[test]
    fn test_nested_environment() {
        let mut environment = Environment::new();
        let blank_location = location::FileLocation::new(0, 0);
        let a_token = token::Token::new(
            token::TokenType::Identifier,
            "a",
            blank_location,
            blank_location,
            None,
        );

        let b_token = token::Token::new(
            token::TokenType::Identifier,
            "b",
            blank_location,
            blank_location,
            None,
        );

        let c_token = token::Token::new(
            token::TokenType::Identifier,
            "c",
            blank_location,
            blank_location,
            None,
        );

        let a_initial_value = lox_type::LoxType::String("a value".to_string());
        let b_initial_value = lox_type::LoxType::String("b value".to_string());
        let c_initial_value = lox_type::LoxType::String("c value".to_string());
        environment.define(&a_token.lexeme, &a_initial_value);
        environment.define(&b_token.lexeme, &b_initial_value);

        environment.new_frame();
        let a_nested_value = lox_type::LoxType::String("a value (nested)".to_string());
        let b_updated_value = lox_type::LoxType::String("b value (updated)".to_string());
        environment.define(&a_token.lexeme, &a_nested_value);
        environment.define(&c_token.lexeme, &c_initial_value);

        let result = environment.assign(&b_token, &b_updated_value);
        assert_eq!(result, Ok(()));

        let result = environment.get(&a_token);
        assert_eq!(result, Ok(a_nested_value));

        let result = environment.get(&b_token);
        assert_eq!(result, Ok(b_updated_value.clone()));

        let result = environment.get(&c_token);
        assert_eq!(result, Ok(c_initial_value));

        environment.pop_frame();

        let result = environment.get(&a_token);
        assert_eq!(result, Ok(a_initial_value));

        let result = environment.get(&b_token);
        assert_eq!(result, Ok(b_updated_value.clone()));

        let result = environment.get(&c_token);
        assert_eq!(
            result,
            Err(RuntimeError {
                message: "Undefined variable 'c'".to_string()
            })
        );
    }
}
