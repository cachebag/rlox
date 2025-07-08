use crate::{ast::stmt::FunctionDecl, environment::{Environment, SharedEnv}, interpreter::{Interpreter, Value}};
use std::fmt;
use crate::callable::Callable;
use crate::error::RuntimeError;

pub struct Function<'source> {
    pub declaration: FunctionDecl<'source>,
    pub closure: SharedEnv<'source>,
}

impl <'source> Callable <'source> for Function <'source> {

    fn call(&self, interpreter: &mut Interpreter<'source>, args: Vec<Value<'source>>) -> Result<Value<'source>, RuntimeError> {
        let env = Environment::from_enclosing(self.closure.clone());

        for (param, arg) in self.declaration.params.iter().zip(args.into_iter()) {
            env.borrow_mut().define(param.lexeme.to_string(), arg);
        } 

        match interpreter.execute_block(&self.declaration.body, env) {
            Ok(()) => Ok(Value::Nil),
            Err(e) => Err(e),
        }
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

}

impl fmt::Debug for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl fmt::Display for Function <'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}
