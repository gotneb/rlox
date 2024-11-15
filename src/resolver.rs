use std::collections::HashMap;

use crate::{
    interpreter::Interpreter,
    print_error,
    syntax::{
        expr::{self, Expr, Visitor},
        stmt::{self, Stmt},
        token::Token,
    },
    RuntimeError,
};

#[derive(Clone, Copy)]
enum FunctionType {
    Function,
    None,
    Method,
    Initializer,
}

#[derive(Clone, Copy)]
enum ClassType {
    None,
    Class,
}

struct State {
    pub is_ready: bool,
    pub is_used: bool,
    pub token: Token,
}

impl State {
    fn new(is_ready: bool, is_used: bool, token: Token) -> State {
        State {
            is_ready,
            is_used,
            token,
        }
    }
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, State>>,
    current_function: FunctionType,
    current_class: ClassType,
}

impl Resolver<'_> {
    pub fn new(interpreter: &mut Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: vec![],
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        for (name, value) in self.scopes.last().unwrap() {
            if !value.is_used {
                RuntimeError {
                    message: format!("Local variable `{}` is never read.", name),
                    token: value.token.clone(),
                }
                .error();
            }
        }
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.peek_scopes();
        if scope.contains_key(&name.lexeme) {
            RuntimeError {
                token: name.clone(),
                message: "Already a variable with this name in this scope.".into(),
            }
            .error();
        }
        scope.insert(name.lexeme.clone(), State::new(false, false, name.clone()));
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.peek_scopes();
        match scope.get(&name.lexeme) {
            Some(State { is_used, .. }) => scope.insert(
                name.lexeme.clone(),
                State::new(true, *is_used, name.clone()),
            ),
            None => scope.insert(name.lexeme.clone(), State::new(true, false, name.clone())),
        };
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        stmt::Visitor::visit_stmt(self, stmt);
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        expr::Visitor::visit_expr(self, expr);
    }

    pub fn resolve_block(&mut self, statements: &Vec<Stmt>) {
        for stmt in statements {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_function(
        &mut self,
        parameters: &Vec<Token>,
        body: &Vec<Stmt>,
        _type_: FunctionType,
    ) {
        let enclosing = self.current_function;
        self.current_function = _type_;

        self.begin_scope();
        for param in parameters {
            self.declare(param);
            self.define(param);
        }
        self.resolve_block(body);
        self.end_scope();

        self.current_function = enclosing;
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(&name.lexeme) {
                let hoops_away = self.scopes.len() - 1 - i;
                self.interpreter.resolve(expr, hoops_away);
                return;
            }
        }
    }

    fn visit_block_stmt(&mut self, statements: &Vec<Stmt>) {
        self.begin_scope();
        self.resolve_block(statements);
        self.end_scope();
    }

    fn visit_class_stmt(&mut self, name: &Token, methods: &Vec<Stmt>) {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(name);
        self.define(name);

        self.begin_scope();
        self.peek_scopes()
            .insert("this".into(), State::new(true, true, name.clone()));

        for method in methods {
            if let Stmt::Function {
                parameters,
                body,
                name,
            } = method
            {
                let mut declaration = FunctionType::Method;

                if name.lexeme == "init" {
                    declaration = FunctionType::Initializer;
                }

                self.resolve_function(parameters, body, declaration);
            }
        }

        self.end_scope();
        self.current_class = enclosing_class;
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) {
        self.resolve_expr(expr);
    }

    fn visit_function_stmt(&mut self, name: &Token, parameters: &Vec<Token>, body: &Vec<Stmt>) {
        self.declare(name);
        self.define(name);

        self.resolve_function(parameters, body, FunctionType::Function);
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) {
        self.resolve_expr(condition);
        self.resolve_stmt(then_branch);
        if let Some(else_branch) = else_branch {
            self.resolve_stmt(else_branch);
        }
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &Option<Expr>) {
        if let FunctionType::None = self.current_function {
            RuntimeError {
                token: keyword.clone(),
                message: "Can't return from a top-level code.".into(),
            }
            .error();
        }

        // Statically disallowed return VALUE inside "init"
        if let Some(value) = value {
            if let FunctionType::Initializer = self.current_function {
                return RuntimeError {
                    token: keyword.clone(),
                    message: "Can't return a value from an initializer.".into(),
                }
                .error();
            }

            self.resolve_expr(value);
        }
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) {
        self.declare(name);

        if let Some(value) = initializer {
            self.resolve_expr(value);
        }

        self.define(name);
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) {
        self.resolve_expr(condition);
        self.resolve_stmt(body);
    }

    fn visit_assign_expr(&mut self, var_expr: &Expr, name: &Token, value: &Expr) {
        for i in (0..self.scopes.len()).rev() {
            if let Some(state) = self.scopes[i].get_mut(&name.lexeme) {
                state.is_used = true;
                break;
            }
        }

        self.resolve_expr(value);
        self.resolve_local(var_expr, name);
    }

    fn visit_binary_expr(&mut self, left: &Expr, right: &Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
    }

    fn visit_call_expr(&mut self, callee: &Expr, arguments: &Vec<Expr>) {
        self.resolve_expr(callee);
        for arg in arguments {
            self.resolve_expr(arg);
        }
    }

    fn visit_get_expr(&mut self, expr: &Expr) {
        self.resolve_expr(expr);
    }

    fn visit_grouping_expr(&mut self, expr: &Expr) {
        self.resolve_expr(expr);
    }

    fn visit_literal_expr(&self) {}

    fn visit_logical_expr(&mut self, left: &Expr, right: &Expr) {
        self.resolve_expr(left);
        self.resolve_expr(right);
    }

    fn visit_set_expr(&mut self, value: &Expr, object: &Expr) {
        self.resolve_expr(value);
        self.resolve_expr(object);
    }

    fn visit_this_expr(&mut self, expr: &Expr, keyword: &Token) {
        if let ClassType::None = self.current_class {
            return RuntimeError {
                message: "Can't use 'this' outside of a class.".into(),
                token: keyword.clone(),
            }
            .error();
        }

        self.resolve_local(expr, keyword);
    }

    fn visit_unary_expr(&mut self, right: &Expr) {
        self.visit_expr(right);
    }

    fn visit_var_expr(&mut self, expr: &Expr, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(state) = scope.get_mut(&name.lexeme) {
                state.is_used = true;
                if !state.is_ready {
                    print_error(name, "Can't read local variable in its own initializer.")
                }
            }
        }

        self.resolve_local(expr, name);
    }

    fn peek_scopes(&mut self) -> &mut HashMap<String, State> {
        self.scopes
            .last_mut()
            .expect("Scope's stack must not be empty!")
    }
}

