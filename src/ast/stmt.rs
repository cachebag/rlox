use crate::ast::expr::Expr;
use crate::token::token::Token;

#[derive(Debug, Clone)]
pub struct FunctionDecl<'source> {
    name: Token<'source>,
    params: Vec<Token<'source>>,
    body: Vec<Stmt<'source>>,
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
    }
}
