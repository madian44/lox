use crate::interpreter::lox_type::LoxType;
use crate::interpreter::{class, lox_type, unwind};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static ID_SRC: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
struct InternalInstance {
    id: usize,
    class: class::Class,
    fields: HashMap<String, lox_type::LoxType>,
}

impl InternalInstance {
    fn get_id() -> usize {
        ID_SRC.fetch_add(1, Ordering::Relaxed)
    }

    pub fn new(class: class::Class) -> Self {
        Self {
            id: InternalInstance::get_id(),
            class,
            fields: HashMap::new(),
        }
    }

    pub fn class_name(&self) -> &str {
        self.class.name()
    }

    pub fn find_method(&self, name: &str) -> Option<lox_type::LoxType> {
        self.class.find_method(name)
    }

    pub fn get(&self, name: &str) -> Result<lox_type::LoxType, unwind::Unwind> {
        if self.fields.contains_key(name) {
            Ok(self.fields.get(name).unwrap().clone())
        } else if let Some(method) = self.find_method(name) {
            Ok(method)
        } else {
            Err(unwind::Unwind::WithError(format!(
                "Undefined property '{}'",
                name
            )))
        }
    }

    pub fn set(&mut self, name: &str, value: lox_type::LoxType) {
        self.fields.insert(name.to_string(), value);
    }
}

impl PartialEq for InternalInstance {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug)]
pub struct Instance {
    instance: Rc<RefCell<InternalInstance>>,
}

impl Instance {
    pub fn new(class: class::Class) -> Self {
        Self {
            instance: Rc::new(RefCell::new(InternalInstance::new(class))),
        }
    }

    pub fn class_name(&self) -> String {
        self.instance.borrow().class_name().to_string()
    }

    fn bind_this(&self, value: LoxType) -> LoxType {
        if let LoxType::Function { function } = value {
            LoxType::Function {
                function: function.bind_this(LoxType::Instance {
                    instance: self.clone(),
                }),
            }
        } else {
            value
        }
    }

    pub fn get(&self, name: &str) -> Result<lox_type::LoxType, unwind::Unwind> {
        let value = self.instance.borrow().get(name)?;
        let value = self.bind_this(value);
        Ok(value)
    }

    pub fn find_method(&self, name: &str) -> Option<LoxType> {
        self.instance
            .borrow()
            .find_method(name)
            .map(|f| self.bind_this(f))
    }

    pub fn set(&self, name: &str, value: LoxType) {
        self.instance.borrow_mut().set(name, value);
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.instance == other.instance
    }
}
