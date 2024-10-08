use super::token::{Literal, Token};

// Explanations
//
// 1. The use of Box
// Why use Box<T> here? Rust structs need to have a DEFINE SIZE in memory
// But if if we have done recursive struct its size wouldn't be know
// Since `Box<T>` has a well know size, Rust allows this. (See enum)
//
// 2. Why not use Structs?
// I've tried using structs and then putting then inside the enum...
// But `Literal` would be definied 2x and it'd be troublesome...
// So the Cleanest way was creating it inside the enum :)

pub trait Visitor<T> {
    fn visit_expr(&mut self, expression: &Expr) -> T;
}

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    },
}
