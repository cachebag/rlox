// stmt.rs 
// author: akrm al-hakimi
// This module defines the statement AST for the rlox interpreter

use crate::ast::expr::Expr;
use crate::token::Token;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct FunctionDecl<'source> {
    pub name: Option<Token<'source>>,
    pub params: Vec<Token<'source>>,
    pub body: Vec<Stmt<'source>>,
}

#[derive(Debug, Clone)]
pub enum Stmt<'source> {
    Block(Vec<Stmt<'source>>),    
    Class {    
        name: Token<'source>,
        methods: Vec<FunctionDecl<'source>>,
    },
    Expression(Rc<Expr<'source>>),
    Function(FunctionDecl<'source>),
    If {
        condition: Rc<Expr<'source>>,
        then_branch: Box<Stmt<'source>>,
        else_branch: Option<Box<Stmt<'source>>>,
    },
    Print(Rc<Expr<'source>>),
    Return {
        keyword: Token<'source>,
        value: Option<Rc<Expr<'source>>>,
    },
    Var {
        name: Token<'source>,
        initializer: Option<Rc<Expr<'source>>>,
    },
    While {
        condition: Rc<Expr<'source>>,
        body: Box<Stmt<'source>>,
    },
    Break {
        keyword: Token<'source>,
    },
}
