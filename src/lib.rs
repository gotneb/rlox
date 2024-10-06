mod utils;
mod interpreter;
mod parser;
mod scanner;
mod syntax;

use std::{
    fs,
    io::{self, Write},
    process,
};

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use syntax::{token::Token, token_type::TokenType};

static mut HAD_ERROR: bool = false;
static mut HAD_RUNTIME_ERROR: bool = false;

enum Exception {
    RuntimeError(RuntimeError),
}

impl Exception {
    fn runtime_error<T>(token: Token, message: String) -> Result<T, Exception> {
        Err(Exception::RuntimeError(RuntimeError { token, message }))
    }
}

struct RuntimeError {
    token: Token,
    message: String,
}

impl RuntimeError {
    fn error(&self) {
        println!("{}", self.message);
        println!("[line {}]", self.token.line);

        unsafe { HAD_RUNTIME_ERROR = true }
    }
}

// TODO: In page 42 there's a place to check runtime error (a.k.a HAD_ERROR)
// I haven't yet done this due that functions are still in process making.

pub fn error(line: usize, message: &str) {
    report(line, "", message);
}

pub fn report(line: usize, location: &str, message: &str) {
    eprintln!("[Line {}] Error {}: {}", line, location, message);
    // Aww men... Here goes unsafe :( Is there another way to make this?
    unsafe { HAD_ERROR = true };
}

pub fn print_error(token: &Token, msg: &str) {
    if token.token_type == TokenType::Eof {
        report(token.line, " at end", msg);
    } else {
        report(token.line, format!("at '{}'", token.lexeme).as_str(), msg);
    }
}

pub fn run_file(path: &str) {
    let mut interpreter = Interpreter::new();
    let contents = fs::read_to_string(path).expect("File must be readable");
    run(contents, &mut interpreter);

    unsafe {
        if HAD_RUNTIME_ERROR {
            process::exit(70)
        }
    }
}

// REPL mode
pub fn run_prompt() {
    let mut interpreter = Interpreter::new();

    loop {
        print!(">>> ");
        let mut user_input = String::new();
        let _ = io::stdout().flush();
        let bytes = io::stdin().read_line(&mut user_input).unwrap();

        let user_input = user_input.trim();
        if user_input == "exit" || bytes == 0 {
            break;
        }

        run(user_input.into(), &mut interpreter);
    }
}

fn run(source: String, interpreter: &mut Interpreter) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(expr) => interpreter.interpret(&expr),
        Err(err) => println!("{:?}", err),
    }
}
