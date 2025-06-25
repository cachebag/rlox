// main.rs 
// author: akrm al-hakimi
// minimal REPL implementation for rlox interpreter

use std::{
    env,
    fs,
    io::{self, Write},
    process,
};
use rlox::{interpreter::Interpreter, scanner::Scanner, parser::Parser};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    
    match args.len() {
        0 => run_prompt(),
        1 => run_file(&args[0]),
        _ => {
            eprintln!("Usage: rlox [script]");
            process::exit(64);
        }
    }
}

fn run_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(source) => run(&source),
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            process::exit(66);
        }
    }
}

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                    break;
                }
                run(trimmed);
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn run(source: &str) {
    // Scanner: source -> tokens
    let mut scanner = Scanner::new(source);
    let tokens = match scanner.scan_tokens() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Scanner error: {}", e);
            return;
        }
    };
    
    // Parser: tokens -> AST
    let mut parser = Parser::new(tokens);
    let expr = match parser.parse() {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("Parser error: {}", e);
            return;
        }
    };
    
    // Interpreter: AST -> result
    let mut interpreter = Interpreter::new();
    if let Err(e) = interpreter.interpret(&expr) {
        eprintln!("Runtime error: {}", e);
    }
}
