use crate::{
    interpreter::class, interpreter::function, interpreter::instance, interpreter::lox_type,
    interpreter::unwind, reporter,
};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub trait Callable: Debug {
    fn call(
        &self,
        reporter: &dyn reporter::Reporter,
        depths: &HashMap<usize, usize>,
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
    Boolean(bool),
    Class {
        class: class::Class,
    },
    Function {
        function: function::Function,
    },
    Instance {
        instance: instance::Instance,
    },
    Number(f64),
    NativeFunction {
        name: String,
        callable: Rc<Box<dyn NativeCallable>>,
    },
    Nil,
    String(String),
}

impl PartialEq for LoxType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            LoxType::Boolean(value) => {
                if let LoxType::Boolean(other) = other {
                    other == value
                } else {
                    false
                }
            }
            LoxType::Class { class: value, .. } => {
                if let LoxType::Class { class: other, .. } = other {
                    other == value
                } else {
                    false
                }
            }
            LoxType::Function {
                function: value, ..
            } => {
                if let LoxType::Function {
                    function: other, ..
                } = other
                {
                    other == value
                } else {
                    false
                }
            }
            LoxType::Instance {
                instance: value, ..
            } => {
                if let LoxType::Instance {
                    instance: other, ..
                } = other
                {
                    other == value
                } else {
                    false
                }
            }
            LoxType::NativeFunction { name: value, .. } => {
                if let LoxType::NativeFunction { name: other, .. } = other {
                    other == value
                } else {
                    false
                }
            }
            LoxType::Nil => matches!(other, LoxType::Nil),
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
        }
    }
}

impl Display for LoxType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxType::Boolean(bool) => write!(f, "{bool}"),
            LoxType::Class { class, .. } => write!(f, "\"class {}\"", class.name()),
            LoxType::Function { function, .. } => write!(f, "\"fun {}\"", function.name()),
            LoxType::Instance { instance, .. } => {
                write!(f, "\"instance of {}\"", instance.class_name())
            }
            LoxType::NativeFunction { name, .. } => write!(f, "\"native fun {name}\""),
            LoxType::Nil => write!(f, "nil"),
            LoxType::Number(number) => write!(f, "{number}"),
            LoxType::String(string) => write!(f, "\"{string}\""),
        }
    }
}

impl LoxType {
    pub fn find_instance_method(instance: &LoxType, name: &str) -> Option<LoxType> {
        if let LoxType::Instance { instance } = instance {
            instance.find_method(name)
        } else {
            None
        }
    }

    pub fn get_instance_value(
        instance: &LoxType,
        name: &str,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        if let LoxType::Instance { instance } = instance {
            instance.get(name)
        } else {
            Err(unwind::Unwind::WithError(
                "Only instances have fields".to_string(),
            ))
        }
    }

    pub fn set_instance_value(
        instance: &LoxType,
        name: &str,
        value: LoxType,
    ) -> Result<(), unwind::Unwind> {
        if let LoxType::Instance { instance } = instance {
            instance.set(name, value);
            Ok(())
        } else {
            Err(unwind::Unwind::WithError(
                "Only instances have fields".to_string(),
            ))
        }
    }
}
