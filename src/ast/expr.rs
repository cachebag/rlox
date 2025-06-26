// expr.rs 
// author: akrm al-hakimi
// This module defines the expression AST for the rlox interpreter
// Design notes:
//             - In Rust, we can use enums to represent the different types of expressions as
//             opposed to Java's class hierarchy.
//             - Box<T> is used to allow for recursive types, as Rust does not allow direct
//             recursion in types. This is still simpler than Java's visitor pattern because 
//             we can use the Display trait to traverse and print expressions directly.


use crate::token::token::{Token, Literal};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Expr<'source> {
    Binary {
        left: Box<Expr<'source>>,
        operator: Token<'source>,
        right: Box<Expr<'source>>,
    },
    Unary {
        operator: Token<'source>,
        right: Box<Expr<'source>>,
    },
    Variable {
        name: Token<'source>,
    },
    Ternary {
        condition: Box<Expr<'source>>,
        true_expr: Box<Expr<'source>>,
        false_expr: Box<Expr<'source>>,
    },
    Literal(Literal),
    Grouping(Box<Expr<'source>>),
}

impl <'source> Expr<'source> {
    
    pub fn binary(left: Expr<'source>, op: Token<'source>, right: Expr<'source>) -> Self {
        Self::Binary { 
            left: Box::new(left),
            operator: op,
            right:Box::new(right),
        }
    }

    pub fn unary(op: Token<'source>, right: Expr<'source>) -> Self {
        Self::Unary { 
            operator: op,
            right: Box::new(right),
        }
    }

    pub fn variable(var_name: Token<'source>) -> Self {
        Self::Variable {
            name: var_name,
        }
    }

    pub fn ternary(cond: Expr<'source>, true_expr: Expr<'source>, false_expr: Expr<'source>) -> Self {
        Self::Ternary {
            condition: Box::new(cond),
            true_expr: Box::new(true_expr),
            false_expr: Box::new(false_expr),
        }
    }

    pub fn literal(val: Literal) -> Self {
        Expr::Literal(val)
    }

    pub fn grouping(expr: Expr<'source>) -> Self {
        Expr::Grouping(Box::new(expr))
    }

} 

// Our pretty printer
// This avoids the vistor pattern of having to pass a mutable reference to the printer
// and allows us to use the `Display` trait for printing expressions
impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary { left, operator, right } => {
                write!(f, "({} {} {})", operator.lexeme, left, right)
            }
            Expr::Unary { operator, right } => {
                write!(f, "({} {})", operator.lexeme, right)
            }
            Expr::Variable { name} => {
                write!(f, "({})", name.lexeme)
            }
            Expr::Ternary { condition, true_expr, false_expr } => {
                write!(f, "({} ? {} : {})", condition, true_expr, false_expr)
            }
            Expr::Literal(lit) => write!(f, "{:#?}", lit),
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
        }
    }
}
