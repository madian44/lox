use crate::interpreter::lox_type;
#[derive(Debug, PartialEq)]
pub enum Unwind {
    WithResult(lox_type::LoxType),
    WithError(String),
}
