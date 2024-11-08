use crate::impls::{
    class::{Class, ClassInstance},
    function::{Function, NativeFunction},
};

/// Represents all possibles values in the language
#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Class(Class),
    ClassInstance(ClassInstance),
    Number(f64),
    String(String),
    Function(Function),
    NativeFunction(NativeFunction),
    Nil,
}
