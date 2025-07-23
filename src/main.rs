// main.rs
// Entry point for the rlox interpreter. Handles CLI arguments, REPL, and file execution.

use std::{
    env, 
    fs, 
    io::{self, Write, Read}, 
    process,
};

use rlox::{
    interpreter::{Interpreter, Value},
    parser::Parser, 
    scanner::Scanner,
    resolver::Resolver,
};

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None => run_prompt(),
        Some(cmd) if cmd == "--show-tokens" || cmd == "show-tokens" => {
            match args.next().as_deref() {
                Some("-") => show_tokens_stdin(),
                Some(path) => show_tokens_file(path),
                None => {
                    eprintln!("Usage: rlox [--]show-tokens <file|->");
                    process::exit(64);
                }
            }
        }
        Some(path) => run_file(path),
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
    let mut scanner = Scanner::new(source);
    let tokens = match scanner.scan_tokens() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Scanner error: {}", e);
            return;
        }
    };

    let mut parser = Parser::new(tokens.clone());
    match parser.parse() {
        Ok(statements) if !statements.is_empty() => {
            let mut resolver = Resolver::new();
            resolver.resolve_stmts(&statements, interpreter);

            let errors = resolver.take_errors();
            if !errors.is_empty() {
                for e in errors {
                    eprintln!("Resolver error: {}", e);
                }
                return;
            }

            if let Err(e) = interpreter.interpret(&statements) {
                eprintln!("Runtime error: {}", e);
            }
        }
        Ok(_) | Err(_) => {
            if !source.contains(';') {
                let mut expr_parser = Parser::new(tokens);
                match expr_parser.expr() {
                    Ok(expr) => match interpreter.evaluate(expr.into()) {
                        Ok(value) => {
                            if !matches!(value, Value::Nil) {
                                // println!("{}", value);
                            }
                        }
                        Err(e) => eprintln!("Runtime error: {}", e),
                    },
                    Err(e) => eprintln!("Parser error: {}", e),
                }
            } else {
                eprintln!("Parser error: could not parse input as statement(s)");
            }
        }
    }
}

fn show_tokens_file(path: &str) {
    let source = fs::read_to_string(path).expect("Could not read file");
    show_tokens(&source);
}

fn show_tokens_stdin() {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source).expect("Failed to read stdin");
    show_tokens(&source);
}

fn show_tokens(source: &str) {
    let mut scanner = Scanner::new(source);
    match scanner.scan_tokens() {
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(e) => {
            eprintln!("Scanner error: {}", e);
        }
    }
}

