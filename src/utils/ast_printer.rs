use crate::syntax::{
    expr::{Expr, Visitor},
    token::{Literal, Token},
    token_type::TokenType,
};

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: Expr) -> String {
        self.visit_expr(&expr)
    }

    fn parenthesize(&mut self, name: &str, exprs: Vec<&Expr>) -> String {
        let mut str = String::from(format!("({}", name));

        for expr in exprs {
            str.push(' ');
            str.push_str(&self.visit_expr(expr));
        }
        str.push_str(")");

        str
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, vec![left, right]),
            Expr::Grouping { expression } => self.parenthesize("group", vec![expression]),
            Expr::Literal { value } => match value {
                Literal::Bool(value) => value.to_string(),
                Literal::Number(value) => value.to_string(),
                Literal::String(value) => value.to_string(),
                Literal::None => "nil".into(),
            },
            Expr::Unary { operator, right } => self.parenthesize(&operator.lexeme, vec![right]),
            Expr::Variable { name: _ } => todo!(),
            Expr::Assign { name: _, value: _ } => todo!(),
            Expr::Logical { left: _, operator: _, right: _ } => todo!(),
        }
    }
}

pub fn test_ast_print() {
    let expression = Expr::Binary {
        left: Box::new(Expr::Unary {
            operator: Token {
                token_type: TokenType::Minus,
                lexeme: String::from("-"),
                literal: Literal::None,
                line: 1,
            },
            right: Box::new(Expr::Literal {
                value: Literal::Number(123.0),
            }),
        }),
        operator: Token {
            token_type: TokenType::Star,
            lexeme: String::from("*"),
            literal: Literal::None,
            line: 1,
        },
        right: Box::new(Expr::Grouping {
            expression: Box::new(Expr::Literal {
                value: Literal::Number(45.67),
            }),
        }),
    };

    let mut printer = AstPrinter;
    let result = printer.print(expression);
    println!("{}", result);
}
