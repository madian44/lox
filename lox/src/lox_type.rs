use crate::native_functions;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum LoxType {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    NativeFunction {
        name: String,
        callable: Rc<Box<dyn native_functions::Callable>>,
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
            LoxType::NativeFunction { name, callable: _ } => write!(f, "\"fun {name}\""),
        }
    }
}
