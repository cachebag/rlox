// error.rs 
// author: akrm al-hakimi
// error types for the scanner and any future components 

use std::{fmt, io};

// In the Java implementation, error handling was more rudimentary,
// but rust almost forces you to handle errors properly.
#[derive(Debug)]
pub enum ScannerError {
    Io(io::Error),
    UnexpectedChar(char, usize),
    UnterminatedString(usize),
    UnterminatedEscape(usize),
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
                write!(f, "Untermianted escape sequence on line {}", line)
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
