use crate::interpreter::lox_type::Callable;
use crate::interpreter::{instance, lox_type, unwind};
use crate::reporter;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
struct InternalClass {
    name: String,
    methods: HashMap<String, lox_type::LoxType>,
}

impl InternalClass {
    fn new(name: &str, methods: HashMap<String, lox_type::LoxType>) -> Self {
        Self {
            name: name.to_string(),
            methods,
        }
    }

    fn arity(&self) -> usize {
        let initializer = self.find_method("init");
        if let Some(lox_type::LoxType::Function { function }) = initializer {
            function.arity()
        } else {
            0
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn find_method(&self, name: &str) -> Option<lox_type::LoxType> {
        self.methods.get(name).cloned()
    }
}

impl PartialEq for InternalClass {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, Debug)]
pub struct Class {
    class: Rc<InternalClass>,
}

impl Class {
    pub fn new(name: &str, methods: HashMap<String, lox_type::LoxType>) -> Self {
        Self {
            class: Rc::new(InternalClass::new(name, methods)),
        }
    }

    pub fn name(&self) -> &str {
        self.class.name()
    }

    pub fn find_method(&self, name: &str) -> Option<lox_type::LoxType> {
        self.class.find_method(name)
    }
}

impl lox_type::Callable for Class {
    fn arity(&self) -> usize {
        self.class.arity()
    }

    fn call(
        &self,
        reporter: &dyn reporter::Reporter,
        depths: &HashMap<usize, usize>,
        arguments: Vec<lox_type::LoxType>,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let instance = instance::Instance::new(self.clone());
        let instance = lox_type::LoxType::Instance { instance };
        let initializer = lox_type::LoxType::find_instance_method(&instance, "init");
        if let Some(lox_type::LoxType::Function { function }) = initializer {
            if let Err(unwind::Unwind::WithError(message)) =
                function.call(reporter, depths, arguments)
            {
                return Err(unwind::Unwind::WithError(message));
            }
        }
        Ok(instance)
    }
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.class == other.class
    }
}
