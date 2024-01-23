use crate::{interpreter::lox_type, interpreter::unwind};
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
        name: &str,
        value: lox_type::LoxType,
    ) -> Result<(), unwind::Unwind> {
        if depth > 0 && self.enclosing.is_some() {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow_mut()
                .assign_at(depth - 1, name, value)
        } else if depth == 0 && self.values.contains_key(name) {
            self.values.insert(name.to_string(), value.clone());
            Ok(())
        } else {
            Err(unwind::Unwind::WithError(format!(
                "Undefined variable '{}'",
                name
            )))
        }
    }

    fn get_at(&self, depth: usize, name: &str) -> Result<lox_type::LoxType, unwind::Unwind> {
        if depth > 0 && self.enclosing.is_some() {
            self.enclosing
                .as_ref()
                .unwrap()
                .borrow()
                .get_at(depth - 1, name)
        } else if depth == 0 && self.values.contains_key(name) {
            Ok(self.values.get(name).cloned().unwrap())
        } else {
            Err(unwind::Unwind::WithError(format!(
                "Undefined variable '{}'",
                name
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
        name: &str,
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
        name: &str,
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

    #[test]
    fn test_environment() {
        let mut environment = Environment::new();
        let a_key = "a";

        let result = environment.get_at(None, a_key);
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
        let result = environment.assign_at(None, a_key, a_initial_value.clone());
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

        environment.define(a_key, a_initial_value.clone());
        let result = environment.get_at(None, a_key);
        assert_eq!(result, Ok(a_initial_value));
        assert_eq!(
            Rc::strong_count(&environment.frame),
            2,
            "Unexpected ref count"
        );

        let a_updated_value = lox_type::LoxType::String("a value (updated)".to_string());
        let result = environment.assign_at(None, a_key, a_updated_value.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(
            Rc::strong_count(&environment.frame),
            2,
            "Unexpected ref count"
        );

        let result = environment.get_at(None, a_key);
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
        let a_key = "a";
        let b_key = "b";
        let c_key = "c";

        let a_initial_value = lox_type::LoxType::String("a value".to_string());
        let b_initial_value = lox_type::LoxType::String("b value".to_string());
        let c_initial_value = lox_type::LoxType::String("c value".to_string());
        original_environment.define(a_key, a_initial_value.clone());
        original_environment.define(b_key, b_initial_value.clone());

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

            nested_environment.define(a_key, a_nested_value.clone());
            nested_environment.define(c_key, c_initial_value.clone());

            let result = nested_environment.assign_at(None, b_key, b_updated_value.clone());
            assert_eq!(result, Ok(()));

            let result = nested_environment.assign_at(Some(1), b_key, b_updated_value.clone());
            assert_eq!(result, Ok(()));

            let result = nested_environment.assign_at(Some(2), b_key, b_updated_value.clone());
            assert_eq!(
                result,
                Err(unwind::Unwind::WithError(
                    "Undefined variable 'b'".to_string()
                ))
            );

            let result = nested_environment.get_at(Some(0), a_key);
            assert_eq!(result, Ok(a_nested_value));

            let result = nested_environment.get_at(None, a_key);
            assert_eq!(result, Ok(a_initial_value.clone()));

            let result = nested_environment.get_at(Some(1), a_key);
            assert_eq!(result, Ok(a_initial_value.clone()));

            let result = nested_environment.get_at(Some(1), b_key);
            assert_eq!(result, Ok(b_updated_value.clone()));

            let result = nested_environment.get_at(Some(0), c_key);
            assert_eq!(result, Ok(c_initial_value));
        }

        assert_eq!(
            Rc::strong_count(&original_environment.frame),
            2,
            "Unexpected ref count after block"
        );

        let result = original_environment.get_at(Some(0), a_key);
        assert_eq!(result, Ok(a_initial_value.clone()));

        let result = original_environment.get_at(Some(0), b_key);
        assert_eq!(result, Ok(b_updated_value.clone()));

        let result = original_environment.get_at(Some(0), c_key);
        assert_eq!(
            result,
            Err(unwind::Unwind::WithError(
                "Undefined variable 'c'".to_string()
            ))
        );
    }
}
