// environment.rs
// author: Akrm Al-Hakimi
// Environment module for managing variable scopes in the interpreter 
// We do this a bit different than in Java, of course...
//                  - We don't get to use operator overloading for enclosing environments
//                  so we have to use a Box to hold the enclosing environment recursively.
//                  - Rust differs from Java in how recursive structures are handled:
//                      - Recursive types (like a chain of environments) require indirection
//                      - We use `Option<Box<Environment>>` to support nesting while maintaining known size at compile time
// This forms the backbone of scope management for block scopes, functions, and closures in the interpreter.

use std::{collections::HashMap};
use crate::{error::error::{RuntimeError}, interpreter::Value};
use crate::token::token::Token;


pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Value>,
}


impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn from_enclosing(enclosing: Environment) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }


    pub fn define(&mut self, name: String, val: Value) -> Result<(), RuntimeError> {
        self.values.insert(name, val);
        Ok(())
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(val) = self.values.get(name.lexeme) {
            Ok(val.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            Err(RuntimeError::UndefinedVariable {
                found: name.lexeme.to_string(),
            })
        }  
    } 

    pub fn assign(&mut self, name: Token, val: &Value) -> Result<Value, RuntimeError> {
        if self.values.contains_key(name.lexeme) {
            Ok(self.values.insert(name.lexeme.to_string(), val.clone()).unwrap_or(Value::Nil))
        } else if let Some(enclosing) = self.enclosing.as_mut() {
            enclosing.assign(name, val)
        } else { 
            Err(RuntimeError::UndefinedVariable {
                found: name.lexeme.to_string() 
            })
        }
    }
}

