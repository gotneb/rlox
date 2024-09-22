mod token;
mod token_type;
mod scanner;
mod expr;
mod parser;
pub mod ast_printer;

use std::io::{self, Write};

use token::Token;
use token_type::TokenType;

static mut HAD_ERROR: bool = false;

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
        report(
            token.line, 
            format!(" at '{}'", token.lexeme).as_str(), 
            msg
        );
    }
}

pub fn run_file(path: &str) {}

// REPL mode
pub fn run_prompt() {
    loop {
        print!(">> ");
        let mut user_input = String::new();
        let _ = io::stdout().flush();
        let bytes = io::stdin().read_line(&mut user_input).unwrap();

        let user_input = user_input.trim();
        if user_input == "exit" || bytes == 0 {
            break;
        }

        run(user_input);
    }
}

fn run(source: &str) {

}