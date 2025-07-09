// callable.rs
// author: akrm al-hakimi
// This file defines the Callable trait, which is used for functions and classes in our interpreter.


use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::interpreter::Value;
use std::fmt;
use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait Callable<'source>: Debug {
    fn call(
        &self,
        interpreter: &mut Interpreter<'source>,
        args: Vec<Value<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>>;
    fn arity(&self) -> usize;
}

#[derive(Debug)]
pub struct Clock;

impl<'source> Callable<'source> for Clock {
    fn call(
        &self,
        _interpreter: &mut Interpreter<'source>,
        _args: Vec<Value<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let now = SystemTime::now();
        let duration_since_epoch = now
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX_EPOCH!");
        let seconds_since_epoch = Value::Number(duration_since_epoch.as_secs_f64());
        Ok(seconds_since_epoch)
    }

    fn arity(&self) -> usize {
        0
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn>")
    }
}
