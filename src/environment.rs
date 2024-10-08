use std::collections::HashMap;

use crate::{
    syntax::{token::Token, value::Value},
    Exception,
};

type Result<T> = std::result::Result<T, Exception>;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => Exception::runtime_error(
                name.clone(),
                format!("Undefined variable '{}'.", name.lexeme),
            ),
        }
    }
}
