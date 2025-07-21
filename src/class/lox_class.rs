// loxclass.rs 
// author: akrm al-hakimi
// our class

use std::collections::HashMap;

use crate::{
    error::RuntimeError, 
    interpreter::{
        Interpreter, 
        Value
    },
    callable::Callable,
    instance::LoxInstance,
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
}

impl LoxClass {
    pub fn new(name: String) -> Self {
        Self {
            name,
        }
    }    
}

impl <'source> Callable<'source> for LoxClass {
    fn call(&self, interpreter: &mut Interpreter<'source>, args: Vec<Value<'source>>) -> Result<Value<'source>, RuntimeError<'source>> {
        let instance = LoxInstance::new(self.clone());
        Ok(Value::Instance(instance))
    }

    fn arity(&self) -> usize {
        0
    }

}

impl std::fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
