// resolve.rs
// author: akrm al-hakimi
// // Module for resolving variable and function declarations in the rlox interpreter

use std::{collections::HashMap};


use crate::{ast::stmt::FunctionDecl, error::CompilerError, function, interpreter::Interpreter};
use crate::ast::stmt::Stmt;
use crate::ast::expr::Expr;
use crate::token::Token;

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
            }
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
            }
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
