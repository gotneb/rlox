use std::fmt::Display;

use crate::{interpreter::Interpreter, syntax::value::Value, Exception};

use super::callable::Callable;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
}

impl Callable for Class {
    fn arity(&self) -> usize {
        0
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, Exception> {
        Ok(Value::ClassInstance(ClassInstance { class: self.clone() }))
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
} 


impl Display for ClassInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}