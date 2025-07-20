// func.rs 
// author: akrm al-hakimi
// This module defines the Function type, which represents a function in our interpreter.

use crate::{ast::stmt::FunctionDecl, environment::{Environment, SharedEnv}, interpreter::{Interpreter, Value}};
use std::fmt;
use crate::callable::Callable;
use crate::error::RuntimeError;

pub struct Function<'source> {
    pub declaration: FunctionDecl<'source>,
    pub closure: SharedEnv<'source>,
}

impl <'source> Callable <'source> for Function <'source> {

    fn call(&self, interpreter: &mut Interpreter<'source>, args: Vec<Value<'source>>) -> Result<Value<'source>, RuntimeError<'source>> {
        let env = Environment::from_enclosing(self.closure.clone());
    
        for (param, arg) in self.declaration.params.iter().zip(args.into_iter()) {
            env.borrow_mut().define(param.lexeme.to_string(), arg);
        }
    
        let previous = interpreter.environment.clone();
        interpreter.environment = env.clone();
    
        let result = {
            let mut body_result = Ok(Value::Nil);
            for stmt in &self.declaration.body {
                match interpreter.execute(stmt) {
                    Err(RuntimeError::ReturnException(val)) => {
                        body_result = Ok(val);
                        break;
                    }
                    Err(e) => {
                        body_result = Err(e);
                        break;
                    }
                    Ok(()) => continue,
                }
            }
            body_result
        };
    
        interpreter.environment = previous;
    
        result
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

}

impl fmt::Debug for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.as_ref().map(|t| t.lexeme).unwrap_or("<anonymous>"))
    }
}

impl fmt::Display for Function <'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.as_ref().map(|t| t.lexeme).unwrap_or("<anonymous>"))
    }
}
