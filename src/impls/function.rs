use crate::{
    environment::{EnvRef, Environment},
    interpreter::Interpreter,
    syntax::{stmt::Stmt, value::Value},
    Exception,
};

use super::{callable::Callable, class::ClassInstanceRef};

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub arity: usize,
    pub callable: fn(&mut Interpreter, Vec<Value>) -> Value,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub declaration: Stmt,
    pub closure: EnvRef,
    is_initializer: bool,
}

impl Function {
    pub fn new(declaration: Stmt, closure: EnvRef, is_initializer: bool) -> Function {
        Function {
            declaration,
            closure,
            is_initializer,
        }
    }

    pub fn bind(&self, instance: ClassInstanceRef) -> Function {
        let env = Environment::new_local(&self.closure);
        env.borrow_mut()
            .define("this".into(), Value::ClassInstance(instance));

        Function::new(self.declaration.clone(), env, self.is_initializer)
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, Exception> {
        Ok((self.callable)(interpreter, arguments))
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        if let Stmt::Function { parameters, .. } = &self.declaration {
            return parameters.len();
        }
        panic!("Function was not initalized with a function declaration!");
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, Exception> {
        let env = Environment::new_local(&self.closure);

        if let Stmt::Function {
            parameters, body, ..
        } = &self.declaration
        {
            for (i, value) in arguments.iter().enumerate() {
                env.borrow_mut()
                    .define(parameters.get(i).unwrap().lexeme.clone(), value.clone());
            }

            if let Err(e) = interpreter.execute_block(body, env) {
                return match e {
                    Exception::RuntimeError(e) => Err(Exception::RuntimeError(e)),
                    Exception::Return(value) => {
                        if self.is_initializer {
                            return self.closure.borrow().get_at(0, &"this".into());
                        }

                        Ok(value)
                    }
                };
            }
        }

        if self.is_initializer {
            return self.closure.borrow().get_at(0, &"this".into());
        }

        Ok(Value::Nil)
    }
}
