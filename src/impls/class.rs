use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
    interpreter::Interpreter,
    syntax::{token::Token, value::Value},
    Exception, RuntimeError,
};

use super::{callable::Callable, function::Function};

type Result<T> = std::result::Result<T, Exception>;

#[derive(Debug, Clone)]
pub struct Class {
    name: String,
    methods: HashMap<String, Function>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Function>) -> Class {
        Class { name, methods }
    }

    pub fn find_method(&self, name: &String) -> Option<Value> {
        self.methods.get(name).map(|f| Value::Function(f.clone()))
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        Ok(Value::ClassInstance(Rc::new(RefCell::new(ClassInstance {
            class: self.clone(),
            fields: HashMap::new(),
        }))))
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

// ======================
// Design note:
// ======================
// Why this?
//
// A `ClassInstance` type doesn't work, because when setting properties
// We need a mutable ref, we still can menage to mutate, but
// When I tried do the getter, it somehow, returned an immutable state...
// Without the changes I've made.
pub type ClassInstanceRef = Rc<RefCell<ClassInstance>>;

#[derive(Debug, Clone)]
pub struct ClassInstance {
    class: Class,
    fields: HashMap<String, Value>,
}

impl ClassInstance {
    pub fn get(&self, name: &Token) -> Result<Value> {
        match self.fields.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => {
                // Looking for a field implicitly implies that fields shadow methods
                if let Some(method) = self.class.find_method(&name.lexeme) {
                    return Ok(method);
                }

                Err(Exception::RuntimeError(RuntimeError {
                    token: name.clone(),
                    message: format!("Undefined property '{}'.", name.lexeme),
                }))
            }
        }
    }

    pub fn set(&mut self, name: &Token, value: &Value) -> Result<()> {
        let key = name.lexeme.clone();
        self.fields.insert(key, value.clone());
        Ok(())
    }
}

impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
