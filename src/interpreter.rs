use crate::{
    environment::Environment,
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
    env: Environment,
}

impl Interpreter {
    // Nothing yet (laughs)...
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
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
        //Interpreter::visit_stmt(self, stmt)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        expr::Visitor::visit_expr(self, expr)
        //Interpreter::visit_expr(self, expr)
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
            Value::String(string) => format!("\"{}\"", string),
            Value::Boolean(value) => value.to_string(),
        }
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<()> {
        let value = self.evaluate(expr)?;
        println!("{}", Interpreter::stringfy(&value));
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Option<Expr>) -> Result<()> {
        let mut value = Value::Nil;

        match initializer {
            Some(expr) => {
                value = self.evaluate(expr)?;
            }
            None => (),
        };

        self.env.define(name.lexeme.clone(), value);
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr)?;
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

    fn visit_literal_expr(&self, expr: &Literal) -> Value {
        match expr {
            Literal::String(value) => Value::String(value.clone()),
            Literal::Number(value) => Value::Number(*value),
            Literal::Bool(value) => Value::Boolean(*value),
            Literal::None => Value::Nil,
        }
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
        self.env.get(name)
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
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Expression(expr) => self.visit_expression_stmt(expr),
            Stmt::Var { name, initializer } => self.visit_var_stmt(name, initializer),
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
        }
    }
}
