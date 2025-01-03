use crate::{
    print_error,
    syntax::{
        expr::Expr,
        stmt::Stmt,
        token::{Literal, Token},
        token_type::TokenType,
    },
    utils::id_factory::new_uid,
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
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Option<Stmt> {
        let result = if self.match_token(&[TokenType::Var]) {
            self.var_declaration()
        } else if self.match_token(&[TokenType::Fun]) {
            self.function("function".into())
        } else if self.match_token(&[TokenType::Class]) {
            self.class_declaration()
        } else {
            self.statement()
        };

        match result {
            Ok(stmt) => Some(stmt),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn class_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected a class name.")?;

        let mut super_class = None;
        if self.match_token(&[TokenType::Less]) {
            let name = self.consume(TokenType::Identifier, "Expected super class name.")?;
            super_class = Some(Expr::Variable { uid: new_uid(), name });
        }

        self.consume(TokenType::LeftBrace, "Expected '{' before class body.")?;

        let mut getters = vec![];
        let mut methods = vec![];
        let mut static_methods = vec![];

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if self.match_token(&[TokenType::Class]) {
                // Static methods
                static_methods.push(self.function("static method".into())?);
            } else if !self.is_at_end() && self.peek_next().token_type == TokenType::LeftBrace {
                getters.push(self.getter()?);
            } else {
                // Instance methods
                methods.push(self.function("method".into())?);
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after class body")?;

        Ok(Stmt::Class {
            getters,
            name,
            methods,
            static_methods,
            super_class,
        })
    }

    fn getter(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected getter name.")?;
        self.consume(TokenType::LeftBrace, "Expected '{' after getter name.")?;

        let body = self.block()?;

        // Getters behave like a function
        Ok(Stmt::Function {
            name,
            body,
            parameters: vec![],
        })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_token(&[TokenType::While]) {
            return self.while_stmt();
        }
        if self.match_token(&[TokenType::Return]) {
            return self.return_stmt();
        }
        if self.match_token(&[TokenType::For]) {
            return self.for_stmt();
        }
        if self.match_token(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block {
                statements: self.block().unwrap_or(vec![]),
            });
        }

        self.expression_stmt()
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after if keyword.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition.")?;

        let then_branch = self.statement()?;
        let mut else_branch = None;
        if self.match_token(&[TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn return_stmt(&mut self) -> Result<Stmt> {
        let keyword = self.previous();
        let mut value = None;
        if !self.check(&TokenType::Semicolon) {
            value = Some(self.expression()?);
        };
        self.consume(TokenType::Semicolon, "Expected ';' after return vale.")?;

        Ok(Stmt::Return { keyword, value })
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected a variable name.")?;

        let mut initializer = None;
        if self.match_token(&[TokenType::Equal]) {
            initializer = Some(self.expression()?);
        }

        self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn while_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition.")?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn for_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftParen, "Expected '(' after for statement.")?;

        let initializer;
        if self.match_token(&[TokenType::Semicolon]) {
            initializer = None;
        } else if self.match_token(&[TokenType::Var]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_stmt()?);
        }

        let mut condition = None;
        if !self.check(&TokenType::Semicolon) {
            condition = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon, "Expected ';' after loop condition.")?;

        let mut increment = None;
        if !self.check(&TokenType::RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(TokenType::RightParen, "Expected ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![body, Stmt::Expression(increment)],
            }
        }

        if let None = condition {
            condition = Some(Expr::Literal {
                uid: new_uid(),
                value: Literal::Bool(true),
            })
        }
        body = Stmt::While {
            condition: condition.unwrap(),
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block {
                statements: vec![initializer, body],
            };
        }

        Ok(body)
    }

    fn expression_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value.")?;
        Ok(Stmt::Expression(expr))
    }

    fn function(&mut self, kind: String) -> Result<Stmt> {
        let name = self.consume(
            TokenType::Identifier,
            format!("Expected {} name.", kind).as_str(),
        )?;

        self.consume(
            TokenType::LeftParen,
            format!("Expected '(' after {} name.", kind).as_str(),
        )?;
        let mut parameters = vec![];

        if !self.check(&TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    self.error(self.peek().clone(), "Can't have more than 255 parameters");
                }
                parameters.push(self.consume(TokenType::Identifier, "Expected a parameter name.")?);

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RightParen,
            format!("Expected ')' after {} params list.", kind).as_str(),
        )?;

        self.consume(
            TokenType::LeftBrace,
            format!("Expected '{{' before {} body.", kind).as_str(),
        )?;

        let body = self.block()?;

        Ok(Stmt::Function {
            name,
            parameters,
            body,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = vec![];

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable { name, .. } = expr {
                return Ok(Expr::Assign {
                    uid: new_uid(),
                    name,
                    value: Box::new(value),
                });
            } else if let Expr::Get { name, object, .. } = expr {
                return Ok(Expr::Set {
                    uid: new_uid(),
                    name,
                    object,
                    value: Box::new(value),
                });
            }

            return Err(self.error(equals, "Invalid assignment target."));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.match_token(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical {
                uid: new_uid(),
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
            return Ok(expr);
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logical {
                uid: new_uid(),
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
            return Ok(expr);
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;
        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                uid: new_uid(),
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term();
        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term();
            expr = Ok(Expr::Binary {
                uid: new_uid(),
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }
        expr
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor();
        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.factor();
            expr = Ok(Expr::Binary {
                uid: new_uid(),
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }
        expr
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary();
        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary();
            expr = Ok(Expr::Binary {
                uid: new_uid(),
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            });
        }
        expr
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                uid: new_uid(),
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut args = vec![];

        if !self.check(&TokenType::RightParen) {
            loop {
                if args.len() >= 255 {
                    self.error(self.peek().clone(), "Can't have more than 255 arguments.");
                }

                args.push(self.expression()?);

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TokenType::RightParen, "Expected ')' after arguments.")?;

        Ok(Expr::Call {
            uid: new_uid(),
            callee: Box::new(callee),
            paren,
            arguments: Box::new(args),
        })
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenType::Dot]) {
                let name =
                    self.consume(TokenType::Identifier, "Expected property name after '.'.")?;
                expr = Expr::Get {
                    uid: new_uid(),
                    name,
                    object: Box::new(expr),
                };
            } else {
                break;
            }
        }

        // println!("{:#?}", expr);
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal {
                uid: new_uid(),
                value: Literal::Bool(false),
            });
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal {
                uid: new_uid(),
                value: Literal::Bool(true),
            });
        }
        if self.match_token(&[TokenType::Nil]) {
            return Ok(Expr::Literal {
                uid: new_uid(),
                value: Literal::None,
            });
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(Expr::Literal {
                uid: new_uid(),
                value: self.previous().literal,
            });
        }

        if self.match_token(&[TokenType::This]) {
            return Ok(Expr::This {
                uid: new_uid(),
                name: self.previous(),
            });
        }

        if self.match_token(&[TokenType::Super]) {
            let keyword = self.previous();
            self.consume(TokenType::Dot, "Expected '.' after 'super'.")?;
            let method = self.consume(TokenType::Identifier, "Expect superclass method name.")?;
            return Ok(Expr::Super { uid: new_uid(), keyword, method });
        }

        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expr::Variable {
                uid: new_uid(),
                name: self.previous(),
            });
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
            return Ok(Expr::Grouping {
                uid: new_uid(),
                expression: Box::new(expr?),
            });
        }

        Err(self.error(self.peek(), "Expected expression"))
    }

    // REFACTOR: Prefer using ´slice´ over ´Vec´. It less verbose...
    fn match_token(&mut self, tokens_types: &[TokenType]) -> bool {
        for token in tokens_types.iter() {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }
        Err(self.error(self.peek(), msg))
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == *token_type
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

    fn peek_next(&self) -> Token {
        let next = self.current + 1;
        self.tokens.get(next).unwrap().clone()
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
