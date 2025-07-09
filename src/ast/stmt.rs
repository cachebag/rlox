// stmt.rs 
// author: akrm al-hakimi
// This module defines the statement AST for the rlox interpreter

use crate::ast::expr::Expr;
use crate::token::token::Token;

#[derive(Debug, Clone)]
pub struct FunctionDecl<'source> {
    pub name: Token<'source>,
    pub params: Vec<Token<'source>>,
    pub body: Vec<Stmt<'source>>,
}

#[derive(Debug, Clone)]
pub enum Stmt<'source> {
    Class {
        name: Token<'source>,
        superclass: Option<Expr<'source>>,
        methods: Vec<FunctionDecl<'source>>,
    },
    Block(Vec<Stmt<'source>>),
    Expression(Expr<'source>),
    Function(FunctionDecl<'source>),
    If {
        condition: Expr<'source>,
        then_branch: Box<Stmt<'source>>,
        else_branch: Option<Box<Stmt<'source>>>,
    },
    Print(Expr<'source>),
    Return {
        keyword: Token<'source>,
        value: Option<Expr<'source>>,
    },
    Var {
        name: Token<'source>,
        initializer: Option<Expr<'source>>,
    },
    While {
        condition: Expr<'source>,
        body: Box<Stmt<'source>>,
    },
    Break {
        keyword: Token<'source>,
    },
}
