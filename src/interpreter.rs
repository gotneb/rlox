use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    environment::{EnvRef, Environment},
    impls::{
        callable::Callable,
        class::Class,
        function::{Function, NativeFunction},
    },
    syntax::{
        expr::{self, Expr},
        stmt::{self, Stmt},
        token::{Literal, Token},
        token_type::TokenType,
        value::Value,
    },
    Exception,
};

type Result<T> = std::result::Result<T, Exception>;

pub struct Interpreter {
    pub globals: EnvRef,
    locals: HashMap<Expr, usize>,
    env: EnvRef,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new_global();

        globals.borrow_mut().define(
            "print".into(),
            Value::NativeFunction(NativeFunction {
                arity: 1,
                callable: |_, args| {
                    let value = args.get(0).unwrap().clone();
                    let value = Interpreter::stringfy(&value);
                    println!("{}", value);
                    Value::Nil
                },
            }),
        );

        globals.borrow_mut().define(
            "clock".into(),
            Value::NativeFunction(NativeFunction {
                arity: 0,
                callable: |_, _| {
                    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    Value::Number(timestamp.as_millis() as f64)
                },
            }),
        );

        Self {
            env: globals.clone(),
            globals,
            locals: HashMap::new(),
        }
    }

    // List of statements == actual program
    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for stmt in statements {
            match self.execute(&stmt) {
                Ok(_) => (),
                Err(e) => match e {
                    Exception::RuntimeError(e) => e.error(),
                    // This edge case (`return` keyword on top level code) is handled by the Resolver.
                    Exception::Return(_) => (),
                },
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    pub fn resolve(&mut self, expr: &Expr, depth: usize) {
        self.locals.insert(expr.clone(), depth);
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, env: EnvRef) -> Result<()> {
        let previous = self.env.clone();
        self.env = env;
        for statement in statements {
            if let Err(e) = self.execute(statement) {
                self.env = previous;
                return Err(e);
            }
        }

        self.env = previous;
        Ok(())
    }

    fn evaluate_super_class(
        &mut self,
        class_name: &Token,
        super_class_expr: &Expr,
    ) -> Result<Class> {
        let evaluated = self.evaluate(super_class_expr)?;
        match evaluated {
            Value::Class(class) => Ok(class),
            _ => Exception::runtime_error(
                class_name.clone(),
                String::from("Superclass must be a class"),
            ),
        }
    }

    fn visit_class_stmt(
        &mut self,
        name: &Token,
        getters: &Vec<Stmt>,
        methods: &Vec<Stmt>,
        static_methods: &Vec<Stmt>,
        super_class: &Option<Expr>,
    ) -> Result<()> {
        let super_class = match super_class {
            Some(expr) => {
                let class = self.evaluate_super_class(name, expr)?;
                Some(Box::new(class))
            }
            None => None,
        };

        self.env
            .borrow_mut()
            .define(name.lexeme.clone(), Value::Nil);

        let prev_env = self.env.clone();
        if let Some(super_class) = &super_class {
            self.env = Environment::new_local(&self.env);
            self.env
                .borrow_mut()
                .define("super".into(), Value::Class(*super_class.clone()));
        }

        let mut class_getters = HashMap::new();
        let mut class_methods = HashMap::new();
        let mut class_static_methods = HashMap::new();

        for getter in getters {
            match getter {
                Stmt::Function { name, .. } => {
                    let function = Function::new(getter.clone(), self.env.clone(), false);
                    class_getters.insert(name.lexeme.clone(), function);
                }
                _ => panic!("Stmt is not a getter!"),
            };
        }

        for static_method in static_methods {
            match static_method {
                Stmt::Function { name, .. } => {
                    let function = Function::new(static_method.clone(), self.env.clone(), false);
                    class_static_methods.insert(name.lexeme.clone(), function);
                }
                _ => panic!("Stmt is not a static method!"),
            };
        }

        for method in methods {
            match method {
                Stmt::Function { name, .. } => {
                    let function =
                        Function::new(method.clone(), self.env.clone(), name.lexeme == "init");
                    class_methods.insert(name.lexeme.clone(), function);
                }
                _ => panic!("Stmt is not a method!"),
            };
        }

        let class = Class::new(
            class_getters,
            name.lexeme.clone(),
            class_methods,
            class_static_methods,
            super_class.clone(),
        );

        if super_class.is_some() {
            self.env = prev_env;
        }

        self.env.borrow_mut().assign(name, Value::Class(class))?;

        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        expr::Visitor::visit_expr(self, expr)
    }

    fn is_equal(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Nil, Value::Nil) => true,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            _ => false,
        }
    }

    fn stringfy(value: &Value) -> String {
        match value {
            Value::Nil => "nil".into(),
            Value::Number(number) => {
                let number = number.to_string();
                if number.ends_with(".0") {
                    return number.chars().take(number.len() - 2).collect();
                }

                number
            }
            Value::String(string) => string.clone(),
            Value::Boolean(value) => value.to_string(),
            Value::Function(f) => {
                if let Stmt::Function { name, .. } = &f.declaration {
                    return format!("<fn {}>", name.lexeme);
                }
                // In theory, it must never happen!
                "<unknown function>".into()
            }
            Value::NativeFunction(_) => "<native fn>".into(),
            Value::Class(class) => class.to_string(),
            Value::ClassInstance(class_instance) => class_instance.borrow().to_string(),
        }
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<()> {
        let mut value = Value::Nil;

        match initializer {
            Some(expr) => {
                value = self.evaluate(expr)?;
            }
            None => (),
        };

        self.env.borrow_mut().define(name.lexeme.clone(), value);
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        while Interpreter::is_truthy(&self.evaluate(condition)?) {
            self.execute(body)?;
        }
        Ok(())
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr, expr: &Expr) -> Result<Value> {
        let value = self.evaluate(value)?;

        let distance = self.locals.get(expr);
        match distance {
            Some(distance) => self.env.borrow_mut().assign_at(*distance, name, &value),
            None => {
                self.globals.borrow_mut().assign(name, value.clone())?;
            }
        };

        Ok(value)
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_function_stmt(&mut self, name: &Token, function_stmt: &Stmt) -> Result<()> {
        let function = Function::new(function_stmt.clone(), self.env.clone(), false);
        self.env
            .borrow_mut()
            .define(name.lexeme.clone(), Value::Function(function));

        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<()> {
        if Interpreter::is_truthy(&self.evaluate(condition)?) {
            self.execute(&then_branch)?;
        } else {
            if let Some(else_branch) = else_branch {
                self.execute(else_branch)?;
            }
        }
        Ok(())
    }

    fn visit_return_stmt(&mut self, value: &Option<Expr>) -> Result<()> {
        match value {
            Some(expr) => Err(Exception::Return(self.evaluate(expr)?)),
            None => Err(Exception::Return(Value::Nil)),
        }
    }

    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match operator.token_type {
            // Equality
            // --------------------------------------
            TokenType::BangEqual => Ok(Value::Boolean(!Interpreter::is_equal(&left, &right))),
            TokenType::EqualEqual => Ok(Value::Boolean(Interpreter::is_equal(&left, &right))),
            // Logic
            // --------------------------------------
            TokenType::Greater => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left > right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::GreaterEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left >= right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::Less => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left < right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::LessEqual => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Boolean(left <= right)),
                _ => Interpreter::number_operands_error(operator),
            },
            // Arithmetic
            // --------------------------------------
            TokenType::Minus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left - right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::Slash => match (left, right) {
                (Value::Number(left), Value::Number(right)) => {
                    if right == 0.0 {
                        return Interpreter::zero_division_error(operator);
                    }
                    Ok(Value::Number(left / right))
                }
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::Star => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                _ => Interpreter::number_operands_error(operator),
            },
            TokenType::Plus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => {
                    let mut s = left.clone();
                    s.push_str(&right);
                    Ok(Value::String(s))
                }
                // Overlord 'string' + 'number'
                (Value::String(string), Value::Number(number)) => {
                    Ok(Value::String(format!("{}{}", string, number)))
                }
                (Value::Number(number), Value::String(string)) => {
                    Ok(Value::String(format!("{}{}", number, string)))
                }
                _ => Interpreter::number_operands_error(operator),
            },
            _ => panic!("Operands not recognized!"),
        }
    }

    fn visit_call_expr(&mut self, callee: &Expr, paren: &Token, args: &Vec<Expr>) -> Result<Value> {
        let callee = self.evaluate(callee)?;

        let mut evaluated_args = vec![];

        for arg in args {
            evaluated_args.push(self.evaluate(arg)?);
        }

        match callee {
            Value::Function(callee) => {
                callee.check_arity(evaluated_args.len(), paren)?;
                callee.call(self, evaluated_args)
            }
            Value::NativeFunction(callee) => {
                callee.check_arity(evaluated_args.len(), paren)?;
                callee.call(self, evaluated_args)
            }
            Value::Class(callee) => {
                callee.check_arity(evaluated_args.len(), paren)?;
                callee.call(self, evaluated_args)
            }
            _ => Exception::runtime_error(
                paren.clone(),
                "Can only call functions and classes.".into(),
            ),
        }
    }

    fn visit_get_expr(&mut self, name: &Token, object: &Expr) -> Result<Value> {
        let object = self.evaluate(object)?;
        match object {
            Value::ClassInstance(instance) => instance.borrow().get(name, instance.clone(), self),
            Value::Class(class) => class.get(name),
            _ => Exception::runtime_error(name.clone(), "Only instances have property".into()),
        }
    }

    fn visit_literal_expr(&self, expr: &Literal) -> Value {
        match expr {
            Literal::String(value) => Value::String(value.clone()),
            Literal::Number(value) => Value::Number(*value),
            Literal::Bool(value) => Value::Boolean(*value),
            Literal::None => Value::Nil,
        }
    }

    fn visit_logical_expr(
        &mut self,
        left: &Box<Expr>,
        operator: &Token,
        right: &Box<Expr>,
    ) -> Result<Value> {
        let left = self.evaluate(left)?;

        if operator.token_type == TokenType::Or {
            if Interpreter::is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !Interpreter::is_truthy(&left) {
                return Ok(left);
            }
        }

        self.evaluate(right)
    }

    fn visit_set_expr(&mut self, name: &Token, object: &Expr, value: &Expr) -> Result<Value> {
        let object = self.evaluate(object)?;
        match object {
            Value::ClassInstance(instance) => {
                let value = self.evaluate(value)?;
                instance.borrow_mut().set(name, &value)?;
                Ok(value)
            }
            _ => Exception::runtime_error(name.clone(), "Only instances have fields.".into()),
        }
    }

    fn visit_super_expr(&mut self, expr: &Expr, method: &Token) -> Result<Value> {
        let distance = self
            .locals
            .get(expr)
            .expect("Super class haven't been resolved");

        let super_class = self
            .env
            .borrow()
            .get_at(*distance, &"super".into())?;
        
        let super_class = match super_class {
            Value::Class(super_class) => super_class,
            _ => panic!("Expecteded superclass to be a class!"),
        };

        let object = self
            .env
            .borrow()
            // "this" is always right inside where "super" is stored
            .get_at(*distance - 1, &"this".into())?;
        let object = match object {
            Value::ClassInstance(instance) => instance,
            _ => panic!("Expecteded 'this' to be a class instance!"),
        };

        let method = super_class.find_method(&method.lexeme).ok_or_else(|| {
            Exception::runtime_error::<()>(
                method.clone(),
                format!("Undefined property {}.", method.lexeme),
            )
            .unwrap_err()
        })?;

        match method {
            Value::Function(method) => Ok(Value::Function(method.bind(object))),
            _ => panic!("Expected method to be a function!"),
        }
    }

    fn visit_this_expr(&mut self, keyword: &Token, expr: &Expr) -> Result<Value> {
        self.loopkup_variable(keyword, expr)
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<Value> {
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::Minus => match right {
                Value::Number(number) => Ok(Value::Number(-number)),
                _ => Interpreter::number_operand_error(operator),
            },
            TokenType::Bang => Ok(Value::Boolean(!Interpreter::is_truthy(&right))),
            _ => todo!(),
        }
    }

    fn visit_variable_expr(&self, name: &Token, expr: &Expr) -> Result<Value> {
        // self.env.borrow().get(name)
        self.loopkup_variable(name, expr)
    }

    fn loopkup_variable(&self, name: &Token, expr: &Expr) -> Result<Value> {
        let distance = self.locals.get(expr);

        if let Some(distance) = distance {
            self.env.borrow().get_at(*distance, &name.lexeme)
        } else {
            self.globals.borrow().get(name)
        }
    }

    fn zero_division_error<T>(operator: &Token) -> Result<T> {
        Exception::runtime_error(operator.clone(), "Zero division error.".into())
    }

    fn number_operand_error<T>(operator: &Token) -> Result<T> {
        Exception::runtime_error(operator.clone(), "Operand must be a number.".into())
    }

    fn number_operands_error<T>(operator: &Token) -> Result<T> {
        Exception::runtime_error(operator.clone(), "Operands must be numbers.".into())
    }

    // Lox folows Ruby's rule: false and nil are false, everything else is true
    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Boolean(value) => *value,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl stmt::Visitor<Result<()>> for Interpreter {
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(expr) => self.visit_expression_stmt(expr),
            Stmt::Class {
                getters,
                name,
                methods,
                static_methods,
                super_class,
            } => self.visit_class_stmt(name, getters, methods, static_methods, super_class),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
            Stmt::Block { statements } => {
                self.execute_block(statements, Environment::new_local(&self.env))
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => self.visit_while_stmt(condition, body),
            Stmt::Function { name, .. } => self.visit_function_stmt(name, stmt),
            Stmt::Return { value, .. } => self.visit_return_stmt(value),
        }
    }
}

impl expr::Visitor<Result<Value>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
                ..
            } => self.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression, .. } => self.evaluate(expression),
            Expr::Literal { value, .. } => Ok(self.visit_literal_expr(value)),
            Expr::Unary {
                operator, right, ..
            } => self.visit_unary_expr(operator, right),
            Expr::Variable { name, .. } => self.visit_variable_expr(name, expr),
            Expr::Assign { name, value, .. } => self.visit_assign_expr(name, value, expr),
            Expr::Logical {
                left,
                operator,
                right,
                ..
            } => self.visit_logical_expr(left, operator, right),
            Expr::Call {
                callee,
                paren,
                arguments,
                ..
            } => self.visit_call_expr(callee, paren, arguments),
            Expr::Get { name, object, .. } => self.visit_get_expr(name, object),
            Expr::Set {
                name,
                object,
                value,
                ..
            } => self.visit_set_expr(name, object, value),
            Expr::This { name, .. } => self.visit_this_expr(name, expr),
            Expr::Super { method, .. } => self.visit_super_expr(expr, method),
        }
    }
}
