use crate::{interpreter::lox_type, interpreter::unwind, reporter};
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub trait Callable: Debug {
    fn call(
        &self,
        reporter: &dyn reporter::Reporter,
        arguments: Vec<lox_type::LoxType>,
    ) -> Result<lox_type::LoxType, unwind::Unwind>;
    fn arity(&self) -> usize;
}

pub trait NativeCallable: Debug {
    fn call(&self, arguments: Vec<lox_type::LoxType>) -> Result<lox_type::LoxType, unwind::Unwind>;
    fn arity(&self) -> usize;
}

#[derive(Debug, Clone)]
pub enum LoxType {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Function {
        name: String,
        callable: Rc<Box<dyn Callable>>,
    },
    NativeFunction {
        name: String,
        callable: Rc<Box<dyn NativeCallable>>,
    },
}

impl PartialEq for LoxType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            LoxType::Nil => matches!(other, LoxType::Nil),
            LoxType::Boolean(value) => {
                if let LoxType::Boolean(other) = other {
                    other == value
                } else {
                    false
                }
            }
            LoxType::Number(value) => {
                if let LoxType::Number(other) = other {
                    other == value
                } else {
                    false
                }
            }
            LoxType::String(value) => {
                if let LoxType::String(other) = other {
                    other == value
                } else {
                    false
                }
            }
            LoxType::Function {
                name: value,
                callable: _,
            } => {
                if let LoxType::Function {
                    name: other,
                    callable: _,
                } = other
                {
                    other == value
                } else {
                    false
                }
            }
            LoxType::NativeFunction {
                name: value,
                callable: _,
            } => {
                if let LoxType::NativeFunction {
                    name: other,
                    callable: _,
                } = other
                {
                    other == value
                } else {
                    false
                }
            }
        }
    }
}

impl Display for LoxType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxType::Nil => write!(f, "nil"),
            LoxType::Boolean(bool) => write!(f, "{bool}"),
            LoxType::Number(number) => write!(f, "{number}"),
            LoxType::String(string) => write!(f, "\"{string}\""),
            LoxType::Function { name, callable: _ } => write!(f, "\"fun {name}\""),
            LoxType::NativeFunction { name, callable: _ } => write!(f, "\"native fun {name}\""),
        }
    }
}
