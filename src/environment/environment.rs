use std::{collections::HashMap};
use crate::{error::error::RuntimeError, interpreter::Value};
use crate::token::token::Token;


pub struct Environment {
    values: HashMap<String, Value>, 
}

impl Environment {
    
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    fn define(&mut self, name: String, val: Value) -> Result<(), RuntimeError> {
        self.values.insert(name.to_string(), val);
        Ok(())
    }

    fn get(&self, name: Token) -> Result<Value, RuntimeError> {
        if self.values.contains_key(name.lexeme) {
            Ok(self.values.get(name.lexeme).cloned().unwrap_or(Value::Nil))
        } else {
            Err(RuntimeError::UndefinedVariable { 
                found: name.lexeme.to_string() 
            })
        }
    }
}
