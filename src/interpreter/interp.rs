// interpreter.rs
// author: akrm al-hakimi
// our interpreter

use crate::{ast::stmt::FunctionDecl, token::token::{Token, TokenType}};
use crate::{
    ast::{expr::Expr, stmt::Stmt},
    environment::env::{Environment, SharedEnv},
    function::Function,
    token::token::Literal,
};
use crate::{
    callable::{Callable, Clock},
    error::RuntimeError,
};
use core::fmt;
use std::rc::Rc;

pub struct Interpreter<'source> {
    pub globals: SharedEnv<'source>,
    pub environment: SharedEnv<'source>,
}

#[derive(Debug, Clone)]
pub enum Value<'source> {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
    Callable(Rc<dyn Callable<'source> + 'source>),
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (String(a), String(b)) => a == b,
            (Number(a), Number(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Nil, Nil) => true,
            // Callable values are never equal
            (Callable(_), Callable(_)) => false,
            _ => false,
        }
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'source> Default for Interpreter<'source> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'source> Interpreter<'source> {
    pub fn new() -> Self {
        let globals = Environment::new();
        globals
            .borrow_mut()
            .define("clock".to_string(), Value::Callable(Rc::new(Clock)));

        Interpreter {
            globals: globals.clone(),
            environment: globals,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt<'source>>) -> Result<(), RuntimeError<'source>> {
        for statement in statements {
            self.execute(statement)?
        }
        Ok(())
    }

    pub fn evaluate(&mut self, expr: Expr<'source>) -> Result<Value<'source>, RuntimeError<'source>> {
        match expr {
            Expr::Literal(lit) => self.evaluate_literal(lit),
            Expr::Unary { operator, right } => self.evaluate_unary(operator, right.as_ref()),
            Expr::Mutate {
                operator,
                operand,
                postfix,
            } => self.evaluate_mutation(operator.clone(), operand, postfix),
            Expr::Call {
                callee,
                paren,
                args,
            } => self.evaluate_call(*callee, paren, args),
            Expr::Assign { name, value } => self.evaluate_assignment(name.clone(), *value),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(*left, &operator, *right),
            Expr::Variable { name } => self.environment.borrow().get(&name),
            Expr::Grouping(inner) => self.evaluate(*inner),
            Expr::Ternary {
                condition,
                true_expr,
                false_expr,
            } => self.evaluate_ternary(*condition, *true_expr, *false_expr),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.evaluate_logical(*left, &operator, *right),
        }
    }

    fn evaluate_function(&mut self, decl: FunctionDecl<'source>) -> Result<(), RuntimeError<'source>> {
        let function = Function {
            declaration: decl.clone(),
            closure: self.environment.clone(),
        };
        self.environment
            .borrow_mut()
            .define(
                decl.name.lexeme.to_string(),
                Value::Callable(Rc::new(function)),
            );
            Ok(())
    }

    pub fn execute(&mut self, stmt: Stmt<'source>) -> Result<(), RuntimeError<'source>> {
        match stmt {
            Stmt::Block(statements) => {
                let new_env = Environment::from_enclosing(self.environment.clone());
                self.execute_block(&statements, new_env)?;
                Ok(())
            }
            Stmt::Expression(expr) => {
                self.evaluate(expr)?;
                Ok(())
            }
            Stmt::Function(decl) => self.evaluate_function(decl),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.evaluate_if_statement(condition, *then_branch, else_branch.map(|b| *b))?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{}", value);
                Ok(())
            }
            Stmt::Return { keyword: _, value } => {
                let result = match value {
                    Some(expr) => self.evaluate(expr)?,
                    None => Value::Nil,
                };
                Err(RuntimeError::ReturnException(result))
            }
            Stmt::While { condition, body } => {
                self.evaluate_while(condition, *body)?;
                Ok(())
            }
            Stmt::Break { keyword: _ } => {
                self.evaluate_break()?;
                Ok(())
            }
            // In jlox, you can define unitialized variables but if you use them they'll just be nil
            Stmt::Var { name, initializer } => {
                self.evaluate_var_decl(name, initializer)?;
                Ok(())
            }
            _ => unimplemented!(),
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt<'source>],
        new_env: SharedEnv<'source>,
    ) -> Result<(), RuntimeError<'source>> {
        let previous = self.environment.clone();
        self.environment = new_env;

        let result = statements
            .iter()
            .try_for_each(|stmt| self.execute(stmt.clone()));

        self.environment = previous;
        result
    }

    fn evaluate_block_statement(
        &mut self,
        stmt: &[Stmt<'source>],
        new_env: SharedEnv<'source>,
    ) -> Result<Value, RuntimeError<'source>> {
        self.execute_block(stmt, new_env)?;
        Ok(Value::Nil)
    }

    fn evaluate_var_decl(
        &mut self,
        name: Token,
        initializer: Option<Expr<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let value = match initializer {
            Some(expr) => self.evaluate(expr)?,
            None => Value::Nil,
        };

        self.environment
            .borrow_mut()
            .define(name.lexeme.to_string(), value);
        Ok(Value::Nil)
    }

    fn evaluate_while(
        &mut self,
        cond: Expr<'source>,
        body: Stmt<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        while {
            let cond_val = self.evaluate(cond.clone())?;
            self.is_truthy(&cond_val)
        } {
            match self.execute(body.clone()) {
                Err(RuntimeError::BreakException) => break,
                Err(e) => return Err(e),
                _ => {}
            }
        }
        Ok(Value::Nil)
    }

    fn evaluate_break(&mut self) -> Result<(), RuntimeError<'source>> {
        Err(RuntimeError::BreakException)
    }

    fn evaluate_if_statement(
        &mut self,
        cond: Expr<'source>,
        then_b: Stmt<'source>,
        else_b: Option<Stmt<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let condition_val = self.evaluate(cond)?;

        if self.is_truthy(&condition_val) {
            self.evaluate_block_statement(
                &[then_b],
                Environment::from_enclosing(self.environment.clone()),
            )?;
        } else if let Some(else_stmt) = else_b {
            self.execute(else_stmt)?;
        }

        Ok(Value::Nil)
    }

    fn evaluate_assignment(
        &mut self,
        name: Token,
        value: Expr<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let value = self.evaluate(value)?;

        self.environment.borrow_mut().assign(name, &value)?;
        Ok(value)
    }

    fn evaluate_literal(&self, lit: Literal) -> Result<Value<'source>, RuntimeError<'source>> {
        match lit {
            Literal::Num(n) => Ok(Value::Number(n)),
            Literal::Str(s) => Ok(Value::String(s.clone())),
            Literal::True => Ok(Value::Bool(true)),
            Literal::False => Ok(Value::Bool(false)),
            Literal::Nil => Ok(Value::Nil),
        }
    }

    fn evaluate_logical(
        &mut self,
        lhs: Expr<'source>,
        operator: &Token,
        rhs: Expr<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let left = self.evaluate(lhs)?;
        match operator.kind {
            TokenType::Or => {
                if self.is_truthy(&left) {
                    Ok(left)
                } else {
                    self.evaluate(rhs)
                }
            }
            TokenType::And => {
                if !self.is_truthy(&left) {
                    Ok(left)
                } else {
                    self.evaluate(rhs)
                }
            }
            _ => unreachable!("Unknown logical operator."),
        }
    }
    fn evaluate_unary(
        &mut self,
        operator: Token,
        right: &Expr<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let right_val = self.evaluate(right.clone())?;

        match operator.kind {
            TokenType::Minus => match right_val {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(RuntimeError::UnaryMinus {
                    lexeme: operator.lexeme.to_string(),
                    line: operator.line,
                }),
            },
            TokenType::Bang => Ok(Value::Bool(!self.is_truthy(&right_val))),
            _ => unreachable!("Unknown unary operator"),
        }
    }

    fn evaluate_mutation(
        &mut self,
        operator: Token,
        operand: Box<Expr<'source>>,
        postfix: bool,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let name = match operand.as_ref() {
            Expr::Variable { name } => name,
            _ => {
                return Err(RuntimeError::MutationError {
                    lexeme: operator.lexeme.to_string(),
                    line: operator.line,
                });
            }
        };

        let current_value = self.environment.borrow().get(name)?;

        match current_value {
            Value::Number(n) => match operator.kind {
                TokenType::Increment => {
                    let new_val = n + 1.0;
                    self.environment
                        .borrow_mut()
                        .assign(name.clone(), &Value::Number(new_val))?;
                    if postfix {
                        Ok(Value::Number(n))
                    } else {
                        Ok(Value::Number(new_val))
                    }
                }
                TokenType::Decrement => {
                    let new_val = n - 1.0;
                    self.environment
                        .borrow_mut()
                        .assign(name.clone(), &Value::Number(new_val))?;
                    if postfix {
                        Ok(Value::Number(n))
                    } else {
                        Ok(Value::Number(new_val))
                    }
                }
                _ => unreachable!("Illegal mutation."),
            },
            _ => Err(RuntimeError::MutationError {
                lexeme: operator.to_string(),
                line: operator.line,
            }),
        }
    }

    fn evaluate_binary(
        &mut self,
        left: Expr<'source>,
        operator: &Token,
        right: Expr<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let left_val = self.evaluate(left.clone())?;
        let right_val = self.evaluate(right.clone())?;
        let lexeme = operator.lexeme.to_string();
        let line = operator.line;

        match operator.kind {
            TokenType::Comma => {
                self.evaluate(left)?;
                self.evaluate(right)
            }
            TokenType::Plus => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                _ => Err(RuntimeError::BinaryPlus { lexeme, line }),
            },
            TokenType::Minus => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                _ => Err(RuntimeError::BinaryMinus { lexeme, line }),
            },
            TokenType::Star => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                _ => Err(RuntimeError::BinaryMult { lexeme, line }),
            },
            TokenType::Slash => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => {
                    if r == 0.0 {
                        return Err(RuntimeError::BinaryDBZ { line });
                    }
                    Ok(Value::Number(l / r))
                }
                _ => Err(RuntimeError::BinaryDiv { lexeme, line }),
            },
            TokenType::EqualEqual => Ok(Value::Bool(left_val == right_val)),
            TokenType::Greater => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l > r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::Less => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::GreaterEqual => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l >= r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::LessEqual => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l <= r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::BangEqual => Ok(Value::Bool(left_val != right_val)),
            _ => unreachable!("Unknown binary operator"),
        }
    }

    fn evaluate_call(
        &mut self,
        callee: Expr<'source>,
        paren: Token<'source>,
        args: Vec<Expr<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let callee = self.evaluate(callee)?;
        let mut arguments: Vec<Value<'source>> = Vec::new();

        for argument in args {
            arguments.push(self.evaluate(argument)?);
        }

        let function = match callee {
            Value::Callable(f) => f,
            _ => {
                return Err(RuntimeError::FunctionError {
                    lexeme: paren.to_string(),
                    line: paren.line,
                    message: "Can only call functions and classes.".to_string(),
                });
            }
        };

        if arguments.len() != function.arity() {
            return Err(RuntimeError::FunctionError {
                lexeme: paren.to_string(),
                line: paren.line,
                message: "Ensure your function call matches the function arity.".to_string(),
            });
        }

        function.call(self, arguments)

    }

    fn evaluate_ternary(
        &mut self,
        condition: Expr<'source>,
        true_expr: Expr<'source>,
        false_expr: Expr<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let condition_val = self.evaluate(condition)?;

        if self.is_truthy(&condition_val) {
            self.evaluate(true_expr)
        } else {
            self.evaluate(false_expr)
        }
    }

    fn is_truthy(&self, value: &Value<'source>) -> bool {
        match value {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Callable(c) => write!(f, "{:?}", c),
        }
    }
}
