mod token;
mod token_type;
mod scanner;
mod expr;
pub mod ast_printer;

use std::io::{self, Write};

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