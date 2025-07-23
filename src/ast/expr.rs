// expr.rs
// Defines the expression AST nodes for the rlox interpreter using Rust enums and Rc for recursion.

use crate::{
    ast::stmt::Stmt,
    token::{Literal, Token},
};
use std::{fmt, rc::Rc};

#[derive(Debug, Clone)]
pub enum Expr<'source> {
    Assign {
        name: Token<'source>,
        value: Rc<Expr<'source>>,
    },
    Binary {
        left: Rc<Expr<'source>>,
        operator: Token<'source>,
        right: Rc<Expr<'source>>,
    },
    Call {
        callee: Rc<Expr<'source>>,
        paren: Token<'source>,
        args: Vec<Rc<Expr<'source>>>,
    },
    Unary {
        operator: Token<'source>,
        right: Rc<Expr<'source>>,
    },
    Mutate {
        operator: Token<'source>,
        operand: Rc<Expr<'source>>,
        postfix: bool,
    },
    Variable {
        name: Token<'source>,
    },
    Ternary {
        condition: Rc<Expr<'source>>,
        true_expr: Rc<Expr<'source>>,
        false_expr: Rc<Expr<'source>>,
    },
    Set {
        object: Rc<Expr<'source>>,
        name: Token<'source>,
        value: Rc<Expr<'source>>,
    },
    Super {
        keyword: Token<'source>,
        method: Token<'source>,
    },
    This {
        keyword: Token<'source>,
    },
    Logical {
        left: Rc<Expr<'source>>,
        operator: Token<'source>,
        right: Rc<Expr<'source>>,
    },
    Lambda {
        params: Vec<Token<'source>>,
        body: Vec<Stmt<'source>>,
    },
    Literal(Literal),
    Get {
        object: Rc<Expr<'source>>,
        name: Token<'source>,
    },
    Grouping(Rc<Expr<'source>>),
}

impl<'source> Expr<'source> {
    pub fn assignment(val_name: Token<'source>, value: Expr<'source>) -> Self {
        Self::Assign {
            name: val_name,
            value: Rc::new(value),
        }
    }

    pub fn binary(left: Expr<'source>, op: Token<'source>, right: Expr<'source>) -> Self {
        Self::Binary {
            left: Rc::new(left),
            operator: op,
            right: Rc::new(right),
        }
    }

    pub fn call(
        callee: Expr<'source>,
        parentheses: Token<'source>,
        arguments: Vec<Rc<Expr<'source>>>,
    ) -> Self {
        Self::Call {
            callee: Rc::new(callee),
            paren: parentheses,
            args: arguments,
        }
    }

    pub fn unary(op: Token<'source>, right: Expr<'source>) -> Self {
        Self::Unary {
            operator: op,
            right: Rc::new(right),
        }
    }

    pub fn mutate(opt: Token<'source>, op: Expr<'source>, pfix: bool) -> Self {
        Self::Mutate {
            operator: opt,
            operand: Rc::new(op),
            postfix: pfix,
        }
    }

    pub fn variable(var_name: Token<'source>) -> Self {
        Self::Variable { name: var_name }
    }

    pub fn ternary(
        cond: Expr<'source>,
        true_expr: Expr<'source>,
        false_expr: Expr<'source>,
    ) -> Self {
        Self::Ternary {
            condition: Rc::new(cond),
            true_expr: Rc::new(true_expr),
            false_expr: Rc::new(false_expr),
        }
    }

    pub fn set(object: Expr<'source>, name: Token<'source>, value: Expr<'source>) -> Self {
        Self::Set {
            object: Rc::new(object),
            name,
            value: Rc::new(value),
        }
    }

    pub fn _super(keyword: Token<'source>, method: Token<'source>) -> Self {
        Self::Super { 
            keyword,
            method, 
        }
    }

    pub fn this(keyword: Token<'source>) -> Self {
        Self::This { keyword }
    }

    pub fn logical(left: Expr<'source>, op: Token<'source>, right: Expr<'source>) -> Self {
        Self::Logical {
            left: Rc::new(left),
            operator: op,
            right: Rc::new(right),
        }
    }

    pub fn literal(val: Literal) -> Self {
        Expr::Literal(val)
    }

    pub fn get(object: Expr<'source>, name: Token<'source>) -> Self {
        Self::Get {
            object: Rc::new(object),
            name,
        }
    }

    pub fn grouping(expr: Expr<'source>) -> Self {
        Expr::Grouping(Rc::new(expr))
    }

    pub fn lambda(paramaters: Vec<Token<'source>>, bod: Vec<Stmt<'source>>) -> Self {
        Self::Lambda {
            params: paramaters,
            body: bod,
        }
    }
}

// Our pretty printer
// This avoids the vistor pattern of having to pass a mutable reference to the printer
// and allows us to use the `Display` trait for printing expressions
impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Assign { name, value } => {
                write!(f, "({} {})", name, value)
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", operator.lexeme, left, right)
            }
            Expr::Call {
                callee,
                paren,
                args,
            } => {
                write!(f, "({} {} {:?})", callee, paren, args)
            }
            Expr::Unary { operator, right } => {
                write!(f, "({} {})", operator.lexeme, right)
            }
            Expr::Mutate {
                operator,
                operand,
                postfix,
            } => {
                write!(f, "({} {} {})", operator.lexeme, operand, postfix)
            }
            Expr::Variable { name } => {
                write!(f, "({})", name.lexeme)
            }
            Expr::Ternary {
                condition,
                true_expr,
                false_expr,
            } => {
                write!(f, "({} ? {} : {})", condition, true_expr, false_expr)
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                write!(f, "({}.{} = {})", object, name, value)
            }
            Expr::Super { keyword, method } => {
                write!(f, "({} {})", keyword, method)
            }
            Expr::This { keyword } => {
                write!(f, "({})", keyword)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", left, operator, right)
            }
            Expr::Literal(lit) => write!(f, "{:#?}", lit),
            Expr::Get { object, name } => {
                write!(f, "({}.{})", object, name)
            }
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
            Expr::Lambda { params, body } => {
                let param_names: Vec<&str> = params.iter().map(|p| p.lexeme).collect();
                write!(f, "(lambda [{}] {:?})", param_names.join(", "), body)
            }
        }
    }
}