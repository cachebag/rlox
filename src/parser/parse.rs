// parse.rs 
// author: akrm al-hakimi
// recursive descent parser for our rlox interpreter
// Design notes:
//         - The parser uses a recursive descent approach, which is simple and effective for our
//         grammar.
//         - Unlike Java, Rust's ownership model allows us to use references and lifetimes to manage 
//         the lifetime of tokens and expressions without needing to pass around mutable references.

use crate::{
    ast::{
        expr, 
        stmt::{FunctionDecl, Stmt}
    }, 
    error::ParserError, 
    token::{
        Literal, 
        Token, 
        TokenType
    }
};
use std::rc::Rc;

pub struct Parser<'source> {
    tokens: Vec<Token<'source>>,
    current: usize,
    loop_depth: usize,
}

impl <'source> Parser<'source> {
    
    pub fn new(tokens: Vec<Token<'source>>) -> Self {
        Self {
            tokens,
            current: 0,
            loop_depth: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt<'source>>, ParserError<'source>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmt) = self.declaration()? {
                statements.push(stmt);
            }
        }

        Ok(statements)
    }

    pub fn expr(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        self.comma()
    }

    fn declaration(&mut self) -> Result<Option<Stmt<'source>>, ParserError<'source>> {
        let result = if self.matches(&[TokenType::Var]) {
            self.var_declaration()
        } else if self.matches(&[TokenType::Class]) {
            self.class()
        } else if self.matches(&[TokenType::Fn]) {
            let token = self.previous();
            self.function(token.clone())
        } else {
            self.statement()
        };

        match result {
            Ok(stmt) => Ok(Some(stmt)),
            Err(_) => {
                self.synchronize();
                Ok(None)
            }
        }
    }

    fn class(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let class_name = self.consume(TokenType::Identifier, "Expect class name.")?;
        let mut superclass: Option<Rc<expr::Expr<'source>>> = None;
        if self.matches(&[TokenType::Less]) {
            let super_name = self.consume(TokenType::Identifier, "Expect superclass name.")?;
            superclass = Some(Rc::new(expr::Expr::Variable { name: super_name }));
        }
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.}")?;

        let mut methods: Vec<FunctionDecl<'source>> = Vec::new();
        while !self.check(&[TokenType::RightBrace]) && !self.is_at_end() {
            let method_token = Token {
                kind: TokenType::Identifier,
                lexeme: "method",
                literal: None,
                line: self.current_line()
            };
            let method = self.function(method_token)?;
            if let Stmt::Function(func_decl) = method {
                methods.push(func_decl);
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;
        Ok(Stmt::Class { 
            name: class_name,
            superclass,
            methods, 
        })
    }

    fn statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        if self.matches(&[TokenType::For]) {
            self.for_statement()
        } else if self.matches(&[TokenType::If]) {
            self.if_statement()
        } else if self.matches(&[TokenType::Print]) {
            self.print_statement()
        } else if self.matches(&[TokenType::Return]) {
            self.return_statement()
        } else if self.matches(&[TokenType::While]) {
            self.while_statement()
        } else if self.matches(&[TokenType::Break]) {
            self.break_statement()
        } else if self.matches(&[TokenType::LeftBrace]) {
            let block_stmts = self.block()?;
            Ok(Stmt::Block(block_stmts))
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        // 1. Consume the 'for' keyword and expect a left parenthesis
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'.")?;
        let initializer: Option<Stmt<'source>> = if self.matches(&[TokenType::Semicolon]) {
            None
        } else if self.matches(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)  
        };
        
        // 2. Parse the condition and advance to the next token
        let cond = if !self.check(&[TokenType::Semicolon]) {
            Some(self.expr()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "Expected ';' after loop condition.")?;

        // 3. Parse the increment expression
        let increment = if !self.check(&[TokenType::RightParen]) {
            Some(self.expr()?)
        } else {
            None
        };

        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![
                body, 
                Stmt::Expression(Rc::new(inc))
            ]);
        }

        let cond = cond.unwrap_or(expr::Expr::Literal(Literal::True));
        body = Stmt::While {
            condition: Rc::new(cond), 
            body: Box::new(body) 
        };

        if let Some(init) = initializer {
            body = Stmt::Block(vec![
                init,
                body
            ]);
        }

        Ok(body)
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
            condition: Rc::new(cond),
            then_branch: Box::new(then_br), 
            else_branch: else_br, 
        })
    }

    fn print_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let value = self.expr()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(Rc::new(value)))
    }

    fn var_declaration(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let value = self.consume(TokenType::Identifier, "Expected variable name.")?;

        let mut init: Option<Rc<expr::Expr<'source>>> = None;
        if self.matches(&[TokenType::Equal]) {
            init = Some(self.expr()?.into());
        }

        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Var { 
            name: value, 
            initializer: init 
        })
    }

    fn return_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let kw = self.previous().clone();
        let mut val: Option<Rc<expr::Expr<'source>>> = None;

        if !self.check(&[TokenType::Semicolon]) {
            val = Some(self.expr()?.into());
        }

        self.consume(TokenType::Semicolon, "Expect ';' after return statement.")?;

        Ok(Stmt::Return { 
            keyword: kw, 
            value: val
        })
    } 

    fn while_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        self.loop_depth += 1;
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'.")?;
        let cond = self.expr()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition.")?;
        let cond_body = self.statement()?;
        self.loop_depth -= 1;
        Ok(Stmt::While { 
            condition: Rc::new(cond), 
            body: Box::new(cond_body),
        })

    }

    fn break_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        if self.loop_depth == 0 {
            return Err(ParserError::BreakException { line: self.current_line() })
        }
        let kword = self.previous().clone();
        self.consume(TokenType::Semicolon, "Expected ';' after keyword.")?;
        Ok(Stmt::Break { keyword: kword })
    }

    fn expression_statement(&mut self) -> Result<Stmt<'source>, ParserError<'source>> {
        let expression = self.expr()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(Rc::new(expression)))
    }

    fn function(&mut self, kind: Token<'source>) -> Result<Stmt<'source>, ParserError<'source>> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;
        self.consume(TokenType::LeftParen, &format!("Expect '(' after {} name.", kind))?;

        let mut parameters = Vec::new();
        if !self.check(&[TokenType::RightParen]) {
            loop {
                if parameters.len() >= 255 {
                    eprintln!("Can't have more than 255 parameters.");
                }
                parameters.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);

                if !self.check(&[TokenType::RightParen]) {
                    self.consume(TokenType::Comma, "Expect ',' between parameters.")?;
                } else {
                    break;
                }
            } 
        }

        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(TokenType::LeftBrace, &format!("Expect '{{' before {} name.", kind))?;
        let body = self.block()?;

        let decl = FunctionDecl {
            name: Some(name), 
            params: parameters,
            body,
        };
        Ok(Stmt::Function(decl))
    }

    fn block(&mut self) -> Result<Vec<Stmt<'source>>, ParserError<'source>> {
        let mut statements: Vec<Stmt<'source>> = Vec::new();
        while !self.check(&[TokenType::RightBrace]) && !self.is_at_end() {
            if let Some(stmt) = self.declaration()? {
                statements.push(stmt);
            }
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn comma(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut expr = self.assignment()?;
        
        while let Some(token) = self.peek() {
            match token.kind {
                    TokenType::Comma => {
                        self.advance();
                        let operator = self.previous().clone();
                        let right = self.assignment()?;
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

                match expr {
                    expr::Expr::Variable { name } => {
                        return Ok(expr::Expr::Assign { 
                            name, 
                            value: Rc::new(value)
                        });
                    }
                    expr::Expr::Get { object, name } => {
                        return Ok(expr::Expr::Set { 
                            object, 
                            name, 
                            value: Rc::new(value), 
                        });
                    }
                    _ => {
                        let token = self.previous();
                        return Err(ParserError::InvalidAssignmentTarget { 
                            found: token.clone(), 
                            line: token.line, 
                        });
                    }
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
                let true_expr = self.assignment()?;

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
        self.call()
    }

    fn finish_call(&mut self, callee: expr::Expr<'source>) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut arguments: Vec<Rc<expr::Expr<'source>>> = Vec::new();

        if !self.check(&[TokenType::RightParen]) {
            loop {
                if arguments.len() >= 255 {
                    eprintln!("Can't have more than 255 arguments.");
                }
                arguments.push(Rc::new(self.assignment()?));

                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let parentheses = self.consume(TokenType::RightParen, "Expect ')' after arguments.")?;
        Ok(expr::Expr::Call {
            callee: Rc::new(callee), 
            paren: parentheses, 
            args: arguments, 
        })
    } 
    
    fn call(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let mut expr = self.primary()?;

        loop {
            match self.peek().map(|t| t.kind) {
                Some(TokenType::LeftParen) => {
                    self.advance();
                    expr = self.finish_call(expr)?;
                }
                Some(TokenType::Dot) => {
                    self.advance();
                    let name = self.consume(TokenType::Identifier, "Expect property name after '.'.")?;
                    expr = expr::Expr::Get { 
                        object: Rc::new(expr), 
                        name, 
                    }
                }
                Some(TokenType::Increment | TokenType::Decrement) => {
                    let operator = self.advance().clone();
                    expr = expr::Expr::mutate(operator, expr, true); // postfix = true
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<expr::Expr<'source>, ParserError<'source>> {
        let token = self.peek().ok_or_else(|| ParserError::UnexpectedEof {
            expected: "expression".to_string(), 
            line: self.current_line() 
        })?.clone();

        match token.kind {
            TokenType::This => {
                let keyword = self.advance().clone();
                Ok(expr::Expr::This { keyword })
            }
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
            TokenType::Fn => {
                self.advance();
                self.consume(TokenType::LeftParen, "Expect '(' after 'fn'")?;
                
                let mut parameters = Vec::new();
                if !self.check(&[TokenType::RightParen]) {
                    loop {
                        if parameters.len() >= 255 {
                            return Err(ParserError::TooManyParams {
                                line: self.current_line() 
                            });
                        }

                        let param = self.consume(TokenType::Identifier, "Expect paramater name.")?;
                        parameters.push(param.clone());

                        if !self.matches(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;
                self.consume(TokenType::LeftBrace, "Expect '{' before lambda body")?;
                let body_block = self.block()?;
                Ok(expr::Expr::Lambda { 
                    params: parameters, 
                    body: body_block  
                })
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
                    | TokenType::Fn
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
