use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum LoxType {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl Display for LoxType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxType::Nil => write!(f, "nil"),
            LoxType::Boolean(bool) => write!(f, "{bool}"),
            LoxType::Number(number) => write!(f, "{number}"),
            LoxType::String(string) => write!(f, "{string}"),
        }
    }
}
