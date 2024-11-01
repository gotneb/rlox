use crate::{
    environment::{EnvRef, Environment},
    impls::{callable::Callable, function::NativeFunction},
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
    globals: EnvRef,
    env: EnvRef,
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Environment::new_global();

        global.borrow_mut().define(
            "print".into(),
            Value::NativeFunction(NativeFunction {
                arity: 1,
                callable: |_, args| args.get(0).unwrap().clone(),
            }),
        );

        Self {
            globals: global.clone(),
            env: global.clone(),
        }
    }

    // List of statements == actual program
    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for stmt in statements {
            match self.execute(&stmt) {
                Ok(_) => (),
                Err(e) => match e {
                    Exception::RuntimeError(e) => e.error(),
                },
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt::Visitor::visit_stmt(self, stmt)
    }

    fn execute_block(&mut self, statements: &Vec<Stmt>, env: EnvRef) -> Result<()> {
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
            Value::Function(_) => "<fn>".into(),
            Value::NativeFunction(_) => "<native fn>".into(),
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

    fn visit_assign_expr(&mut self, name: &Token, expr: &Expr) -> Result<Value> {
        let value = self.evaluate(expr)?;
        self.env.borrow_mut().assign(name, value)
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr)?;
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
            _ => todo!(),
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
                callee.call(self, evaluated_args)
            },
            Value::NativeFunction(callee) => {
                callee.check_arity(evaluated_args.len(), paren)?;
                callee.call(self, evaluated_args)
            }
            _ => Exception::runtime_error(
                paren.clone(),
                "Can only call functions and classes.".into(),
            ),
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

    fn visit_variable_expr(&self, name: &Token) -> Result<Value> {
        self.env.borrow().get(name)
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
            } => self.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Literal { value } => Ok(self.visit_literal_expr(value)),
            Expr::Unary { operator, right } => self.visit_unary_expr(operator, right),
            Expr::Variable { name } => self.visit_variable_expr(name),
            Expr::Assign { name, value } => self.visit_assign_expr(name, value),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.visit_logical_expr(left, operator, right),
            Expr::Call {
                callee,
                paren,
                arguments,
            } => self.visit_call_expr(callee, paren, arguments),
        }
    }
}
