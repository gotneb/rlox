use crate::{
    expr::{self, Expr, Visitor},
    token::{Literal, Token},
    token_type::TokenType,
    value::Value, Exception,
};

type Result<T> = std::result::Result<T, Exception>;

pub struct Interpreter;

impl Interpreter {
    // Nothing wet (laughs)...
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret(&mut self, expr: &Expr) {
        match self.evaluate(expr) {
            Ok(value) => println!("{}", Interpreter::stringfy(&value)),
            Err(e) => match e {
                Exception::RuntimeError(e) => e.error(),
            }
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        // expr::Visitor::visit_expr(self, expr)
        Interpreter::visit_expr(self, expr)
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
                    return number.chars().take(number.len()-2).collect()
                }

                number
            },
            _ => format!("{:?}", value),
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
                _ => Interpreter::number_operands_error(operator)
            },
            TokenType::Slash => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left / right)),
                _ => Interpreter::number_operands_error(operator)
            },
            TokenType::Star => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left * right)),
                _ => Interpreter::number_operands_error(operator)
            }
            TokenType::Plus => match (left, right) {
                (Value::Number(left), Value::Number(right)) => Ok(Value::Number(left + right)),
                (Value::String(left), Value::String(right)) => {
                    let mut s = left.clone();
                    s.push_str(&right);
                    Ok(Value::String(s))
                },
                _ => Interpreter::number_operands_error(operator)
            }
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
            TokenType::Bang => Ok(Value::Boolean(Interpreter::is_truthy(&right))),
            _ => todo!(),
        }
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

impl expr::Visitor<Result<Value>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Binary { left, operator, right } => self.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => self.evaluate(expression),
            Expr::Literal { value } => Ok(self.visit_literal_expr(value)),
            Expr::Unary { operator, right } => self.visit_unary_expr(operator, right),
        }
    }
}