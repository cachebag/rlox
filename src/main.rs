// main.rs 
// author: akrm al-hakimi
// minimal REPL implementation for rlox interpreter

use std::{
    env, 
    fs, 
    io::{self, Write}, 
    process
};

use rlox::{
    interpreter::{Interpreter, Value},
    parser::parser::Parser, 
    scanner::scanner::Scanner,
};

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
    let source = fs::read_to_string(path).expect("Could not read file");
    let mut interpreter = Interpreter::<'_>::new();
    run(&source, &mut interpreter);
}

fn run_prompt() {
    let stdin = io::stdin();
    let mut line_buf = String::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        line_buf.clear();

        if stdin.read_line(&mut line_buf).unwrap() == 0 {
            break;
        }

        let input = line_buf.trim();
        if input.is_empty() {
            continue;
        }

        let mut interpreter = Interpreter::<'_>::new();
        run(input, &mut interpreter);
    }
}

fn run<'source>(source: &'source str, interpreter: &mut Interpreter<'source>) {
    // Scanner: source -> tokens
    let mut scanner = Scanner::new(source);
    let tokens = match scanner.scan_tokens() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Scanner error: {}", e);
            return;
        }
    };

    // Try to parse as statements first
    let mut parser = Parser::new(tokens.clone());
    match parser.parse() {
        Ok(statements) if !statements.is_empty() => {
            if let Err(e) = interpreter.interpret(statements) {
                eprintln!("Runtime error: {}", e);
            }
        }
        _ => {
            let mut expr_parser = Parser::new(tokens);
            match expr_parser.expr() {
                Ok(expr) => match interpreter.evaluate(expr) {
                    Ok(value) => {
                        if !matches!(value, Value::Nil) {
                            println!("{}", value);
                        }
                    }
                    Err(e) => eprintln!("Runtime error: {}", e),
                },
                Err(e) => eprintln!("Parser error: {}", e),
            }
        }
    }
}

