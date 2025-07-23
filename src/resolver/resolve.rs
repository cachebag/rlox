// resolve.rs
// Implements variable and function resolution for rlox, managing scopes and resolving identifiers.

use std::collections::HashMap;

use crate::ast::expr::Expr;
use crate::token::Token;
use crate::{
    ast::stmt::{FunctionDecl, Stmt},
    error::CompilerError,
    interpreter::Interpreter,
};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ClassType {
    None,
    Class,
    SubClass,
}

pub struct Resolver<'source> {
    scopes: Vec<HashMap<String, bool>>,
    errors: Vec<CompilerError<'source>>,
    current_function: FunctionType,
    current_class: ClassType,
}

#[allow(clippy::needless_lifetimes)]
impl<'source> Default for Resolver<'source> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'source> Resolver<'source> {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            errors: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
        }
    }

    pub fn resolve_stmts(
        &mut self,
        stmts: &[Stmt<'source>],
        interpreter: &mut Interpreter<'source>,
    ) {
        for stmt in stmts {
            self.resolve_stmt(stmt, interpreter);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt<'source>, interpreter: &mut Interpreter<'source>) {
        match stmt {
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve_stmts(stmts, interpreter);
                self.end_scope();
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(expr) = initializer {
                    self.resolve_expr(expr, interpreter);
                }
                self.define(name);
            }
            Stmt::Function(func) => {
                if let Some(name) = &func.name {
                    self.declare(name);
                    self.define(name);
                }
                self.resolve_function(func, interpreter, FunctionType::Function);
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                let enclosing_class = self.current_class;
                self.current_class = ClassType::Class;

                self.declare(name);
                self.define(name);

                if let Some(super_expr) = &superclass {
                    if let Expr::Variable { name: super_name } = &**super_expr {
                        if super_name.lexeme == name.lexeme {
                            self.errors.push(CompilerError::SelfInheritance {
                                line: super_name.line,
                            });
                        }
                    }
                }

                if let Some(superclass_expr) = &superclass {
                    self.current_class = ClassType::SubClass;
                    self.resolve_expr(superclass_expr, interpreter);
                }

                if let Some(_superclass) = &superclass {
                    self.begin_scope();
                    if let Some(scope) = self.scopes.last_mut() {
                        scope.insert("super".to_string(), true);
                    };
                }

                self.begin_scope();
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert("this".to_string(), true);

                for method in methods {
                    let declaration =
                        if method.name.as_ref().map(|name| name.lexeme) == Some("init") {
                            FunctionType::Initializer
                        } else {
                            FunctionType::Method
                        };
                    self.resolve_function(method, interpreter, declaration);
                }
                self.end_scope();

                if let Some(_superclass) = &superclass {
                    self.end_scope();
                }

                self.current_class = enclosing_class;
            }
            Stmt::Expression(expr) => self.resolve_expr(expr, interpreter),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition, interpreter);
                self.resolve_stmt(then_branch, interpreter);
                if let Some(else_branch_stmt) = else_branch {
                    self.resolve_stmt(else_branch_stmt, interpreter);
                }
            }
            Stmt::Print(expr) => self.resolve_expr(expr, interpreter),
            Stmt::Return { keyword, value } => {
                if self.current_function == FunctionType::None {
                    self.errors.push(CompilerError::IllegalReturn {
                        keyword: keyword.clone(),
                    });
                }
                if self.current_function == FunctionType::Initializer && value.is_some() {
                    self.errors.push(CompilerError::InitializerReturn {
                        keyword: keyword.clone(),
                    });
                }
                if let Some(value) = value {
                    self.resolve_expr(value, interpreter);
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition, interpreter);
                self.resolve_stmt(body, interpreter);
            }
            _ => {}
        }
    }

    fn resolve_expr(&mut self, expr: &Rc<Expr<'source>>, interpreter: &mut Interpreter<'source>) {
        match &**expr {
            Expr::Variable { name } => {
                if !self.scopes.is_empty() {
                    if let Some(scope) = self.scopes.last() {
                        if let Some(false) = scope.get(name.lexeme) {
                            self.errors
                                .push(CompilerError::LocalVarDecl { name: name.clone() });
                        }
                    }
                }
                self.resolve_local(expr.clone(), name, interpreter);
            }
            Expr::Assign { name, value } => {
                self.resolve_expr(value, interpreter);
                self.resolve_local(expr.clone(), name, interpreter);
            }
            Expr::Binary {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left, interpreter);
                self.resolve_expr(right, interpreter);
            }
            Expr::Ternary {
                condition,
                true_expr,
                false_expr,
            } => {
                self.resolve_expr(condition, interpreter);
                self.resolve_expr(true_expr, interpreter);
                self.resolve_expr(false_expr, interpreter);
            }
            Expr::Call {
                callee,
                paren: _,
                args,
            } => {
                self.resolve_expr(callee, interpreter);
                for arg in args {
                    self.resolve_expr(arg, interpreter);
                }
            }
            Expr::Set {
                object,
                name: _,
                value,
            } => {
                self.resolve_expr(value, interpreter);
                self.resolve_expr(object, interpreter);
            }
            Expr::Super { keyword, method: _ } => {
                if self.current_class == ClassType::None {
                    self.errors.push(CompilerError::SuperTypeError {
                        msg: "Can't use 'super' outside of a class.".to_string(),
                        line: keyword.line,
                    })
                } else if self.current_class != ClassType::SubClass {
                    self.errors.push(CompilerError::SuperTypeError {
                        msg: "Can't use 'super' in a class with no superclass.".to_string(),
                        line: keyword.line,
                    })
                }
                self.resolve_local(expr.clone(), keyword, interpreter);
            }
            Expr::Get { object, name: _ } => {
                self.resolve_expr(object, interpreter);
            }
            Expr::This { keyword } => {
                if self.current_class == ClassType::None {
                    self.errors.push(CompilerError::ThisOutsideClass {
                        keyword: keyword.clone(),
                    })
                }
                self.resolve_local(expr.clone(), keyword, interpreter);
            }
            Expr::Grouping(expr) => self.resolve_expr(expr, interpreter),
            Expr::Logical {
                left,
                operator: _,
                right,
            } => {
                self.resolve_expr(left, interpreter);
                self.resolve_expr(right, interpreter);
            }
            Expr::Unary { operator: _, right } => self.resolve_expr(right, interpreter),
            Expr::Literal(_) => {}
            _ => unimplemented!(),
        }
    }

    fn resolve_local(
        &mut self,
        expr: Rc<Expr<'source>>,
        name: &Token<'source>,
        interpreter: &mut Interpreter<'source>,
    ) {
        for i in (0..self.scopes.len()).rev() {
            if self.scopes[i].contains_key(name.lexeme) {
                let depth = self.scopes.len() - 1 - i;
                interpreter.resolve(expr, depth);
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        function: &FunctionDecl<'source>,
        interpreter: &mut Interpreter<'source>,
        func_type: FunctionType,
    ) {
        let enclosing_func = self.current_function;
        self.current_function = func_type;

        self.begin_scope();
        for param in &function.params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(&function.body, interpreter);
        self.end_scope();
        self.current_function = enclosing_func;
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token<'source>) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name.lexeme) {
                self.errors
                    .push(CompilerError::ExistingVar { line: name.line })
            }
            scope.insert(name.lexeme.to_string(), false);
        }
    }

    fn define(&mut self, name: &Token<'source>) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.to_string(), true);
        }
    }

    pub fn take_errors(self) -> Vec<CompilerError<'source>> {
        self.errors
    }
}