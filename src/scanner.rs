// scanner.rs 
// author: akrm al-hakimi
// our scanner "class" to parse through and organize source code.
//
// Some notes: 
//         - Rust str is UTF-8. Random indexing by code-point is O(n).
//         - We iterate with Chars (next scalar value each time) but keep current and start as byte
//           offsets so we can slice the original &str cheaply.
//         - After pulling a char, we bump current by ch.len_utf8() so the byte cursor stays in sync.
//         - A multi-byte grapheme still slices correctly because we never split a code-point inside the slice.


// use std::{collections::{HashMap, HashSet}, sync::TryLockError};
use std::{str::Chars};


use crate::{error::ScannerError, token::{Literal, Token}};
use crate::token_type::TokenType;

// Scanner struct to hold the state of the Scanner 
// The impl block contains methods to scan the source code and produce tokens.
// Again, this is our imitation of Java's class behavior in Rust.
pub struct Scanner<'source> {
    source: &'source str, 
    tokens: Vec<Token<'source>>, // This will hold the tokens we produce 
    start: usize,
    current: usize,
    line: usize,
    chars_iter: Chars<'source>, // This is O(n) so perhaps we can optimize this later
}

impl <'source> Scanner<'source> {
    
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0, // &str is byte indexed 
            current: 0,
            line: 1, // but lines always start at 1
            chars_iter: source.chars(), // TODO: optimize this later
        }
    }
    // Scans the source code and returns a vector of tokens.
    pub fn scan_tokens(&mut self) -> Result<Vec<Token<'source>>, ScannerError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        // Add the EOF token at the end of the tokens vector
        self.tokens.push(Token::new(TokenType::Eof, "", None, self.line));
        Ok(std::mem::take(&mut self.tokens))

    }

    fn scan_token(&mut self) -> Result<(), ScannerError> {
        let c = self.advance();
        match c {
            Some('(') => self.add_token(TokenType::LeftParen),
            Some(')') => self.add_token(TokenType::RightParen),
            Some('{') => self.add_token(TokenType::LeftBrace),
            Some('}') => self.add_token(TokenType::RightBrace),
            Some(',') => self.add_token(TokenType::Comma),
            Some('.') => self.add_token(TokenType::Dot),
            Some('-') => self.add_token(TokenType::Minus),
            Some('+') => self.add_token(TokenType::Plus),
            Some(';') => self.add_token(TokenType::Semicolon),
            Some('*') => self.add_token(TokenType::Star),
            Some(c)   => return Err(ScannerError::UnexpectedChar(c, self.line)),
            None      => {},
        }
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars_iter.next()?;
        self.current += ch.len_utf8();
        Some(ch)
    }

    // There is no overload in Rust, so we need to use different methods for adding tokens 
    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None)
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = &self.source[self.start..self.current];
        let token = Token {
            kind: token_type,
            lexeme: text,
            literal,
            line: self.line,
        };
        self.tokens.push(token);
    }
}
