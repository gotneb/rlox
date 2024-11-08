use crate::impls::{
    class::Class,
    function::{Function, NativeFunction},
};

/// Represents all possibles values in the language
#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Class(Class),
    Number(f64),
    String(String),
    Function(Function),
    NativeFunction(NativeFunction),
    Nil,
}
