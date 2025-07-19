// resolve.rs
// author: akrm al-hakimi
// // Module for resolving variable and function declarations in the rlox interpreter

use std::{collections::HashMap};


use crate::{
    ast::stmt::{
        FunctionDecl,
        Stmt,
    },
    error::CompilerError, 
    interpreter::Interpreter
};
use crate::ast::expr::Expr;
use crate::token::Token;
use by_address::ByAddress;

pub struct Resolver<'source> {
    interpreter: &'source mut Interpreter<'source>,
    scopes: Vec<HashMap<String, bool>>,
    errors: Vec<CompilerError<'source>>
}

impl <'source> Resolver <'source> {

    pub fn new(interpreter: &'source mut Interpreter<'source>, scopes: Vec<HashMap<String, bool>>, errors: Vec<CompilerError<'source>>) -> Self {
        Self {
            interpreter,
            scopes,
            errors,
        }
    }

    pub fn resolve_stmts(&mut self, stmts: &[Stmt<'source>]) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt<'source>) {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve_stmts(stmts);
                self.end_scope();
            },
            Stmt::Var{name, initializer} => {
                self.declare(&name.clone());
                if let Some(expr) = initializer {
                    self.resolve_expr(expr);
                }
                self.define(&name.clone());
            },
            Stmt::Function(stmt) => {
                if let Some(name) = &stmt.name {
                    self.declare(name);
                    self.define(name);
                }
                self.resolve_function(stmt);
            },
            Stmt::Expression(expr) => {
                self.resolve_expr(expr);
            },
            Stmt::If { condition, then_branch, else_branch } => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);
                if let Some(else_branch_stmt) = else_branch {
                    self.resolve_stmt(else_branch_stmt);
                }
            },
            Stmt::Print(expr) => {
                self.resolve_expr(expr);
            },
            Stmt::Return { keyword: _, value } => {
                if let Some(value) = value {
                    self.resolve_expr(value);
                }
            },
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(&body);
            },
            _ => {},
        }
    }

    fn resolve_expr(&mut self, expr: &Expr<'source>) {
        match expr {
            Expr::Variable { name } => {
                if !self.scopes.is_empty() {
                    if let Some(scope) = self.scopes.last() {
                        if let Some(false) = scope.get(name.lexeme) {
                            self.errors.push(CompilerError::LocalVarDecl { name: name.clone() });
                        }
                    }
                }
            },
            Expr::Assign { name, value} => {
                self.resolve_expr(value);
                self.resolve_local(expr.clone(), name);
            },
            Expr::Binary { left, operator: _, right } => {
                self.resolve_expr(&left);
                self.resolve_expr(&right);
            },
            Expr::Call { callee, paren: _, args } => {
                self.resolve_expr(&callee);
                for argument in args {
                    self.resolve_expr(argument);
                }
            },
            Expr::Grouping(expr) => {
                self.resolve_expr(expr);
            },
            Expr::Literal(_) => {},
            Expr::Logical { left, operator: _, right } => {
                self.resolve_expr(&left);
                self.resolve_expr(&right);
            },
            Expr::Unary { operator: _, right } => {
                self.resolve_expr(&right);
            },
           _ => unimplemented!()
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }
        if let Some(scope) = self.scopes.last_mut(){
            scope.insert(name.lexeme.to_string(), false);
        } 
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.to_string(), true);
        }
    }

    fn resolve_local(&mut self, expr: Expr<'source>, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(name.lexeme) {
                let depth = self.scopes.len() - 1 - i;
                self.interpreter.resolve(expr, depth);
                return;
            }
        }
    }

    fn resolve_function(&mut self, function: &FunctionDecl<'source> ) {
        self.begin_scope();

        for param in &function.params {
            self.declare(param);
            self.define(param);
        }

        self.resolve_stmts(&function.body);
        self.end_scope();
    }
}
