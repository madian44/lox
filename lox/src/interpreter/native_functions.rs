use crate::{interpreter::lox_type, interpreter::unwind};
use std::rc::Rc;
use std::time;

#[derive(Debug)]
struct Clock;

impl lox_type::NativeCallable for Clock {
    fn call(&self, _: Vec<lox_type::LoxType>) -> Result<lox_type::LoxType, unwind::Unwind> {
        Ok(lox_type::LoxType::Number(
            time::SystemTime::now()
                .duration_since(time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        ))
    }

    fn arity(&self) -> usize {
        0
    }
}

pub fn clock() -> lox_type::LoxType {
    lox_type::LoxType::NativeFunction {
        name: "clock".to_string(),
        callable: Rc::new(Box::new(Clock)),
    }
}
