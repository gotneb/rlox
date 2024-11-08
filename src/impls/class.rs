use std::{collections::HashMap, fmt::Display};

use crate::{
    interpreter::Interpreter,
    syntax::{token::Token, value::Value},
    Exception, RuntimeError,
};

use super::callable::Callable;

type Result<T> = std::result::Result<T, Exception>;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
}

impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        Ok(Value::ClassInstance(ClassInstance {
            class: self.clone(),
            fields: HashMap::new(),
        }))
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub struct ClassInstance {
    class: Class,
    fields: HashMap<String, Value>,
}

impl ClassInstance {
    pub fn get(&self, name: &Token) -> Result<Value> {
        match self.fields.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => Err(Exception::RuntimeError(RuntimeError {
                token: name.clone(),
                message: format!("Undefined property '{}'.", name.lexeme),
            })),
        }
    }
}

impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
