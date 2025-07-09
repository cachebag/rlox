// error.rs 
// author: akrm al-hakimi
// error types for the scanner and any future components 

use std::{fmt, io};
use crate::token::token::TokenType;
use crate::token::token::Token;
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
