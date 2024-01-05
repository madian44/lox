use crate::{interpreter::lox_type, interpreter::unwind, token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

struct Frame {
    enclosing: Option<Rc<RefCell<Frame>>>,
    values: HashMap<String, lox_type::LoxType>,
}

impl Frame {
    fn define(&mut self, name: &str, value: lox_type::LoxType) {
        self.values.insert(name.to_string(), value);
    }

    fn assign(
        &mut self,
        name: &token::Token,
        value: lox_type::LoxType,
    ) -> Result<(), unwind::Unwind> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value.clone());
            Ok(())
        } else if self.enclosing.is_some() {
            let enclosing = self.enclosing.clone().unwrap();
            let mut enclosing = enclosing.borrow_mut();
            enclosing.assign(name, value)
        } else {
            Err(unwind::Unwind::WithError(format!(
                "Undefined variable '{}'",
                name.lexeme
            )))
        }
    }

    pub fn get(&self, name: &token::Token) -> Result<lox_type::LoxType, unwind::Unwind> {
        if self.values.contains_key(&name.lexeme) {
            Ok(self.values.get(&name.lexeme).cloned().unwrap())
        } else if self.enclosing.is_some() {
            let enclosing = self.enclosing.clone().unwrap();
            let enclosing = enclosing.borrow();
            enclosing.get(name)
        } else {
            Err(unwind::Unwind::WithError(format!(
                "Undefined variable '{}'",
                name.lexeme
            )))
        }
    }
}

#[derive(Clone)]
pub struct Environment {
    frame: Rc<RefCell<Frame>>,
}

impl Environment {
    pub fn new() -> Self {
        let frame = Frame {
            enclosing: None,
            values: HashMap::new(),
        };

        Self {
            frame: Rc::new(RefCell::new(frame)),
        }
    }

    pub fn new_with_enclosing(enclosing: &Environment) -> Self {
        let frame = Frame {
            enclosing: Some(enclosing.frame.clone()),
            values: HashMap::new(),
        };
        Self {
            frame: Rc::new(RefCell::new(frame)),
        }
    }

    pub fn define(&mut self, name: &str, value: lox_type::LoxType) {
        self.frame.borrow_mut().define(name, value)
    }

    pub fn assign(
        &mut self,
        name: &token::Token,
        value: lox_type::LoxType,
    ) -> Result<(), unwind::Unwind> {
        self.frame.borrow_mut().assign(name, value)
    }

    pub fn get(&self, name: &token::Token) -> Result<lox_type::LoxType, unwind::Unwind> {
        self.frame.borrow().get(name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{location, token};

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
            Err(unwind::Unwind::WithError(
                "Undefined variable 'a'".to_string()
            ))
        );
        assert_eq!(
            Rc::strong_count(&environment.frame),
            1,
            "Unexpected ref count"
        );

        let a_initial_value = lox_type::LoxType::String("a value".to_string());
        let result = environment.assign(&a_token, a_initial_value.clone());
        assert_eq!(
            result,
            Err(unwind::Unwind::WithError(
                "Undefined variable 'a'".to_string()
            ))
        );
        assert_eq!(
            Rc::strong_count(&environment.frame),
            1,
            "Unexpected ref count"
        );

        environment.define(&a_token.lexeme, a_initial_value.clone());
        let result = environment.get(&a_token);
        assert_eq!(result, Ok(a_initial_value));
        assert_eq!(
            Rc::strong_count(&environment.frame),
            1,
            "Unexpected ref count"
        );

        let a_updated_value = lox_type::LoxType::String("a value (updated)".to_string());
        let result = environment.assign(&a_token, a_updated_value.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(
            Rc::strong_count(&environment.frame),
            1,
            "Unexpected ref count"
        );

        let result = environment.get(&a_token);
        assert_eq!(result, Ok(a_updated_value.clone()));
        assert_eq!(
            Rc::strong_count(&environment.frame),
            1,
            "Unexpected ref count"
        );
    }

    #[test]
    fn test_nested_environment() {
        let mut original_environment = Environment::new();
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
        original_environment.define(&a_token.lexeme, a_initial_value.clone());
        original_environment.define(&b_token.lexeme, b_initial_value.clone());

        let a_nested_value = lox_type::LoxType::String("a value (nested)".to_string());
        let b_updated_value = lox_type::LoxType::String("b value (updated)".to_string());

        assert_eq!(
            Rc::strong_count(&original_environment.frame),
            1,
            "Unexpected ref count before block"
        );

        {
            let mut nested_environment = Environment::new_with_enclosing(&original_environment);

            assert_eq!(
                Rc::strong_count(&original_environment.frame),
                2,
                "Unexpected ref count at start of block"
            );

            nested_environment.define(&a_token.lexeme, a_nested_value.clone());
            nested_environment.define(&c_token.lexeme, c_initial_value.clone());

            let result = nested_environment.assign(&b_token, b_updated_value.clone());
            assert_eq!(result, Ok(()));

            let result = nested_environment.get(&a_token);
            assert_eq!(result, Ok(a_nested_value));

            let result = nested_environment.get(&b_token);
            assert_eq!(result, Ok(b_updated_value.clone()));

            let result = nested_environment.get(&c_token);
            assert_eq!(result, Ok(c_initial_value));
        }

        assert_eq!(
            Rc::strong_count(&original_environment.frame),
            1,
            "Unexpected ref count after block"
        );

        let result = original_environment.get(&a_token);
        assert_eq!(result, Ok(a_initial_value));

        let result = original_environment.get(&b_token);
        assert_eq!(result, Ok(b_updated_value.clone()));

        let result = original_environment.get(&c_token);
        assert_eq!(
            result,
            Err(unwind::Unwind::WithError(
                "Undefined variable 'c'".to_string()
            ))
        );
    }
}
