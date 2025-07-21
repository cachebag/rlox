// inst.rs 
// author: akrm al-hakimi
// instance structure 

use std::{collections::HashMap};
use crate::{
    class::LoxClass, error::RuntimeError, interpreter::interp::Value, token::Token
};

#[derive(Debug, Clone)]
pub struct LoxInstance<'source> {
    klass: LoxClass,
    fields: HashMap<String, Value<'source>>,
}

impl <'source> LoxInstance<'source> {

    pub fn new(klass: LoxClass) -> Self {
        Self {
            klass,
            fields: HashMap::new()
        }
    }

    pub fn get(&self, name: Token<'source>) -> Result<Value<'source>, RuntimeError<'source>> {
        if let Some(value) = self.fields.get(name.lexeme) {
            return Ok(value.clone());
        }

        Err(RuntimeError::TypeError { 
            msg: format!("Undefined property {}.", name.lexeme), 
            line: name.line, 
        })
    }

    pub fn set(&mut self, name: Token<'source>, value: Value<'source>) {
        self.fields.insert(name.lexeme.to_string(), value);
    }
}

impl std::fmt::Display for LoxInstance<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.klass)
    }
}
