// env.rs
// Implements environment and scope management for variables in rlox.
// Environment module for managing variable scopes in the interpreter

use crate::token::Token;
use crate::{error::RuntimeError, interpreter::Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type SharedEnv<'source> = Rc<RefCell<Environment<'source>>>;

pub struct Environment<'source> {
    pub enclosing: Option<SharedEnv<'source>>,
    values: HashMap<String, Value<'source>>,
}

impl<'source> Environment<'source> {
    pub fn new() -> SharedEnv<'source> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn from_enclosing(enclosing: SharedEnv<'source>) -> SharedEnv<'source> {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }))
    }

    pub fn define(&mut self, name: String, val: Value<'source>) {
        self.values.insert(name, val);
    }

    pub fn ancestor(env: SharedEnv<'source>, distance: usize) -> Option<SharedEnv<'source>> {
        let mut current = env;

        for _ in 0..distance {
            let next = {
                let borrowed = current.borrow();
                borrowed.enclosing.clone()
            };

            match next {
                Some(env) => current = env,
                None => return None,
            }
        }
        Some(current)
    }

    pub fn get_at(
        env: SharedEnv<'source>,
        distance: usize,
        name: &Token<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        if let Some(target) = Self::ancestor(env, distance) {
            target.borrow().get(name)
        } else {
            Err(RuntimeError::UndefinedVariable {
                found: name.lexeme.to_string(),
            })
        }
    }

    pub fn get_at_string(
        env: SharedEnv<'source>,
        distance: usize,
        name: &str,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        if let Some(target) = Self::ancestor(env, distance) {
            if let Some(value) = target.borrow().values.get(name) {
                Ok(value.clone())
            } else {
                Err(RuntimeError::UndefinedVariable {
                    found: name.to_string(),
                })
            }
        } else {
            Err(RuntimeError::UndefinedVariable {
                found: name.to_string(),
            })
        }
    }

    pub fn assign_at(
        env: SharedEnv<'source>,
        distance: usize,
        name: Token<'source>,
        val: &Value<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        if let Some(scope) = Self::ancestor(env, distance) {
            scope.borrow_mut().assign(name, val)
        } else {
            Err(RuntimeError::UndefinedVariable {
                found: name.lexeme.to_string(),
            })
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value<'source>, RuntimeError<'source>> {
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

    pub fn assign(
        &mut self,
        name: Token<'source>,
        val: &Value<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let key = name.lexeme.to_string();

        if self.values.insert(key.clone(), val.clone()).is_some() {
            Ok(val.clone())
        } else {
            // Take a clone of the Rc, not a borrow of self
            if let Some(enclosing) = self.enclosing.clone() {
                enclosing.borrow_mut().assign(name, val)
            } else {
                Err(RuntimeError::UndefinedVariable { found: key })
            }
        }
    }
}
