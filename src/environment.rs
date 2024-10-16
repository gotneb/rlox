use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    syntax::{token::Token, value::Value},
    Exception,
};

pub type EnvRef = Rc<RefCell<Environment>>;
type Result<T> = std::result::Result<T, Exception>;

pub struct Environment {
    pub enclosing: Option<EnvRef>,
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new_global() -> EnvRef {
        Rc::new(RefCell::new(Environment {
            enclosing: None,
            values: HashMap::new(),
        }))
    }

    pub fn new_local(enclosing: &EnvRef) -> EnvRef {
        Rc::new(RefCell::new(Environment {
            enclosing: Some(enclosing.clone()),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => {
                if let Some(enclosing) = &self.enclosing {
                    return enclosing.borrow().get(name);
                }

                return Exception::runtime_error(
                    name.clone(),
                    format!("Undefined variable '{}'.", name.lexeme),
                );
            }
        }
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<Value> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value.clone());
            return Ok(value);
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Exception::runtime_error(
            name.clone(),
            format!("Undefined variable \"{}\".", name.lexeme),
        )
    }
}
