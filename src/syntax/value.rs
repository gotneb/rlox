use crate::impls::function::Function;

/// Represents all possibles values in the language
#[derive(Debug, Clone)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    String(String),
    Function(Function),
    Nil,
} 