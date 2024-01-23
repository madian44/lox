use crate::interpreter::interpret_with_environment;
use crate::interpreter::lox_type::LoxType;
use crate::{interpreter::environment, interpreter::lox_type, interpreter::unwind, reporter, stmt};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::iter::zip;
use std::rc::Rc;

struct InternalFunction {
    closure: environment::Environment,
    pub function: stmt::function::Function,
    is_initialiser: bool,
}

impl InternalFunction {
    fn new(
        closure: &environment::Environment,
        function: stmt::function::Function,
        is_initialiser: bool,
    ) -> Self {
        Self {
            closure: closure.clone(),
            function,
            is_initialiser,
        }
    }

    fn bind_this(&self, this: lox_type::LoxType) -> Self {
        let mut closure = environment::Environment::new_with_enclosing(&self.closure);
        closure.define("this", this);

        Self {
            closure,
            function: self.function.clone(),
            is_initialiser: self.is_initialiser,
        }
    }

    fn return_this_or_unwind_with(
        &self,
        environment: &environment::Environment,
        default: LoxType,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        if self.is_initialiser {
            Err(unwind::Unwind::WithResult(
                environment.get_at(Some(1), "this").unwrap(),
            ))
        } else {
            Err(unwind::Unwind::WithResult(default))
        }
    }

    fn name(&self) -> &str {
        &self.function.name().lexeme
    }
}

impl Debug for InternalFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.function.name().lexeme)
    }
}

impl PartialEq for InternalFunction {
    fn eq(&self, other: &Self) -> bool {
        self.function.name() == other.function.name()
    }
}

impl lox_type::Callable for InternalFunction {
    fn arity(&self) -> usize {
        self.function.params().len()
    }

    fn call(
        &self,
        reporter: &dyn reporter::Reporter,
        depths: &HashMap<usize, usize>,
        arguments: Vec<lox_type::LoxType>,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let closure = self.closure.clone();
        let mut environment = environment::Environment::new_with_enclosing(&closure);

        for (k, v) in zip(self.function.params(), arguments) {
            environment.define(&k.lexeme, v);
        }

        let result =
            interpret_with_environment(reporter, depths, &mut environment, self.function.body());
        match result {
            Err(unwind::Unwind::WithError(message)) => Err(unwind::Unwind::WithError(message)),
            Err(unwind::Unwind::WithResult(value)) => {
                self.return_this_or_unwind_with(&environment, value)
            }
            _ => self.return_this_or_unwind_with(&environment, lox_type::LoxType::Nil),
        }
    }
}

#[derive(Clone)]
pub struct Function {
    function: Rc<InternalFunction>,
}

impl Function {
    pub fn new(
        closure: &environment::Environment,
        function: stmt::function::Function,
        is_initialiser: bool,
    ) -> Self {
        Self {
            function: Rc::new(InternalFunction::new(closure, function, is_initialiser)),
        }
    }

    pub fn bind_this(&self, this: lox_type::LoxType) -> Self {
        Self {
            function: Rc::new(self.function.bind_this(this)),
        }
    }

    pub fn name(&self) -> &str {
        self.function.name()
    }
}

impl lox_type::Callable for Function {
    fn arity(&self) -> usize {
        self.function.arity()
    }

    fn call(
        &self,
        reporter: &dyn reporter::Reporter,
        depths: &HashMap<usize, usize>,
        arguments: Vec<lox_type::LoxType>,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        self.function.call(reporter, depths, arguments)
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.function.name())
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.function.name())
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function
    }
}
