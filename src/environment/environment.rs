use std::{collections::HashMap};
use crate::{error::error::{RuntimeError}, interpreter::Value};
use crate::token::token::Token;


pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: HashMap<String, Value>,
}

impl Environment {
    
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, val: Value) -> Result<(), RuntimeError> {
        self.values.insert(name, val);
        Ok(())
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if self.values.contains_key(name.lexeme) {
            Ok(self.values.get(name.lexeme).cloned().unwrap_or(Value::Nil))
        } else {
            Err(RuntimeError::UndefinedVariable { 
                found: name.lexeme.to_string() 
            })
        }
    } 

    pub fn assign(&mut self, name: Token, val: &Value) -> Result<Value, RuntimeError> {
        if self.values.contains_key(name.lexeme) {
            Ok(self.values.insert(name.lexeme.to_string(), val.clone()).unwrap_or(Value::Nil))
        } else {
            Err(RuntimeError::UndefinedVariable {
                found: name.lexeme.to_string() 
            })
        }
    }
}
