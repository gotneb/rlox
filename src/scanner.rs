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
            _ => {
                error(self.line, "Unexpected character.");
            },
        }
    }

    fn add_token(&mut self, token_type: TokenType, literal: Literal) {
        let text = self
                            .source[self.start..self.current]
                            .to_string();
        self.tokens
            .push(Token::new(token_type, text, literal, self.line));
    }

    fn is_at_end(&mut self) -> bool {
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
}
