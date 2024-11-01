use crate::{interpreter::Interpreter, syntax::value::Value, Exception};

use super::callable::Callable;

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub arity: usize,
    pub callable: fn(&mut Interpreter, Vec<Value>) -> Value,
}

#[derive(Debug, Clone)]
pub struct Function;

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
        todo!()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, Exception> {
        todo!()
    }
}
