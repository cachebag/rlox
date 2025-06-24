// error.rs 
// author: akrm al-hakimi
// error types for the scanner and any future components 

use std::{fmt, io, process::exit};
use crate::token_type::TokenType;
use crate::token::Token;

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

pub enum ParserError<'source> {
    Io(io::Error),
    UnterminatedParen{ line: usize },
    UnexpectedExpression{ found: Token<'source>, line: usize },
    UnexpectedToken{ expected: TokenType, found: Token<'source>, line: usize },
    UnexpectedEof{ expected: String, line: usize },
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
