// interpreter.rs
// author: akrm al-hakimi
// our interpreter


use core::fmt;

use crate::{ast::{expr::Expr, stmt::Stmt}, environment::{env::SharedEnv, env::Environment}, token::token::Literal};
use crate::token::token::{Token, TokenType};
use crate::error::error::RuntimeError;

pub struct Interpreter<'source> {
    environment: SharedEnv<'source>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

#[allow(clippy::needless_lifetimes)]
impl<'source> Default for Interpreter<'source> {
    fn default() -> Self {
        Self::new()
    }
}

impl <'source>Interpreter <'source> {

    pub fn new() -> Self {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt<'source>>) -> Result<(), RuntimeError> {
        for statement in statements {
            self.execute(statement)?
        }  
        Ok(())
    }

    pub fn evaluate(&mut self, expr: &Expr<'source>) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => self.evaluate_literal(lit),
            Expr::Unary { operator, right } => { self.evaluate_unary(operator, right) }
            Expr::Assign { name, value } => { self.evaluate_assignment(name.clone(), (**value).clone()) }

            Expr::Binary {
                left,
                operator,
                right,
            } => { self.evaluate_binary(left, operator, right) }
            Expr::Variable { 
                name,
            } => { self.environment.borrow().get(name )}
            Expr::Grouping(inner) => self.evaluate(inner),
            Expr::Ternary {
                condition,
                true_expr,
                false_expr,
            } => { self.evaluate_ternary(condition, true_expr, false_expr) }
        }
    }

    fn execute(&mut self, stmt: Stmt<'source>) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Block(statements) => {
                let new_env = Environment::from_enclosing(self.environment.clone());
                self.execute_block(&statements, new_env)
            }
            Stmt::Expression(expr) => {
                self.evaluate(&expr)?;
                Ok(())
            },
            Stmt::Print(expr) => {
                let value = self.evaluate(&expr)?;
                println!("{}", value);
                Ok(())
            },
            // In jlox, you can define unitialized variables but if you use them they'll just be nil
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.evaluate(&expr)?,
                    None => Value::Nil,
                };
                self.environment.borrow_mut().define(name.lexeme.to_string(), value);
                Ok(())
            }
            _ => unimplemented!()
        }
    }

    fn execute_block(&mut self, statements: &[Stmt<'source>], new_env: SharedEnv<'source>) -> Result<(), RuntimeError> {
        let previous = self.environment.clone();
        self.environment = new_env;

        let result = statements.iter().try_for_each(|stmt| self.execute(stmt.clone()));

        self.environment = previous;
        result
    } 

    fn evaluate_block_statement(&mut self, stmt: &[Stmt<'source>], new_env: SharedEnv<'source>) -> Result<Value, RuntimeError> {
        self.execute_block(stmt, new_env)?;
        Ok(Value::Nil)
    }

    fn evaluate_var_decl(&mut self, name: Token<'source>, initializer: Option<Expr<'source>>) -> Result<Value, RuntimeError> {
        let value = match initializer {
            Some(expr) => self.evaluate(&expr)?,
            None => Value::Nil,
        };

        self.environment.borrow_mut().define(name.to_string(), value);
        Ok(Value::Nil)
    } 

    fn evaluate_assignment(&mut self, name: Token<'source>, value: Expr<'source>) -> Result<Value, RuntimeError> {

        let value = self.evaluate(&value)?;

        self.environment.borrow_mut().assign(name, &value)?;
        Ok(value)
    } 

    fn evaluate_literal(&self, lit: &Literal) -> Result<Value, RuntimeError> {
        match lit {
            Literal::Num(n) => Ok(Value::Number(*n)),
            Literal::Str(s) => Ok(Value::String(s.clone())),
            Literal::True => Ok(Value::Bool(true)),
            Literal::False => Ok(Value::Bool(false)),
            Literal::Nil => Ok(Value::Nil),
        }
    }

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr<'source>) -> Result<Value, RuntimeError> {
        let right_val = self.evaluate(right)?;

        match operator.kind {
            TokenType::Minus => match right_val {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(RuntimeError::UnaryMinus { lexeme: operator.lexeme.to_string(), line: operator.line }),
            },
            TokenType::Bang => Ok(Value::Bool(!self.is_truthy(&right_val))),
            _ => unreachable!("Unknown unary operator"),
        }
    }

    fn evaluate_binary (&mut self, left: &Expr<'source>, operator: &Token, right: &Expr<'source>) -> Result<Value, RuntimeError> {
        let left_val = self.evaluate(left)?;
        let right_val = self.evaluate(right)?;

        let lexeme = operator.lexeme.to_string();
        let line = operator.line;


        match operator.kind {
            TokenType::Plus => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                _ => Err(RuntimeError::BinaryPlus { lexeme, line })
            },
            TokenType::Minus => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                _ => Err(RuntimeError::BinaryMinus { lexeme, line })
            },
            TokenType::Star => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                _ => Err(RuntimeError::BinaryMult { lexeme, line })
            },
            TokenType::Slash => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => {
                    if r == 0.0 {
                        return Err(RuntimeError::BinaryDBZ { line })
                    }
                    Ok(Value::Number(l / r))
                }
                _ => Err(RuntimeError::BinaryDiv { lexeme, line })
            },
            TokenType::EqualEqual => { Ok(Value::Bool(left_val == right_val)) }
            TokenType::Greater =>  match(left_val, right_val) { 
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l > r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::Less =>  match(left_val, right_val) { 
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::GreaterEqual =>  match(left_val, right_val) { 
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l >= r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::LessEqual =>  match(left_val, right_val) { 
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l <= r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            }
            TokenType::BangEqual => Ok(Value::Bool(left_val != right_val)),
            _ => unreachable!("Unknown binary operator"),
        }
    }

    fn evaluate_ternary(&mut self, condition: &Expr<'source>, true_expr: &Expr<'source>, false_expr: &Expr<'source>) -> Result<Value, RuntimeError> {
        let condition_val = self.evaluate(condition)?;
        
        if self.is_truthy(&condition_val) {
            self.evaluate(true_expr) 
        } else {
            self.evaluate(false_expr)
        }

    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}
