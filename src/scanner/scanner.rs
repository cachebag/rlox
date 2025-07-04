// scanner.rs 
// author: akrm al-hakimi
// our scanner "class" to parse through and organize source code.


use std::{collections::HashMap, sync::RwLock};
use once_cell::sync::Lazy;

use crate::{error::error::ScannerError, token::token::{Literal, Token}};
use crate::token::token::TokenType;

// Scanner struct to hold the state of the Scanner 
// The impl block contains methods to scan the source code and produce tokens.
// Again, this is our imitation of Java's class behavior in Rust.
pub struct Scanner<'source> {
    source: &'source str, 
    tokens: Vec<Token<'source>>, // This will hold the tokens we produce 
    start: usize,
    current: usize,
    line: usize,
}

// This is our "static initializer" for keywords.
// 
static KEYWORDS: Lazy<RwLock<HashMap<&'static str, TokenType>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("and",          TokenType::And);
    m.insert("class",        TokenType::Class);
    m.insert("else",         TokenType::Else);
    m.insert("false",        TokenType::False);
    m.insert("for",          TokenType::For);
    m.insert("fn",          TokenType::Fn);
    m.insert("if",           TokenType::If);
    m.insert("nil",          TokenType::Nil);
    m.insert("or",           TokenType::Or);
    m.insert("print",        TokenType::Print);
    m.insert("return",       TokenType::Return);
    m.insert("super",        TokenType::Super);
    m.insert("this",         TokenType::This);
    m.insert("true",         TokenType::True);
    m.insert("var",          TokenType::Var);
    m.insert("while",        TokenType::While);
    m.insert("break",         TokenType::Break);
    RwLock::new(m)
}); 

impl <'source> Scanner<'source> {
    
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0, // &str is byte indexed 
            current: 0,
            line: 1, // but lines always start at 1
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
        // In Rust, match arms implicitly break
        match c {
            Some('(') => self.add_token(TokenType::LeftParen),
            Some(')') => self.add_token(TokenType::RightParen),
            Some('{') => self.add_token(TokenType::LeftBrace),
            Some('}') => self.add_token(TokenType::RightBrace),
            Some(',') => self.add_token(TokenType::Comma),
            Some('.') => self.add_token(TokenType::Dot),
            Some('-') => {
                let kind = if self.match_char('-') {
                    TokenType::Decrement
                } else {
                    TokenType::Minus
                };
                self.add_token(kind);
            },
            Some('+') => {
                let kind = if self.match_char('+') {
                    TokenType::Increment
                } else {
                    TokenType::Plus
                };
                self.add_token(kind);
            },
            Some(';') => self.add_token(TokenType::Semicolon),
            Some('*') => self.add_token(TokenType::Star),
            Some('?') => self.add_token(TokenType::Question),
            Some(':') => self.add_token(TokenType::Colon),
            Some('!') => {
                let kind = if self.match_char('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(kind);
            }
            Some('=') => {
                let kind = if self.match_char('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(kind);
            }
            Some('<') => {
                let kind = if self.match_char('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(kind);
            }
            Some('>') => {
                let kind = if self.match_char('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(kind);
            }
            Some('/') => {
                let kind: Option<TokenType> = if self.match_char('/') {
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        self.advance();
                    }
                    None
                } else if self.match_char('*') {
                    self.consume_multiline_comment()?;
                    None
                } else {
                    Some(TokenType::Slash)
                };
                if let Some(k) = kind {
                    self.add_token(k);
                }
            }
            Some(' ')  => {},
            Some('\r') => {},
            Some('\t') => {},
            Some('\n') => self.line += 1,
            Some('"')  => self.string()?,
            Some(c)   =>  {
                if self.is_digit(c) {
                    self.number()?;
                } else if self.is_alpha(c) {
                    self.identifier();
                } else {
                return Err(ScannerError::UnexpectedChar(c, self.line))
                };
            }
            None       => {},
        };
        Ok(())
    }

    fn identifier(&mut self) {
        // we don't want rusts built in is_alphabetic() because it includes non-ascii characters
        while self.peek().map(|c| self.is_alpha_numeric(c)).unwrap_or(false) { 
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_type = KEYWORDS
            .read()
            .expect("Failed to read keywords")
            .get(text)
            .cloned()
            .unwrap_or(TokenType::Identifier);
        self.add_token(token_type)
    }

    fn number(&mut self) -> Result<(), ScannerError> {
        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            self.advance();
        }

        if self.peek() == Some('.') &&
            self.peek_next().map(|c| c.is_ascii_digit()).unwrap_or(false)
        {
            self.advance(); // consume the '.'
            
            while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                self.advance();
            }
        }

        let lexeme = &self.source[self.start..self.current];
        let value = lexeme.parse::<f64>().expect("valid float");

        self.add_token_with_literal(TokenType::Number, Some(Literal::Num(value)));
        Ok(())
    }

    fn string(&mut self) -> Result<(), ScannerError> {
        while let Some(ch) = self.peek() {
            match ch {
                '"'  => break,                  // End of string  
                '\n' => { self.line += 1; },    // Newline, increment line count
                '\\' => {                       // Escape sequence  
                    self.advance();
                    if self.is_at_end() {
                        return Err(ScannerError::UnterminatedEscape(self.line));
                    }
                    self.advance(); // Again because we already peeked
                }
                _   => { self.advance(); } // Just consume the character
            };
        }
        
        // If we reach here, we either found a closing quote or reached the end of the source
        if self.is_at_end() {
            return Err(ScannerError::UnterminatedString(self.line));
        }
        // We found the closing quote, so we consume  it
        self.advance();

        let lexeme = &self.source[self.start + 1 .. self.current - 1];
        self.add_token_with_literal(TokenType::String, Some(Literal::Str(lexeme.to_owned())));
        Ok(())
    }
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() { return false; }

        // SAFETY: current is always a char boundary by construction 
        let slice = &self.source[self.current..];
        let mut chars = slice.chars();
        if chars.next() != Some(expected) {
            return false;
        }

        self.current += expected.len_utf8();   // move by byte length
        true
    }

    fn peek(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut it = self.source[self.current..].chars();
        it.next()?;
        it.next()
    }

    fn is_alpha(&mut self, c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alpha_numeric(&mut self, c:char) -> bool {
        self.is_alpha(c) || c.is_ascii_digit()
    }

    fn is_digit(&mut self, c: char) -> bool {
        return c >= '0' && c <= '9'
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.current += ch.len_utf8();
        Some(ch)
    }

    
    fn consume_multiline_comment(&mut self) -> Result<(), ScannerError> {
        loop {
            match self.peek() {
                Some('*') => {
                    if self.peek_next() == Some('/') {
                        self.advance(); // consume '*'
                        self.advance(); // consume '/'
                        return Ok(());
                    }
                    self.advance(); // lone '*', keep scanning
                }
                Some('\n') => {
                    self.line += 1; // increment line counter for newlines in comments
                    self.advance();
                }
                Some(_) => {
                    self.advance(); // any other char
                }
                None => return Err(ScannerError::UnterminatedComment(self.line)), // reached EOF
            }
        }
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
