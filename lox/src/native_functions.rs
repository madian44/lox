use crate::lox_type;
use crate::runtime_error::RuntimeError;
use std::{fmt, time};

pub trait Callable: fmt::Debug {
    fn call(&self, arguments: Vec<lox_type::LoxType>) -> Result<lox_type::LoxType, RuntimeError>;
    fn arity(&self) -> usize;
}

#[derive(Debug)]
pub struct Clock;

impl Callable for Clock {
    fn call(&self, _: Vec<lox_type::LoxType>) -> Result<lox_type::LoxType, RuntimeError> {
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
