use std::hash::Hash;

use crate::utils::id_factory::Id;

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

#[derive(Debug, Clone)]
pub enum Expr {
    Binary {
        uid: Id,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        uid: Id,
        expression: Box<Expr>,
    },
    Literal {
        uid: Id,
        value: Literal,
    },
    Unary {
        uid: Id,
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        uid: Id,
        name: Token,
    },
    Assign {
        uid: Id,
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        uid: Id,
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        uid: Id,
        callee: Box<Expr>,
        paren: Token,
        arguments: Box<Vec<Expr>>,
    },
}

impl Expr {
    fn get_uid(&self) -> Id {
        match self {
            Expr::Binary { uid, .. } => *uid,
            Expr::Grouping { uid, .. } => *uid,
            Expr::Literal { uid, .. } => *uid,
            Expr::Unary { uid, .. } => *uid,
            Expr::Variable { uid, .. } => *uid,
            Expr::Assign { uid, .. } => *uid,
            Expr::Logical { uid, .. } => *uid,
            Expr::Call { uid, .. } => *uid,
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.get_uid() == other.get_uid()
    }
}

impl Eq for Expr {}

impl Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_uid().hash(state);
    }
}