impl stmt::Visitor<()> for Resolver<'_> {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression(expr) => self.visit_expression_stmt(expr),
            Stmt::Class { name, methods } => self.visit_class_stmt(name, methods),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::Block { statements } => self.visit_block_stmt(statements),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
            Stmt::Function {
                name,
                parameters,
                body,
            } => self.visit_function_stmt(name, parameters, body),
            Stmt::Return { keyword, value } => self.visit_return_stmt(keyword, value),
        }
    }
}

impl expr::Visitor<()> for Resolver<'_> {
    fn visit_expr(&mut self, expression: &Expr) {
        match expression {
            Expr::Binary { left, right, .. } => self.visit_binary_expr(left, right),
            Expr::Grouping { expression, .. } => self.visit_grouping_expr(expression),
            Expr::Literal { .. } => self.visit_literal_expr(),
            Expr::Unary { right, .. } => self.visit_unary_expr(right),
            Expr::Variable { name, .. } => self.visit_var_expr(expression, name),
            Expr::Assign { name, value, .. } => self.visit_assign_expr(expression, name, value),
            Expr::Logical { left, right, .. } => self.visit_logical_expr(left, right),
            Expr::Call {
                callee, arguments, ..
            } => self.visit_call_expr(callee, arguments),
            Expr::Get { object, .. } => self.visit_get_expr(object),
            Expr::Set { object, value, .. } => self.visit_set_expr(value, object),
            Expr::This { name, .. } => self.visit_this_expr(expression, name),
        }
    }
}
