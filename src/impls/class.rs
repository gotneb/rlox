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
    getters: HashMap<String, Function>,
    name: String,
    methods: HashMap<String, Function>,
    static_methods: HashMap<String, Function>,
    super_class: Option<Box<Class>>,
}

impl Class {
    pub fn new(
        getters: HashMap<String, Function>,
        name: String,
        methods: HashMap<String, Function>,
        static_methods: HashMap<String, Function>,
        super_class: Option<Box<Class>>,
    ) -> Class {
        Class {
            getters,
            name,
            methods,
            static_methods,
            super_class,
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        match self.static_methods.get(&name.lexeme) {
            Some(method) => Ok(Value::Function(method.clone())),
            None => Exception::runtime_error(
                name.clone(),
                format!(
                    "Class doesn't have a static method called \"{}\".",
                    name.lexeme
                ),
            ),
        }
    }

    pub fn find_getter(&self, name: &Token) -> Option<Value> {
        self.getters
            .get(&name.lexeme)
            .map(|f| Value::Function(f.clone()))
    }

    pub fn find_method(&self, name: &String) -> Option<Value> {
        self.methods
            .get(name)
            .map(|f| Value::Function(f.clone()))
            .or(self
                .super_class
                .as_ref()
                .and_then(|super_class| super_class.find_method(name)))
    }
}

impl Callable for Class {
    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method(&"init".into()) {
            match initializer {
                Value::Function(initializer) => return initializer.arity(),
                _ => panic!("initializer is not a function!"),
            }
        }

        0
    }

    fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value> {
        let instance = ClassInstance::new(self.clone());

        if let Some(method) = self.find_method(&"init".into()) {
            if let Value::Function(initializer) = method {
                initializer.bind(instance.clone()).call(interpreter, args)?;
            }
        }

        Ok(Value::ClassInstance(instance))
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
    pub class: Class,
    fields: HashMap<String, Value>,
}

impl ClassInstance {
    pub fn new(class: Class) -> ClassInstanceRef {
        let instance = Self {
            class,
            fields: HashMap::new(),
        };

        Rc::new(RefCell::new(instance))
    }

    pub fn get(
        &self,
        name: &Token,
        instance_ref: ClassInstanceRef,
        interpreter: &mut Interpreter,
    ) -> Result<Value> {
        match self.fields.get(&name.lexeme) {
            // Field
            Some(value) => Ok(value.clone()),
            None => {
                // Getter
                // Looking for a field implicitly implies that fields shadow getters
                if let Some(Value::Function(getter)) = self.class.find_getter(name) {
                    let bound_getter = getter.bind(instance_ref);
                    let value = bound_getter.call(interpreter, vec![])?;
                    return Ok(value);
                }

                // Method
                // Looking for a getters implicitly implies that getters shadow methods
                if let Some(Value::Function(method)) = self.class.find_method(&name.lexeme) {
                    let bound_method = method.bind(instance_ref);
                    return Ok(Value::Function(bound_method));
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
