use crate::{
    print_error,
    syntax::{
        expr::Expr,
        stmt::Stmt,
        token::{Literal, Token},
        token_type::TokenType,
    },
};

#[derive(Debug)]
pub struct ParserError;

type Result<T> = std::result::Result<T, ParserError>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = vec![];

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_token(vec![TokenType::Print]) {
            return self.print_stmt();
        }
        self.expression_stmt()
    }

    fn print_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(expr))
    }

    fn expression_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison();
        while self.match_token(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison();
            expr = Ok(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            })
        }
        expr
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term();
        while self.match_token(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term();
            expr = Ok(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }
        expr
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor();
        while self.match_token(vec![TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.factor();
            expr = Ok(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }
        expr
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary();
        while self.match_token(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary();
            expr = Ok(Expr::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }
        expr
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_token(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.match_token(vec![TokenType::False]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(false),
            });
        }
        if self.match_token(vec![TokenType::True]) {
            return Ok(Expr::Literal {
                value: Literal::Bool(true),
            });
        }
        if self.match_token(vec![TokenType::Nil]) {
            return Ok(Expr::Literal {
                value: Literal::None,
            });
        }

        if self.match_token(vec![TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal {
                value: self.previous().literal,
            });
        }

        if self.match_token(vec![TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr?),
            });
        }

        Err(self.error(self.peek(), "Expected expression"))
    }

    // REFACTOR: Prefer using ´slice´ over ´Vec´. It less verbose...
    fn match_token(&mut self, tokens: Vec<TokenType>) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token> {
        if self.check(token_type) {
            return Ok(self.advance());
        }
        Err(self.error(self.peek(), msg))
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        return self.previous();
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.current).unwrap().clone()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn error(&self, token: Token, msg: &str) -> ParserError {
        print_error(&token, msg);
        ParserError {}
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}
