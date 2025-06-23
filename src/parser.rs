// parser.rs 
// author: akrm al-hakimi
// recursive descent parser for our rlox interpreter
// Design notes:
//              - We go without a few helpers that the book mentions such as `consume`, 'match' and `check`
//               because we can use the `peek` and `advance` methods to handle most of the logic,
//               and match statements to handle the different token types.
//              - We use a single `expr` method to start parsing, which will call the other methods
//              recursively.


use crate::{ token::Literal, token_type::TokenType };
use crate::token::Token;
use crate::ast::expr;

struct Parser<'source> {
    tokens: Vec<Token<'source>>,
    current: usize,
}

impl <'source> Parser<'source> {
    
    pub fn new(tokens: Vec<Token<'source>>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    fn expr(&mut self) -> expr::Expr<'source> {
        self.equality()
    }

    fn equality(&mut self) -> expr::Expr<'source> {
        let mut expr = self.comparison();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenType::BangEqual | TokenType::EqualEqual => {
                    self.advance();
                    let operator = self.previous().clone();
                    let right = self.comparison();
                    expr = expr::Expr::binary(expr, operator, right);
            }
            _ => break,
            }
        }
        expr
    }

    fn comparison(&mut self) -> expr::Expr<'source> {
        let mut expr = self.term();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenType::Greater | TokenType::GreaterEqual
                | TokenType::Less  | TokenType::LessEqual => {
                    self.advance();
                    let operator = self.previous().clone();
                    let right = self.term();
                    expr = expr::Expr::binary(expr, operator, right);
                }
                _ => break,
            }
        }
        expr
    }

    fn term(&mut self) -> expr::Expr<'source> {
        let mut expr = self.factor();

        while let Some(token) = self.peek() {
            match token.kind {
                TokenType::Minus | TokenType::Plus => {
                    self.advance();
                    let operator = self.previous().clone();
                    let right = self.factor();
                    expr = expr::Expr::binary(expr, operator, right);
                }
                _ => break,
            }
        }
        expr
    }

    fn factor(&mut self) -> expr::Expr<'source> {
        let mut expr = self.unary();
        
        while let Some(token) = self.peek() {
            match token.kind {
                TokenType::Slash | TokenType::Star => {
                    self.advance();
                    let operator = self.previous().clone();
                    let right = self.unary();
                    expr = expr::Expr::binary(expr, operator, right);
                }
                _ => break,
            }
        }
        expr
    }

    fn unary(&mut self) -> expr::Expr<'source> {
        if let Some(token) = self.peek() {
            match token.kind {
                TokenType::Bang | TokenType::Minus => {
                    self.advance();
                    let operator = self.previous().clone();
                    let right = self.unary();
                    return expr::Expr::unary(operator, right);
                }
                _ => {}
            }
        }
        self.primary()
    }

    fn primary(&mut self) -> expr::Expr<'source> {
        let token = self.peek().expect("Unexpected end of input");

        match token.kind {
            TokenType::False => {
                self.advance();
                expr::Expr::literal(Literal::False)
            }
            TokenType::True => {
                self.advance();
                expr::Expr::literal(Literal::True)
            }
            TokenType::Nil => {
                self.advance();
                expr::Expr::literal(Literal::Nil)
            }
            TokenType::Number | TokenType::String => {
                let token = self.advance();
                let literal = token.literal
                    .clone()
                    .expect("Literal token missing literal value");
                expr::Expr::literal(literal)
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expr();
                let next = self.peek();
                if next.unwrap().kind == TokenType::RightParen {
                    self.advance();
                    expr::Expr::grouping(expr)
                } else {
                    panic!("Expect ')' after expression.")
                }
            }
            _ => panic!("Expected expression."),
        }
    }

    fn advance(&mut self) -> &Token<'source> {
        if !self.is_at_end() {
            self.current += 1;
        }
       self.previous()
    }

    fn is_at_end(&mut self) -> bool {
        matches!(self.peek(), Some(token) if token.kind == TokenType::Eof)
    }

    fn peek(&mut self) -> Option<&Token<'source>> {
        self.tokens.get(self.current)
    }

    fn previous(&mut self) -> &Token<'source> {
        &self.tokens[self.current - 1]
    }
}
