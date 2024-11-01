use super::callable::Callable;

#[derive(Debug, Clone)]
pub struct Function;

impl Callable for Function {
    fn arity(&self) -> usize {
        todo!()
    }

    fn call(
        &self,
        interpreter: &mut crate::interpreter::Interpreter,
        arguments: Vec<crate::syntax::value::Value>,
    ) -> Result<crate::syntax::value::Value, crate::Exception> {
        todo!()
    }
}
