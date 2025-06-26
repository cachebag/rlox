// token.rs 
// author: akrm al-hakimi
// our token "class" for location information and for use in later phases of our interpreter
// this file also includes our token_type enum

use std::fmt::{self, Formatter};

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

// Strongly typed vesion of Java's raw 'Object' literal 
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Str(String),
    Num(f64),
    True,
    False,
    Nil,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
    Question, Colon,

    // One or two character tokens
    Bang, BangEqual, Equal, EqualEqual,
    Greater, GreaterEqual, Less, LessEqual,

    // Literals
    Identifier, String, Number,

    // Keywords
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    Eof,
}

// Imitation of Java's `toString()`
impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {:?}", self.kind, self.lexeme, self.literal)
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Num(n) => write!(f, "{n}"),
            Literal::Str(s) => write!(f, "\"{s}\""),
            Literal::True   => write!(f, "True"),
            Literal::False   => write!(f, "False"),
            Literal::Nil       => write!(f, "nil"),
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let token_str = match self {
            // Single-character tokens
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::LeftBrace => "{",
            TokenType::RightBrace => "}",
            TokenType::Comma => ",",
            TokenType::Dot => ".",
            TokenType::Minus => "-",
            TokenType::Plus => "+",
            TokenType::Semicolon => ";",
            TokenType::Slash => "/",
            TokenType::Star => "*",
            TokenType::Question => "?",
            TokenType::Colon => ":",
            
            // One or two character tokens
            TokenType::Bang => "!",
            TokenType::BangEqual => "!=",
            TokenType::Equal => "=",
            TokenType::EqualEqual => "==",
            TokenType::Greater => ">",
            TokenType::GreaterEqual => ">=",
            TokenType::Less => "<",
            TokenType::LessEqual => "<=",
            
            // Literals
            TokenType::Identifier => "IDENTIFIER",
            TokenType::String => "STRING",
            TokenType::Number => "NUMBER",
            
            // Keywords
            TokenType::And => "and",
            TokenType::Class => "class",
            TokenType::Else => "else",
            TokenType::False => "false",
            TokenType::Fun => "fun",
            TokenType::For => "for",
            TokenType::If => "if",
            TokenType::Nil => "nil",
            TokenType::Or => "or",
            TokenType::Print => "print",
            TokenType::Return => "return",
            TokenType::Super => "super",
            TokenType::This => "this",
            TokenType::True => "true",
            TokenType::Var => "var",
            TokenType::While => "while",
            TokenType::Eof => "EOF",
        };
        write!(f, "{}", token_str)
    }
}
