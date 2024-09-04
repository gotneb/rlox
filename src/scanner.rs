use std::collections::HashMap;

use crate::{
    error, token::{Literal, Token}, token_type::TokenType
};

#[derive(Debug)]
pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            Literal::None,
            self.line,
        ));

        self.tokens.clone()
    }

    fn scan_token(&mut self) {
        let c = self.advance();

        match c {
            // Those are simple, they always come alone :)
            '(' => self.add_token(TokenType::LeftParen, Literal::None),
            ')' => self.add_token(TokenType::RightParen, Literal::None),
            '{' => self.add_token(TokenType::LeftBrace, Literal::None),
            '}' => self.add_token(TokenType::RightBrace, Literal::None),
            ',' => self.add_token(TokenType::Comma, Literal::None),
            '.' => self.add_token(TokenType::Dot, Literal::None),
            '-' => self.add_token(TokenType::Minus, Literal::None),
            '+' => self.add_token(TokenType::Plus, Literal::None),
            ';' => self.add_token(TokenType::Semicolon, Literal::None),
            '*' => self.add_token(TokenType::Star, Literal::None),

            // Those are not, they might come with some lexeme else...
            '!' => {
                let is_matched = self.match_next_token('=');
                self.add_token(if is_matched  { TokenType::BangEqual } else { TokenType::Bang }, Literal::None)
            },
            '=' => {
                let is_matched = self.match_next_token('=');
                self.add_token(if is_matched { TokenType::EqualEqual } else { TokenType::Equal }, Literal::None)
            },
            '<' => {
                let is_matched = self.match_next_token('=');
                self.add_token(if is_matched { TokenType::LessEqual } else { TokenType::Less }, Literal::None)
            },
            '>' => {
                let is_matched = self.match_next_token('=');
                self.add_token(if is_matched { TokenType::GreaterEqual } else { TokenType::Greater }, Literal::None)
            },
            // Special case
            '/' => {
                if self.match_next_token('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash, Literal::None)
                }
            }
            // Meaningless lexemes... skip
            ' ' | '\r' | '\t' => {},
            '\n' => self.line += 1,
            _ => {
                // Detecting numbers is a litle more complex, we can check them
                // in the not matched branch, because in all above cases is more easy to
                // verify other cases instead of numbers
                if self.is_digit(c) {
                    self.number()
                } else if self.is_alpha(c) {
                    self.identifier()
                } else {
                    error(self.line, "Unexpected character.");
                }
            },
        }
    }

    fn identifier(&mut self) {
        while self.is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.source[self.start..self.current].to_string();
        match self.get_keywords().get(&text) {
            Some(token_type) => self.add_token(token_type.clone(), Literal::None),
            None => self.add_token(TokenType::Identifier, Literal::None),
        }
    }

    fn number(&mut self) {
        while self.is_digit(self.peek()) {
            self.advance();
        }

        // Look for fractional part
        if self.peek() == '.' && self.is_digit(self.peek_next()) {
            self.advance();

            while self.is_digit(self.peek()) {
                self.advance();
            }
        }

        let value: f64 = self.source
                            .get(self.start..self.current)
                            .unwrap()
                            .parse()
                            .unwrap();
        self.add_token(TokenType::Number, Literal::Number(value))
    }

    fn string(&mut self) {
        if self.peek() != '"' && !self.is_at_end() {
            // Lox has support for multi-string
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return error(self.line, "Unterminated string");
        }

        // The closing ".
        self.advance();

        // Trim surrounding quotes
        let value = self.source[self.start+1..self.current-1]
                            .to_string();
        self.add_token(TokenType::String, Literal::String(value));

    }

    fn add_token(&mut self, token_type: TokenType, literal: Literal) {
        let text = self
                            .source[self.start..self.current]
                            .to_string();
        self.tokens
            .push(Token::new(token_type, text, literal, self.line));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    // Takes the current character and returns it. Then increment.
    fn advance(&mut self) -> char {
        let char = self.source.as_bytes()[self.current] as char;
        self.current += 1;
        char
    }

    fn match_next_token(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] as char != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source.as_bytes()[self.current] as char
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0'
        }
        self.source.as_bytes()[self.current + 1] as char
    }

    fn is_alpha(&self, c: char) -> bool {
        return (c >= 'a' && c <= 'z') ||
               (c >= 'A' && c <= 'Z') || 
               c == '_';
    }

    // TODO: Rust' std has a lib for this, i think...
    fn is_alpha_numeric(&self, c: char) -> bool {
        return self.is_alpha(c) || self.is_digit(c)
    }

    // TODO: Rust' std has a lib for this, i think...
    fn is_digit(&self, c: char) -> bool {
        return c >= '0' && c <= '9'
    }

    fn get_keywords(&self) -> HashMap<String, TokenType> {
        let mut hash = HashMap::new();
        
        hash.insert("and".into(), TokenType::And);
        hash.insert("class".into(), TokenType::Class);
        hash.insert("else".into(), TokenType::Else);
        hash.insert("false".into(), TokenType::False);
        hash.insert("for".into(), TokenType::For);
        hash.insert("fun".into(), TokenType::Fun);
        hash.insert("if".into(), TokenType::If);
        hash.insert("nil".into(), TokenType::Nil);
        hash.insert("or".into(), TokenType::Or);
        hash.insert("print".into(), TokenType::Print);
        hash.insert("return".into(), TokenType::Return);
        hash.insert("super".into(), TokenType::Super);
        hash.insert("this".into(), TokenType::This);
        hash.insert("true".into(), TokenType::True);
        hash.insert("var".into(), TokenType::Var);
        hash.insert("while".into(), TokenType::While);

        hash
    }
}
