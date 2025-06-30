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

use std::{
    collections::HashMap,
    rc::Rc,
    cell::RefCell,
};
use crate::{error::error::{RuntimeError}, interpreter::Value};
use crate::token::token::Token;

pub type SharedEnv = Rc<RefCell<Environment>>;

pub struct Environment {
    enclosing: Option<SharedEnv>,
    values: HashMap<String, Value>,
}

impl Environment {
    
    pub fn new() -> SharedEnv {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn from_enclosing(enclosing: SharedEnv) -> SharedEnv {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }))
    }

    pub fn define(&mut self, name: String, val: Value) {
        self.values.insert(name, val);
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(val) = self.values.get(name.lexeme) {
            Ok(val.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            Err(RuntimeError::UndefinedVariable {
                found: name.lexeme.to_string(),
            })
        }  
    }
 


    pub fn assign(&mut self, name: Token, val: &Value) -> Result<Value, RuntimeError> {
        if self.values.contains_key(name.lexeme) {
            self.values.insert(name.lexeme.to_string(), val.clone());
            Ok(val.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, val)
        } else {
            Err(RuntimeError::UndefinedVariable {
                found: name.lexeme.to_string(),
            })
        }
    }
}

