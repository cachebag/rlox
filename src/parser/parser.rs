// parser.rs 
// author: akrm al-hakimi
// recursive descent parser for our rlox interpreter
// Design notes:
//              - We go without a few helpers that the book mentions such as `consume`, 'match' and `check`
//               because we can use the `peek` and `advance` methods to handle most of the logic,
//               and match statements to handle the different token types.
//              - We use a single `expr` method to start parsing, which will call the other methods
//              recursively.


use crate::{ ast::expr, 
    error::error::ParserError, 
    token::token::{Token, Literal}, 
    token::token::TokenType,
    ast::stmt::Stmt
};

pub struct Parser<'source> {
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

    pub fn parse(&mut self) -> Result<Vec<Stmt<'source>>, ParserError<'source>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    pub fn expr(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        self.comma()
    }

    fn statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        if self.matches(&[TokenType::Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let value = self.expr()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.").unwrap();
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let expression = self.expr()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(expression))
    }
    fn comma(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut expr = self.ternary()?;
        
        while let Some(token) = self.peek() {
            match token.kind {
                    TokenType::Comma => {
                        self.advance();
                        let operator = self.previous().clone();
                        let right = self.ternary()?;
                        expr = expr::Expr::binary(expr, operator, right);
                    }
                    _ => break,
                }
            }
            Ok(expr)
    }

    fn ternary(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let expr = self.equality()?;

        if let Some(token) = self.peek().cloned() {
            if token.kind == TokenType::Question {
                self.advance();
                let true_expr = self.expr()?;

                if let Some(colon_token) = self.peek() {
                    if colon_token.kind == TokenType::Colon {
                        self.advance();
                        let false_expr = self.ternary()?;
                        return Ok(expr::Expr::ternary(expr, true_expr, false_expr))
                    } else {
                        return Err(ParserError::UnexpectedToken { expected: TokenType::Colon, found: colon_token.clone(), line: token.line });
                    }
                } else {
                    return Err(ParserError::UnexpectedEof { expected: "':'".to_string(), line: self.current_line() });
                }
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut expr = self.comparison()?;

        while let Some(token) = self.peek() {
            match token.kind {
                TokenType::BangEqual | TokenType::EqualEqual => {
                    self.advance();
                    let operator = self.previous().clone();
                    let right = self.comparison()?;
                    expr = expr::Expr::binary(expr, operator, right);
            }
            _ => break,
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut expr = self.term()?;

        while let Some(token) = self.peek() {
            match token.kind {
                TokenType::Greater | TokenType::GreaterEqual
                | TokenType::Less  | TokenType::LessEqual => {
                    self.advance();
                    let operator = self.previous().clone();
                    let right = self.term()?;
                    expr = expr::Expr::binary(expr, operator, right);
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut expr = self.factor()?;

        while let Some(token) = self.peek() {
            match token.kind {
                TokenType::Minus | TokenType::Plus => {
                    self.advance();
                    let operator = self.previous().clone();
                    let right = self.factor()?;
                    expr = expr::Expr::binary(expr, operator, right);
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut expr = self.unary()?;
        
        while let Some(token) = self.peek() {
            match token.kind {
                TokenType::Slash | TokenType::Star => {
                    self.advance();
                    let operator = self.previous().clone();
                    let right = self.unary()?;
                    expr = expr::Expr::binary(expr, operator, right);
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        if let Some(token) = self.peek() {
            match token.kind {
                TokenType::Bang | TokenType::Minus => {
                    self.advance();
                    let operator = self.previous().clone();
                    let right = self.unary()?;
                    return Ok(expr::Expr::unary(operator, right));
                }
                _ => {}
            }
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let token = self.peek().ok_or_else(|| ParserError::UnexpectedEof {
            expected: "expression".to_string(), 
            line: self.current_line() 
        })?.clone();

        match token.kind {
            TokenType::False => {
                self.advance();
                Ok(expr::Expr::literal(Literal::False))
            }
            TokenType::True => {
                self.advance();
                Ok(expr::Expr::literal(Literal::True))
            }
            TokenType::Nil => {
                self.advance();
                Ok(expr::Expr::literal(Literal::Nil))
            }
            TokenType::Number | TokenType::String => {
                let token = self.advance();
                let literal = token.literal
                    .clone()
                    .expect("Literal token missing literal value");
                Ok(expr::Expr::literal(literal))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expr()?;
                let next = self.peek();
                if next.unwrap().kind == TokenType::RightParen {
                    self.advance();
                    Ok(expr::Expr::grouping(expr))
                } else {
                    Err(ParserError::UnterminatedParen { line: token.line })
                }
            }
            _ => Err(ParserError::UnexpectedExpression { found: token.clone(), line: token.line }),
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().kind == TokenType::Semicolon {
                return;
            }

            match self.peek() {
                Some(token) => match token.kind {
                    TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Print
                    | TokenType::Return => return,
                    _ => { 
                        self.advance();
                    }
                },
                None => return,
            }
        }
    }

    fn consume(&mut self, expected: TokenType, message: &str) -> Result<Token<'source>, ParserError<'source>> {
        match self.peek() {
            Some(token) if token.kind == expected => Ok(self.advance()),
            Some(token) => Err(ParserError::UnexpectedToken {
                expected, 
                found: token.clone(), 
                line: token.line 
            }),
            None => Err(ParserError::UnexpectedEof {
                expected: message.to_string(),
                line: self.current_line(),
            }),
        }
    }

    fn advance(&mut self) -> Token<'source> {
        if !self.is_at_end() {
            self.current += 1;
        }
       self.previous().clone()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(token) if token.kind == TokenType::Eof)
    }

    fn peek(&self) -> Option<&Token<'source>> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> &Token<'source> {
        &self.tokens[self.current - 1]
    }

    fn current_line(&self) -> usize {
        self.peek().map(|token| token.line).unwrap_or(1)
    }

    fn matches(&mut self, types: &[TokenType]) -> bool {
        if let Some(token) = self.peek() {
            if types.contains(&token.kind) {
                self.advance();
                return true;
            }
        }
        false
    }

}
