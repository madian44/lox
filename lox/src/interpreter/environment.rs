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

    fn assign_at(
        &mut self,
        depth: usize,
        name: &token::Token,
        value: lox_type::LoxType,
    ) -> Result<(), unwind::Unwind> {
        if depth > 0 && self.enclosing.is_some() {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow_mut()
                .assign_at(depth - 1, name, value)
        } else if depth == 0 && self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value.clone());
            Ok(())
        } else {
            Err(unwind::Unwind::WithError(format!(
                "Undefined variable '{}'",
                name.lexeme
            )))
        }
    }

    pub fn get_at(
        &self,
        depth: usize,
        name: &token::Token,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        if depth > 0 && self.enclosing.is_some() {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow()
                .get_at(depth - 1, name)
        } else if depth == 0 && self.values.contains_key(&name.lexeme) {
            Ok(self.values.get(&name.lexeme).cloned().unwrap())
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
    global: Rc<RefCell<Frame>>,
}

impl Environment {
    pub fn new() -> Self {
        let frame = Frame {
            enclosing: None,
            values: HashMap::new(),
        };

        let frame = Rc::new(RefCell::new(frame));
        Self {
            frame: frame.clone(),
            global: frame.clone(),
        }
    }

    pub fn new_with_enclosing(enclosing: &Environment) -> Self {
        let frame = Frame {
            enclosing: Some(enclosing.frame.clone()),
            values: HashMap::new(),
        };
        Self {
            frame: Rc::new(RefCell::new(frame)),
            global: enclosing.global.clone(),
        }
    }

    pub fn define(&mut self, name: &str, value: lox_type::LoxType) {
        self.frame.borrow_mut().define(name, value)
    }

    pub fn assign_at(
        &mut self,
        depth: Option<usize>,
        name: &token::Token,
        value: lox_type::LoxType,
    ) -> Result<(), unwind::Unwind> {
        match depth {
            None => self.global.borrow_mut().assign_at(0, name, value),
            Some(depth) => self.frame.borrow_mut().assign_at(depth, name, value),
        }
    }

    pub fn get_at(
        &self,
        depth: Option<usize>,
        name: &token::Token,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        match depth {
            None => self.global.borrow().get_at(0, name),
            Some(depth) => self.frame.borrow().get_at(depth, name),
        }
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

        let result = environment.get_at(None, &a_token);
        assert_eq!(
            result,
            Err(unwind::Unwind::WithError(
                "Undefined variable 'a'".to_string()
            ))
        );
        assert_eq!(
            Rc::strong_count(&environment.frame),
            2,
            "Unexpected ref count"
        );

        let a_initial_value = lox_type::LoxType::String("a value".to_string());
        let result = environment.assign_at(None, &a_token, a_initial_value.clone());
        assert_eq!(
            result,
            Err(unwind::Unwind::WithError(
                "Undefined variable 'a'".to_string()
            ))
        );
        assert_eq!(
            Rc::strong_count(&environment.frame),
            2,
            "Unexpected ref count"
        );

        environment.define(&a_token.lexeme, a_initial_value.clone());
        let result = environment.get_at(None, &a_token);
        assert_eq!(result, Ok(a_initial_value));
        assert_eq!(
            Rc::strong_count(&environment.frame),
            2,
            "Unexpected ref count"
        );

        let a_updated_value = lox_type::LoxType::String("a value (updated)".to_string());
        let result = environment.assign_at(None, &a_token, a_updated_value.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(
            Rc::strong_count(&environment.frame),
            2,
            "Unexpected ref count"
        );

        let result = environment.get_at(None, &a_token);
        assert_eq!(result, Ok(a_updated_value.clone()));
        assert_eq!(
            Rc::strong_count(&environment.frame),
            2,
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
            2,
            "Unexpected ref count before block"
        );

        {
            let mut nested_environment = Environment::new_with_enclosing(&original_environment);

            assert_eq!(
                Rc::strong_count(&original_environment.frame),
                4,
                "Unexpected ref count at start of block"
            );

            nested_environment.define(&a_token.lexeme, a_nested_value.clone());
            nested_environment.define(&c_token.lexeme, c_initial_value.clone());

            let result = nested_environment.assign_at(None, &b_token, b_updated_value.clone());
            assert_eq!(result, Ok(()));

            let result = nested_environment.assign_at(Some(1), &b_token, b_updated_value.clone());
            assert_eq!(result, Ok(()));

            let result = nested_environment.assign_at(Some(2), &b_token, b_updated_value.clone());
            assert_eq!(
                result,
                Err(unwind::Unwind::WithError(
                    "Undefined variable 'b'".to_string()
                ))
            );

            let result = nested_environment.get_at(Some(0), &a_token);
            assert_eq!(result, Ok(a_nested_value));

            let result = nested_environment.get_at(None, &a_token);
            assert_eq!(result, Ok(a_initial_value.clone()));

            let result = nested_environment.get_at(Some(1), &a_token);
            assert_eq!(result, Ok(a_initial_value.clone()));

            let result = nested_environment.get_at(Some(1), &b_token);
            assert_eq!(result, Ok(b_updated_value.clone()));

            let result = nested_environment.get_at(Some(0), &c_token);
            assert_eq!(result, Ok(c_initial_value));
        }

        assert_eq!(
            Rc::strong_count(&original_environment.frame),
            2,
            "Unexpected ref count after block"
        );

        let result = original_environment.get_at(Some(0), &a_token);
        assert_eq!(result, Ok(a_initial_value.clone()));

        let result = original_environment.get_at(Some(0), &b_token);
        assert_eq!(result, Ok(b_updated_value.clone()));

        let result = original_environment.get_at(Some(0), &c_token);
        assert_eq!(
            result,
            Err(unwind::Unwind::WithError(
                "Undefined variable 'c'".to_string()
            ))
        );
    }
}
