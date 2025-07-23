// lox_class.rs
// Implements LoxClass, representing class objects and inheritance in rlox.

// author: akrm al-hakimi
// our class

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use crate::{
    error::RuntimeError, 
    interpreter::{
        Interpreter, 
        Value
    },
    callable::Callable,
    instance::LoxInstance,
    function::Function,
};

#[derive(Debug, Clone)]
pub struct LoxClass<'source> {
    pub name: String,
    pub superclass: Option<Rc<LoxClass<'source>>>,
    methods: HashMap<String, Function<'source>>,
}

impl <'source> LoxClass<'source>{
    pub fn new(name: String, methods: HashMap<String, Function<'source>>, superclass: Option<Rc<LoxClass<'source>>>) -> Self {
        Self {
            name,
            methods,
            superclass,
        }
    }

    pub fn find_method(&self, name: &str) -> Option<&Function<'source>> {
        if let Some(method) = self.methods.get(name) {
            return Some(method);
        }

        if let Some(ref superclass) = self.superclass {
            return superclass.find_method(name);
        }
        None
    }
}

impl <'source> Callable<'source> for LoxClass<'source> {

    fn call(
        &self, 
        interpreter: &mut Interpreter<'source>, 
        args: Vec<Value<'source>>
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let instance = Rc::new(RefCell::new(LoxInstance::new(self.clone())));

        if let Some(initializer )= self.find_method("init") {
            initializer.bind(instance.clone()).call(interpreter, args)?;
        }
        Ok(Value::Instance(instance))
    }

    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }

}

impl std::fmt::Display for LoxClass<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
