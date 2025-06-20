// token.rs 
// author: akrm al-hakimi
// our token "class" for location information and for use in later phases of our interpreter

use std::fmt::{self, Formatter};
use crate::token_type::TokenType;

// Strongly typed vesion of Java's raw 'Object' literal 
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Str(String),
    Num(f64),
    True,
    False,
    Nil,
}

// This struct + impl allows us to imitate OOP's class behavior. 
// Data and behavior is seperated here but pairing is conceptually the same
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'source> {
    pub kind: TokenType,
    pub lexeme: &'source str, // Just want to borrow a slice, no ownership needed for now
    pub literal: Option<Literal>,
    pub line: usize,
}

impl <'source> Token<'source> {
    
    pub fn new(
        kind: TokenType,
        lexeme: &'source str,
        literal: Option<Literal>,
        line: usize,
    ) -> Self {
        Self { kind, lexeme, literal, line }
    }
}


// Imitation of Java's `toString()`
impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {:?}", self.kind, self.lexeme, self.literal)
    }
}
