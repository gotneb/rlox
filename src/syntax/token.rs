use std::fmt::Display;

use super::token_type::TokenType;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Literal, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {}",
            self.token_type, self.lexeme, self.line
        )
    }
}