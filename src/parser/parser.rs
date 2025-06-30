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
            if let Some(stmt) = self.declaration() {
                statements.push(stmt);
            }
        }

        Ok(statements)
    }

    pub fn expr(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        self.comma()
    }

    fn declaration(&mut self) -> Option<Stmt<'source>> {
        let result = if self.matches(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match result {
            Ok(stmt) => Some(stmt),
            Err(_) => {
                self.synchronize();
                None
            }
        }
    }

    fn statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        if self.matches(&[TokenType::If]) {
            self.if_statement()
        } else if  self.matches(&[TokenType::Print]) {
            self.print_statement()
        } else if self.matches(&[TokenType::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let cond= self.expr()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let then_br = self.statement()?;
        let mut else_br: Option<Box<Stmt<'source>>> = None;
        if self.matches(&[TokenType::Else]) {
            else_br = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition: cond,
            then_branch: Box::new(then_br), 
            else_branch: else_br, 
        })
    }

    fn print_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let value = self.expr()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn var_declaration(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let value = self.consume(TokenType::Identifier, "Expected variable name.")?;

        let mut init: Option<expr::Expr<'source>> = None;
        if self.matches(&[TokenType::Equal]) {
            init = Some(self.expr()?);
        }

        self.consume(TokenType::Semicolon, "Expect ';' after value.").unwrap();
        Ok(Stmt::Var { 
            name: value, 
            initializer: init 
        })
    }

    fn expression_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let expression = self.expr()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(expression))
    }

    fn block(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let mut statements: Vec<Stmt<'source>> = Vec::new();
        while !self.check(&[TokenType::RightBrace]) && !self.is_at_end() {
            statements.push(self.declaration().unwrap());
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(Stmt::Block(statements))
    }

    fn comma(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut expr = self.assignment()?;
        
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

    fn assignment(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let expr = self.or()?;
        
        // Is the next token '='?
        if let Some(token) = self.peek() {
            if token.kind == TokenType::Equal {
                self.advance(); // consume it 
                // Recursively call assignment to get the value - We are now on the right-hand side
                // of the assignment 
                let value = self.assignment()?;

                // Check if the left-hand side is a variable
                if let expr::Expr::Variable { name } = expr {
                    return Ok(expr::Expr::Assign { name, value: Box::new(value) });
                } else { 
                    // If not, we have an invalid assignment target
                    let token = self.previous();
                    return Err(ParserError::InvalidAssignmentTarget { found: token.clone(), line: token.line });
                }
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut expr = self.and()?;

        while let Some(token) = self.peek() {
            if token.kind == TokenType::Or {
                self.advance();
                let op = self.previous().clone();
                let rhs = self.and()?;
                expr = expr::Expr::logical(expr, op, rhs)
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut expr = self.ternary()?;

        while let Some(token) = self.peek() {
            if token.kind == TokenType::And {
                self.advance();
                let op = self.previous().clone();
                let rhs = self.ternary()?;
                expr = expr::Expr::logical(expr, op, rhs) 
            } else {
                break;
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
            TokenType::Identifier => {
                let identifier = self.advance().clone();
                Ok(expr::Expr::Variable {
                    name: identifier
                })
            }
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

    fn check(&self, matches: &[TokenType]) -> bool {
       if let Some(token) = self.peek() {
            if matches.contains(&token.kind) {
                return true;
            }
        } 
        false 
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
