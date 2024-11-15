use std::{env, process};

use rlox::{run_file, run_prompt};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    match args.len() {
        // No arguments passed. Shows REPL.
        1 => run_prompt(),
        // '.lox' file passed. Runs file's source code.
        2 => run_file(args.get(1).unwrap().as_str()),
        // Bad usage. Shows message.
        _ => {
            println!("Usage: jlox [script]");
            process::exit(64)
        }
    }
}
