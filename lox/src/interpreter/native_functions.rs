use crate::{interpreter::lox_type, interpreter::unwind};
use std::time;

#[derive(Debug)]
pub struct Clock;

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
