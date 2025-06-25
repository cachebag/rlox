// interpreter.rs
// author: akrm al-hakimi
// our interpreter


use crate::{ast::expr::Expr, token::Literal};
use crate::token::{Token, TokenType};
use crate::error::RuntimeError;

pub struct Interpreter {}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl Interpreter {

    pub fn new() -> Self {
        Interpreter {}
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => self.evaluate_literal(lit),
            Expr::Unary { operator, right } => { self.evaluate_unary(operator, right) }
            Expr::Binary {
                left,
                operator,
                right,
            } => { self.evaluate_binary(left, operator, right) }
            Expr::Grouping(inner) => self.evaluate(inner),
            Expr::Ternary {
                condition,
                true_expr,
                false_expr,
            } => { unimplemented!() }
        }
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

    fn evaluate_unary(&mut self, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
        let right_val = self.evaluate(right)?;

        match operator.kind {
            TokenType::Minus => match right_val {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(RuntimeError::UnaryMinus { lexeme: operator.lexeme.to_string(), line: operator.line }),
            },
            TokenType::Bang => match right_val {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(RuntimeError::UnaryNot { lexeme: operator.lexeme.to_string(), line: operator.line }),
            },
            _ => unreachable!("Uknown unary operator"),
        }
    }

    fn evaluate_binary (&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value, RuntimeError> {
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

    fn evaluate_ternary(&mut self, condition: &Expr, true_expr: &Expr, false_expr: &Expr) -> Result<Value, RuntimeError> {
        todo!()
    }
}
