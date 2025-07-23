// func.rs
// Implements function objects for rlox, supporting closures and binding.

use crate::callable::Callable;
use crate::error::RuntimeError;
use crate::{
    ast::stmt::FunctionDecl,
    environment::{Environment, SharedEnv},
    instance::LoxInstance,
    interpreter::{Interpreter, Value},
};
use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Clone)]
pub struct Function<'source> {
    pub declaration: FunctionDecl<'source>,
    pub closure: SharedEnv<'source>,
    pub is_initializer: bool,
}

impl<'source> Function<'source> {
    pub fn new(declaration: FunctionDecl<'source>, closure: SharedEnv<'source>) -> Self {
        let is_initializer = declaration
            .name
            .as_ref()
            .map(|name| name.lexeme == "init")
            .unwrap_or(false);

        Self {
            declaration,
            closure,
            is_initializer,
        }
    }

    pub fn new_initializer(
        declaration: FunctionDecl<'source>,
        closure: SharedEnv<'source>,
    ) -> Self {
        Self {
            declaration,
            closure,
            is_initializer: true,
        }
    }

    pub fn bind(&self, instance: Rc<RefCell<LoxInstance<'source>>>) -> Function<'source> {
        let env = Environment::from_enclosing(self.closure.clone());
        env.borrow_mut()
            .define("this".to_string(), Value::Instance(instance));

        Function {
            declaration: self.declaration.clone(),
            closure: env,
            is_initializer: self.is_initializer,
        }
    }
}

impl<'source> Callable<'source> for Function<'source> {
    fn call(
        &self,
        interpreter: &mut Interpreter<'source>,
        args: Vec<Value<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let env = Environment::from_enclosing(self.closure.clone());

        for (param, arg) in self.declaration.params.iter().zip(args.into_iter()) {
            env.borrow_mut().define(param.lexeme.to_string(), arg);
        }

        let previous = interpreter.environment.clone();
        interpreter.environment = env.clone();

        let result = {
            let mut body_result = Ok(Value::Nil);
            for stmt in &self.declaration.body {
                match interpreter.execute(stmt) {
                    Err(RuntimeError::ReturnException(val)) => {
                        if self.is_initializer {
                            body_result =
                                Environment::get_at_string(self.closure.clone(), 0, "this");
                        } else {
                            body_result = Ok(val);
                        }
                        break;
                    }
                    Err(e) => {
                        body_result = Err(e);
                        break;
                    }
                    Ok(()) => continue,
                }
            }
            if self.is_initializer && matches!(body_result, Ok(Value::Nil)) {
                body_result = Environment::get_at_string(self.closure.clone(), 0, "this");
            }

            body_result
        };

        interpreter.environment = previous;

        result
    }

    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
}

impl fmt::Debug for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<fn {}>",
            self.declaration
                .name
                .as_ref()
                .map(|t| t.lexeme)
                .unwrap_or("<anonymous>")
        )
    }
}

impl fmt::Display for Function<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<fn {}>",
            self.declaration
                .name
                .as_ref()
                .map(|t| t.lexeme)
                .unwrap_or("<anonymous>")
        )
    }
}
