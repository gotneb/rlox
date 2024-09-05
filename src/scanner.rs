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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punctuators() {
        let mut scanner = Scanner::new("(){};,+-*!===<=>=!=<>/.".into());
        let tokens = scanner.scan_tokens();

        let expected = vec![
            Token::new(TokenType::LeftParen, "(".into(), Literal::None, 1),
            Token::new(TokenType::RightParen, ")".into(), Literal::None, 1),
            Token::new(TokenType::LeftBrace, "{".into(), Literal::None, 1),
            Token::new(TokenType::RightBrace, "}".into(), Literal::None, 1),
            Token::new(TokenType::Semicolon, ";".into(), Literal::None, 1),
            Token::new(TokenType::Comma, ",".into(), Literal::None, 1),
            Token::new(TokenType::Plus, "+".into(), Literal::None, 1),
            Token::new(TokenType::Minus, "-".into(), Literal::None, 1),
            Token::new(TokenType::Star, "*".into(), Literal::None, 1),
            Token::new(TokenType::BangEqual, "!=".into(), Literal::None, 1),
            Token::new(TokenType::EqualEqual, "==".into(), Literal::None, 1),
            Token::new(TokenType::LessEqual, "<=".into(), Literal::None, 1),
            Token::new(TokenType::GreaterEqual, ">=".into(), Literal::None, 1),
            Token::new(TokenType::BangEqual, "!=".into(), Literal::None, 1),
            Token::new(TokenType::Less, "<".into(), Literal::None, 1),
            Token::new(TokenType::Greater, ">".into(), Literal::None, 1),
            Token::new(TokenType::Slash, "/".into(), Literal::None, 1),
            Token::new(TokenType::Dot, ".".into(), Literal::None, 1),
            Token::new(TokenType::Eof, "".into(), Literal::None, 1),
        ];

        assert_eq!(tokens.len(), expected.len());
        for (index, token) in expected.iter().enumerate() {
            assert_eq!(*token, expected[index]);
        }
    }

    #[test]
    fn numbers() {
        let mut scanner = Scanner::new("3.14159\n299792458\n2.71828\n123.\n.123".into());
        let tokens = scanner.scan_tokens();

        let expected = vec![
            Token::new(TokenType::Number, "3.14159".into(), Literal::Number(3.14159), 1),
            Token::new(TokenType::Number, "299792458".into(), Literal::Number(299792458.), 2),
            Token::new(TokenType::Number, "2.71828".into(), Literal::Number(2.71828), 3),
            Token::new(TokenType::Number, "123".into(), Literal::Number(123.0), 4),
            Token::new(TokenType::Dot, ".".into(), Literal::None, 4),
            Token::new(TokenType::Dot, ".".into(), Literal::None, 5),
            Token::new(TokenType::Dot, "123".into(), Literal::None, 5),
            Token::new(TokenType::Eof, "".into(), Literal::None, 6),
        ];

        assert_eq!(tokens.len(), expected.len());
        for (index, token) in expected.iter().enumerate() {
            assert_eq!(*token, expected[index]);
        }
    }

    #[test]
    fn keywords() {
        let mut scanner = Scanner::new("and class else false for if nil or print return super this true var while".into());

        let tokens = scanner.scan_tokens();

        let expected_tokens = vec![
            Token::new(TokenType::And, "and".into(), Literal::None, 1),
            Token::new(TokenType::Class, "class".into(), Literal::None, 1),
            Token::new(TokenType::Else, "else".into(), Literal::None, 1),
            Token::new(TokenType::False, "false".into(), Literal::None, 1),
            Token::new(TokenType::For, "for".into(), Literal::None, 1),
            Token::new(TokenType::If, "if".into(), Literal::None, 1),
            Token::new(TokenType::Nil, "nil".into(), Literal::None, 1),
            Token::new(TokenType::Or, "or".into(), Literal::None, 1),
            Token::new(TokenType::Print, "print".into(), Literal::None, 1),
            Token::new(TokenType::Return, "return".into(), Literal::None, 1),
            Token::new(TokenType::Super, "super".into(), Literal::None, 1),
            Token::new(TokenType::This, "this".into(), Literal::None, 1),
            Token::new(TokenType::True, "true".into(), Literal::None, 1),
            Token::new(TokenType::Var, "var".into(), Literal::None, 1),
            Token::new(TokenType::While, "while".into(), Literal::None, 1),
            Token::new(TokenType::Eof, "".into(), Literal::None, 1),
        ];

        assert_eq!(tokens.len(), expected_tokens.len());
        for (index, token) in tokens.iter().enumerate() {
            assert_eq!(*token, expected_tokens[index]);
        }
    }

    fn whistespaces() {
        let mut scanner = Scanner::new("var
        // Yes, this variable is longer on purpose :)
        data_do_ano_do_descorimento_do_brasil             =
        1500
        ;
        ".into());

        let tokens = scanner.scan_tokens();

        let expected = vec![
            Token::new(TokenType::Var, "var".into(), Literal::None, 1),
            Token::new(TokenType::Slash, "//".into(), Literal::None, 2),
            Token::new(TokenType::Identifier, "data_do_ano_do_descorimento_do_brasil".into(), Literal::None, 3),
            Token::new(TokenType::Equal, "=".into(), Literal::None, 3),
            Token::new(TokenType::Number, "1500".into(), Literal::Number(1500.0), 4),
            Token::new(TokenType::Semicolon, ";".into(), Literal::None, 5),
            Token::new(TokenType::Eof, "".into(), Literal::None, 1),
        ];
    }
}
