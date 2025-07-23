// inst.rs
// Implements LoxInstance, representing object instances in rlox.

use std::{
    collections::HashMap,
    rc::Rc,
    cell::RefCell,
};
use crate::{
    class::LoxClass, 
    error::RuntimeError, 
    interpreter::interp::Value, 
    token::Token,
};

#[derive(Debug, Clone)]
pub struct LoxInstance<'source> {
    klass: LoxClass<'source>,
    fields: HashMap<String, Value<'source>>,
}

impl <'source> LoxInstance<'source> {

    pub fn new(klass: LoxClass<'source>) -> Self {
        Self {
            klass,
            fields: HashMap::new()
        }
    }

    pub fn get(&self, instance: Rc<RefCell<LoxInstance<'source>>>, name: Token<'source>) -> Result<Value<'source>, RuntimeError<'source>> {
        if let Some(value) = self.fields.get(name.lexeme) {
            return Ok(value.clone());
        } 

        if let Some(method) = self.klass.find_method(name.lexeme) {
            return Ok(Value::Callable(Rc::new(method.bind(instance.clone()))))
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