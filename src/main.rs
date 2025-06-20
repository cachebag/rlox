// main.rs 
// author: akrm al-hakimi
// the main entry point for the rlox interpreter
// handles file input and interactive REPL prompt 

use std::{
    env,
    fs,
    io::{self, Write},
    process,
    path::{Path},
};
use rlox::{ast::expr, scanner::Scanner};
use rlox::error::ScannerError;
use rlox::ast::expr::Expr;
use rlox::token::{Token, Literal};
use rlox::token_type::TokenType;

// use rlox::token::Token;

// result alias for failing functions
// this is used to simplify the return type of functions
// and prevent boilerplate code 
type Result<T> = std::result::Result<T, ScannerError>;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let exit_code = match args.len() {
        0 => run_prompt(),
        1 => run_file(&args[0]), 
        _ => {
            eprintln!("Usage: rlox [script]");
            Err(ScannerError::Io(io::Error::new(io::ErrorKind::Other, "bad args")))
        }
    }
    .map(|_| 0)
    .unwrap_or_else(|e| { eprintln!("{e}"); 65});

    process::exit(exit_code);
}

fn dummy_token<'src>(kind: TokenType, lexeme: &'src str) -> Token<'src> {
    Token {
        kind,
        lexeme,
        literal: None,
        line: 0,
    }
}


fn run_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let source = fs::read_to_string(path)?;
    run(&source)?;
    Ok(())
}

fn run_prompt() -> Result<()> {
    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut line = String::new();
        if io::stdin().read_line(&mut line)? == 0 { break; }  // EOF

        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
            break;
        }

        run(trimmed)?;     // feed the scanner
    }
    Ok(())
}

fn run(source: &str) -> Result<()> {
    let mut scanner = Scanner::new(source);    // Scanner<' _>
    let tokens  = scanner.scan_tokens()?;

    for token in tokens {
        println!("Token: {:#?}", token)
    }
    println!("{}", source);

    let expression = Expr::binary(
        Expr::unary(
            dummy_token(TokenType::Minus, "-"),
            Expr::Literal(Literal::Num(123.0)),
        ),
        dummy_token(TokenType::Star, "*"),
        Expr::grouping(
            Expr::literal(Literal::Num(45.67)),
        ),
    );

    println!("AST: {}", expression);

    Ok(())
}
