// err.rs
// Defines error types for scanner, parser, runtime, and compiler phases in rlox.

use std::{fmt, io};
use crate::token::{
    TokenType,
    Token,
};
use crate::interpreter::Value;

// In the Java implementation, error handling was more rudimentary,
// but rust almost forces you to handle errors properly.
#[derive(Debug)]
pub enum ScannerError {
    Io(io::Error),
    UnexpectedChar(char, usize),
    UnterminatedString(usize),
    UnterminatedEscape(usize),
    UnterminatedComment(usize),
}

#[derive(Debug)]
pub enum ParserError<'source> {
    Io(io::Error),
    UnterminatedParen{ line: usize },
    UnexpectedExpression{ found: Token<'source>, line: usize },
    UnexpectedToken{ expected: TokenType, found: Token<'source>, line: usize },
    UnexpectedEof{ expected: String, line: usize },
    InvalidAssignmentTarget{ found: Token<'source>, line: usize },
    BreakException{ line: usize },
    TooManyParams{ line: usize },
}

pub enum RuntimeError<'source> {
    Io(io::Error),
    UnaryMinus{ lexeme: String, line: usize },
    BinaryMinus{ lexeme: String, line: usize },
    BinaryPlus{ lexeme: String, line: usize },
    BinaryMult{ lexeme: String, line: usize },
    BinaryDiv{ lexeme: String, line: usize },
    BinaryComp{ lexeme: String, line: usize },
    BinaryDBZ{ line: usize },
    UndefinedVariable{ found: String },
    BreakException,
    MutationError{ lexeme: String, line: usize },
    FunctionError{ lexeme: String, line: usize, message: String },
    ReturnException(Value<'source>),
    TypeError{ msg: String, line: usize },
}

pub enum CompilerError<'source> {
    LocalVarDecl{ name: Token<'source>},
    ExistingVar{ line: usize },
    IllegalReturn{ keyword: Token<'source>},
    ThisOutsideClass{ keyword: Token<'source>},
    InitializerReturn{ keyword: Token<'source>},
    SelfInheritance{ line: usize },
    SuperTypeError{ msg: String, line: usize },
}

impl fmt::Display for CompilerError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::LocalVarDecl { name } => {
                write!(f, "Cannot read local variable in its own initializer. | Error: {}", name)
            }
            CompilerError::ExistingVar { line } => {
                write!(f, "Already a variable with this name in this scope. | Found on line {}", line)
            }
            CompilerError::IllegalReturn { keyword } => {
                write!(f, "Error: {} - Can't return from top level code. (line {})", keyword.lexeme, keyword.line)
            }
            CompilerError::ThisOutsideClass { keyword } => {
                write!(f, "Can't use 'this' outside of class. - {}", keyword)
            }
            CompilerError::InitializerReturn { keyword } => {
                write!(f, "Can't return a value from an initializer. - {}", keyword)
            }
            CompilerError::SelfInheritance { line } => {
                write!(f, "A class can't inherit from itself. Error on line {}", line)
            }
            CompilerError::SuperTypeError { msg, line } => {
                write!(f, "{} on line {}", msg, line)
            }
        }
    }
}

impl fmt::Display for RuntimeError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::Io(e) => write!(f, "io error: {}", e),
            RuntimeError::UnaryMinus { lexeme, line } => {
                write!(f, "Unary minus applied applied to non-number | violator: '{}' on line {}", lexeme, line)
            }
            RuntimeError::BinaryPlus { lexeme, line } => {
                write!(f, "Addition attempted on non-number/non-string values | violator: '{}' on line {}", lexeme, line)
            } 
            RuntimeError::BinaryMinus { lexeme, line } => {
                write!(f, "Subtraction attempted on non-number values | violator: '{}' on line {}", lexeme, line)
            }
            RuntimeError::BinaryMult { lexeme, line } => {
                write!(f, "Multiplication attempted on non-number values | violator: '{}' on line {}", lexeme, line)
            }
            RuntimeError::BinaryDiv { lexeme, line } => {
                write!(f, "Division attempted on non-number values | violator: '{}' on line {}", lexeme, line)
            }
            RuntimeError::BinaryComp { lexeme, line } => {
                write!(f, "Comparison check attempted on non-number/non-string values | violator: '{}' on line {}", lexeme, line)
            }
            RuntimeError::BinaryDBZ { line } => {
                write!(f, "Division by zero  on line {}", line)
            }
            RuntimeError::UndefinedVariable { found } => {
                write!(f, "Undefined variable '{}'. ", found)
            }
            RuntimeError::BreakException => {
                write!(f, "Break statement execute.")
            }
            RuntimeError::MutationError { lexeme, line } => {
                write!(f, "Mutation attempted on illegal expression |'{}' line: {}", lexeme, line)
            }
            RuntimeError::FunctionError { lexeme, line, message } => {
                write!(f, "Here {} on line {} - {}", lexeme, line, message)
            }
            RuntimeError::ReturnException(val) => write!(f, "{}", val),
            RuntimeError::TypeError { msg, line } => {
                write!(f, "{} on line {}", msg, line)
            }
        }
    }
}

// Display implementation for ParserError
impl fmt::Display for ParserError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::Io(e) => write!(f, "io error: {}", e),
            ParserError::UnterminatedParen { line } => {
                write!(f, "You have an unterminated grouping on line {}", line)
            }
            ParserError::UnexpectedToken { expected, found, line } => {
                write!(f, "Expected token '{}' - '{}' on line {}", expected, found, line)
            }
            ParserError::UnexpectedEof { expected, line } => {
                write!(f, "Unexpected end of file: '{}' on line {}", expected, line)
            }
            ParserError::UnexpectedExpression { found, line } => {
                write!(f, "Expected expression, found '{}' on line {}", found, line)
            }
            ParserError::InvalidAssignmentTarget { found, line } => {
                write!(f, "Invalid assignment target '{}' on line {}", found, line)
            }
            ParserError::BreakException { line } => {
                write!(f, "Cannot use break outside of a loop | Issue found on line {}.", line)
            }
            ParserError::TooManyParams { line } => {
                write!(f, "Parameters for a function cannot exceed 255. | line {}.", line)
            }
        }
    }
}

// Display implementation for ScannerError
impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScannerError::Io(e)   => write!(f, "io error: {}", e),
            ScannerError::UnexpectedChar(c, line) =>  {
                write!(f, "Unexpected character '{}' on line {}", c, line)
            }
            ScannerError::UnterminatedString(line) => {
                write!(f, "Unterminated string on line {}", line)
            }
            ScannerError::UnterminatedEscape(line) => {
                write!(f, "Unterminated escape sequence on line {}", line)
            }
            ScannerError::UnterminatedComment(line) => {
                write!(f, "Unterminated comment on line {}", line)
            }  
        }
    }
}

// Implementing the std::error::Error trait for ScannerError
impl std::error::Error for ScannerError {}

// Conversion from io::Error to ScannerError
impl From<io::Error> for ScannerError { 
    fn from(e: io::Error) -> Self { 
        Self::Io(e)
    } 
}
}
