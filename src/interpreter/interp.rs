// interp.rs
// Implements the rlox AST interpreter, executing statements and expressions.

use crate::{
    ast::{
        expr::Expr, stmt::{FunctionDecl, Stmt}
    }, callable::{
        Callable,
        Clock,
    }, class::LoxClass, environment::env::{
            Environment,
            SharedEnv,
        }, error::RuntimeError, function::Function, instance::{LoxInstance}, token::{
        Literal, 
        Token, 
        TokenType
    }
};
use core::fmt;
use std::{cell::RefCell, rc::Rc};
use std::collections::HashMap;
use by_address::ByAddress;

type ExprRef<'source> = Rc<Expr<'source>>;
type ExprKey<'source> = ByAddress<ExprRef<'source>>;


pub struct Interpreter<'source> {
    pub globals: SharedEnv<'source>,
    pub environment: SharedEnv<'source>,
    pub locals: HashMap<ExprKey<'source>, usize>,
}

#[derive(Debug, Clone)]
pub enum Value<'source> {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
    Callable(Rc<dyn Callable<'source> + 'source>),
    Class(Rc<LoxClass<'source>>),
    Instance(Rc<RefCell<LoxInstance<'source>>>),
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (String(a), String(b)) => a == b,
            (Number(a), Number(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Nil, Nil) => true,
            // Callable values are never equal
            (Callable(_), Callable(_)) => false,
            (Class(_), Class(_)) => false,
            _ => false,
        }
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'source> Default for Interpreter<'source> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'source> Interpreter<'source> {
    pub fn new() -> Self {
        let globals = Environment::new();
        globals
            .borrow_mut()
            .define("clock".to_string(), Value::Callable(Rc::new(Clock)));

        Interpreter {
            globals: globals.clone(),
            environment: globals,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(
        &mut self,
        statements: &[Stmt<'source>],
    ) -> Result<(), RuntimeError<'source>> {
        for statement in statements {
            self.execute(statement)?
        }
        Ok(())
    }

    pub fn evaluate(
        &mut self,
        expr: Rc<Expr<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        match &*expr {
            Expr::Lambda { params, body } => self.evaluate_lambda(params.clone(), body.clone()),
            Expr::Literal(lit) => self.evaluate_literal(lit.clone()),
            Expr::Unary { operator, right } => self.evaluate_unary(operator.clone(), &right.clone()),
            Expr::Mutate {
                operator,
                operand,
                postfix,
            } => self.evaluate_mutation(operator.clone(), operand.clone(), *postfix),
            Expr::Call {
                callee,
                paren,
                args,
            } => self.evaluate_call(callee.clone(), paren.clone(), args.clone()),
            Expr::Assign { name, value } => self.evaluate_assignment(expr.clone(), name.clone(), value.clone()),
            Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_binary(left.clone(), operator, right.clone()),
            Expr::Variable { name } => {
                let key = ByAddress(expr.clone());
                if let Some(distance) = self.locals.get(&key) {
                    Environment::get_at(self.environment.clone(), *distance, name)
                } else {
                    self.globals.borrow().get(name)
                }
            }
            Expr::Get { object, name } => {
                self.evaluate_get(object.clone(), name.clone())
            }
            Expr::Set { object, name, value } => {
                self.evaluate_set(object, name.clone(), value)
            }
            Expr::Super { keyword, method } => {
                self.evaluate_super(expr.clone(), keyword.clone(), method.clone())
            }
            Expr::This { keyword } => {
                let key = ByAddress(expr.clone());
                if let Some(distance) = self.locals.get(&key) {
                    Environment::get_at(self.environment.clone(), *distance, keyword)
                } else {
                    self.globals.borrow().get(keyword)
                }
            }
            Expr::Grouping(inner) => self.evaluate(inner.clone()),
            Expr::Ternary {
                condition,
                true_expr, 
                false_expr,
            } => self.evaluate_ternary(condition.clone(), true_expr.clone(), false_expr.clone()),
            Expr::Logical {
                left,
                operator,
                right,
            } => self.evaluate_logical(left.clone(), operator, right.clone()),
        }
    }

    fn evaluate_function(
        &mut self,
        decl: FunctionDecl<'source>,
    ) -> Result<(), RuntimeError<'source>> {
        let function = Function {
            declaration: decl.clone(),
            closure: self.environment.clone(),
            is_initializer: false,
        };
        self.environment.borrow_mut().define(
            decl.name.as_ref().map(|t| t.lexeme).unwrap_or("<anonymous>").to_string(),
            Value::Callable(Rc::new(function)),
        );
        Ok(())
    }

    pub fn execute(&mut self, stmt: &Stmt<'source>) -> Result<(), RuntimeError<'source>> {
        match stmt {
            Stmt::Block(statements) => {
                let new_env = Environment::from_enclosing(self.environment.clone());
                self.execute_block(statements, new_env)?;
                Ok(())
            }
            Stmt::Class { name: _, superclass: _, methods: _ } => {
                let _value = self.evaluate_class(stmt.clone())?;
                Ok(())
            }
            Stmt::Expression(expr) => {
                let _value = self.evaluate(expr.clone())?;
                Ok(())
            }
            Stmt::Function(decl) => self.evaluate_function(decl.clone()),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.evaluate_if_statement(condition.clone(), then_branch, else_branch.as_deref())?;
                Ok(())
            }
            Stmt::Print(expr) => {
                let value = self.evaluate(expr.clone())?;
                println!("{}", value);
                Ok(())
            }
            Stmt::Return { keyword: _, value } => {
                let result = match value {
                    Some(expr) => self.evaluate(expr.clone())?,
                    None => Value::Nil,
                };
                Err(RuntimeError::ReturnException(result))
            }
            Stmt::While { condition, body } => {
                self.evaluate_while(condition.clone(), body)?;
                Ok(())
            }
            Stmt::Break { keyword: _ } => {
                self.evaluate_break()?;
                Ok(())
            }
            // In jlox, you can define unitialized variables but if you use them they'll just be nil
            Stmt::Var { name, initializer } => {
                self.evaluate_var_decl(name.clone(), initializer.clone())?;
                Ok(())
            }
        }
    }

    pub fn resolve(&mut self, expr: Rc<Expr<'source>>, depth: usize) {
        self.locals.insert(ByAddress(expr), depth);
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt<'source>],
        new_env: SharedEnv<'source>,
    ) -> Result<(), RuntimeError<'source>> {
        let previous = self.environment.clone();
        self.environment = new_env;

        let result = statements
            .iter()
            .try_for_each(|stmt| self.execute(stmt));

        self.environment = previous;
        result
    }

    pub fn evaluate_class(&mut self, class: Stmt<'source>) -> Result<Value<'source>, RuntimeError<'source>> {
        if let Stmt::Class { name, superclass, methods } = class {
            self.environment.borrow_mut().define(name.lexeme.to_string(), Value::Nil);

            let mut super_class_value: Option<Rc<LoxClass<'source>>> = None;
            if let Some(super_expr) = &superclass {
                let eval = self.evaluate(super_expr.clone())?;
                match eval {
                    Value::Class(class_obj) => {
                        super_class_value = Some(class_obj.clone());

                        let new_env: Rc<RefCell<Environment>> = Environment::from_enclosing(self.environment.clone());
                        Environment::define(&mut new_env.borrow_mut(),"super".to_string(), Value::Class(class_obj.clone()));
                        self.environment = new_env;
                    }
                    _ => {
                        return Err(RuntimeError::TypeError { 
                            msg: "Superclass must be a class.".to_string(), 
                            line: name.line,
                        })
                    }
                }
            }
            let mut method_map: HashMap<String, Function<'source>> = HashMap::new();
            for method in methods {
                let function = if method.name.as_ref().map(|name| name.lexeme) == Some("init") {
                    Function::new_initializer(method.clone(), self.environment.clone())
                } else {
                    Function::new(method.clone(), self.environment.clone())
                };
                if let Some(method_name) = method.name {
                    method_map.insert(method_name.lexeme.to_string(), function);
                }
            }
            let klass = LoxClass::new(name.lexeme.to_string(), method_map, super_class_value);
            self.environment.borrow_mut().assign(name, &Value::Class(klass.into()))?;

            if superclass.is_some() {
                    let enclosing_env = {
                        let env_ref = self.environment.borrow();
                        env_ref.enclosing.clone()
                    };
                    if let Some(enclosing) = enclosing_env {
                        self.environment = enclosing;
                }
            }
            Ok(Value::Nil)
        } else {
            Err(RuntimeError::TypeError {
                msg: "Expected class statement".to_string(),
                line: 0
            })
        }
    }

    fn evaluate_lambda(
        &mut self,
        paramaters: Vec<Token<'source>>,
        body_block: Vec<Stmt<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let function = Function {
            declaration: FunctionDecl {
                name: None,
                params: paramaters,
                body: body_block,
            },
            closure: self.environment.clone(),
            is_initializer: false,
        };
        Ok(Value::Callable(Rc::new(function)))
    }

    fn evaluate_var_decl(
        &mut self,
        name: Token,
        initializer: Option<Rc<Expr<'source>>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let value = match initializer {
            Some(expr) => self.evaluate(expr)?,
            None => Value::Nil,
        };

        self.environment
            .borrow_mut()
            .define(name.lexeme.to_string(), value);
        Ok(Value::Nil)
    }

    fn evaluate_while(
        &mut self,
        cond: Rc<Expr<'source>>,
        body: &Stmt<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        while {
            let cond_val = self.evaluate(cond.clone())?;
            self.is_truthy(&cond_val)
        } {
            match self.execute(body) {
                Err(RuntimeError::BreakException) => break,
                Err(e) => return Err(e),
                _ => {}
            }
        }
        Ok(Value::Nil)
    }

    fn evaluate_break(&mut self) -> Result<(), RuntimeError<'source>> {
        Err(RuntimeError::BreakException)
    }

    fn evaluate_if_statement(
        &mut self,
        cond: Rc<Expr<'source>>,
        then_b: &Stmt<'source>,
        else_b: Option<&Stmt<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let condition_val = self.evaluate(cond)?;

        if self.is_truthy(&condition_val) {
            self.execute(then_b)?; // Remove the array wrapping
        } else if let Some(else_stmt) = else_b {
            self.execute(else_stmt)?;
        }

            Ok(Value::Nil)
        }

    fn evaluate_assignment(
        &mut self,
        expr: Rc<Expr<'source>>,
        name: Token<'source>,
        value_expr: Rc<Expr<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let value = self.evaluate(value_expr)?;
        let key = ByAddress(expr.clone());

        if let Some(distance) = self.locals.get(&key) {
            Environment::assign_at(self.environment.clone(), *distance, name, &value)?;
        } else {
            self.globals.borrow_mut().assign(name, &value)?;
        }

        Ok(value)
    }

    fn evaluate_literal(&self, lit: Literal) -> Result<Value<'source>, RuntimeError<'source>> {
        match lit {
            Literal::Num(n) => Ok(Value::Number(n)),
            Literal::Str(s) => Ok(Value::String(s.clone())),
            Literal::True => Ok(Value::Bool(true)),
            Literal::False => Ok(Value::Bool(false)),
            Literal::Nil => Ok(Value::Nil),
        }
    }

    fn evaluate_logical(
        &mut self,
        lhs: Rc<Expr<'source>>,
        operator: &Token,
        rhs: Rc<Expr<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let left = self.evaluate(lhs)?;
        match operator.kind {
            TokenType::Or => {
                if self.is_truthy(&left) {
                    Ok(left)
                } else {
                    self.evaluate(rhs)
                }
            }
            TokenType::And => {
                if !self.is_truthy(&left) {
                    Ok(left)
                } else {
                    self.evaluate(rhs)
                }
            }
            _ => unreachable!("Unknown logical operator."),
        }
    }

    fn evaluate_set(
        &mut self,
        object: &Rc<Expr<'source>>,
        name: Token<'source>,
        value: &Rc<Expr<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let object = self.evaluate(object.clone())?;

        match object {
            Value::Instance(instance) => {
                let val = self.evaluate(value.clone())?;
                instance.borrow_mut().set(name, val.clone());
                Ok(val)
            }
            _ => Err(RuntimeError::TypeError { 
                msg: "Invalid set target.".to_string(), 
                line: name.line 
            })
        }
    }

    fn evaluate_super(
        &mut self,
        expr: Rc<Expr<'source>>,
        keyword: Token<'source>,
        method: Token<'source>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let distance = if let Some(distance) = self.locals.get(&ByAddress(expr.clone())) {
            *distance
        } else {
            return Err(RuntimeError::TypeError { 
                msg: "Undefined variable 'super'.".into(), 
                line: keyword.line
            });
        };

        let superclass = match Environment::get_at_string(self.environment.clone(), distance, "super")? {
            Value::Class(class_rc) => class_rc.clone(),
            _ => return Err(RuntimeError::TypeError { 
                msg: "super must be a class.".into(), 
                line: keyword.line
            }),
        };

        let object = match Environment::get_at_string(self.environment.clone(), distance - 1, "this")? {
            Value::Instance(instance_rc) => instance_rc.clone(),
            _ => return Err(RuntimeError::TypeError { 
                msg: "'this' must be instance.".into(),
                line: keyword.line 
            })
        };

        // Now you need to find the method and return it
        // This part depends on your class/method implementation
        if let Some(method_fn) = superclass.find_method(method.lexeme) {
            Ok(Value::Callable(Rc::new(method_fn.bind(object))))
        } else {
            Err(RuntimeError::UndefinedVariable { 
                found: method.lexeme.to_string(),
            })
        }
    }

    fn evaluate_unary(
        &mut self,
        operator: Token,
        right: &Rc<Expr<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let right_val = self.evaluate(right.clone())?;

        match operator.kind {
            TokenType::Minus => match right_val {
                Value::Number(n) => Ok(Value::Number(-n)),
                _ => Err(RuntimeError::UnaryMinus {
                    lexeme: operator.lexeme.to_string(),
                    line: operator.line,
                }),
            },
            TokenType::Bang => Ok(Value::Bool(!self.is_truthy(&right_val))),
            _ => unreachable!("Unknown unary operator"),
        }
    }

    fn evaluate_mutation(
        &mut self,
        operator: Token,
        operand: Rc<Expr<'source>>,
        postfix: bool,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let name = match operand.as_ref() {
            Expr::Variable { name } => name,
            _ => {
                return Err(RuntimeError::MutationError {
                    lexeme: operator.lexeme.to_string(),
                    line: operator.line,
                });
            }
        };

        let current_value = self.environment.borrow().get(name)?;

        match current_value {
            Value::Number(n) => match operator.kind {
                TokenType::Increment => {
                    let new_val = n + 1.0;
                    self.environment
                        .borrow_mut()
                        .assign(name.clone(), &Value::Number(new_val))?;
                    if postfix {
                        Ok(Value::Number(n))
                    } else {
                        Ok(Value::Number(new_val))
                    }
                }
                TokenType::Decrement => {
                    let new_val = n - 1.0;
                    self.environment
                        .borrow_mut()
                        .assign(name.clone(), &Value::Number(new_val))?;
                    if postfix {
                        Ok(Value::Number(n))
                    } else {
                        Ok(Value::Number(new_val))
                    }
                }
                _ => unreachable!("Illegal mutation."),
            },
            _ => Err(RuntimeError::MutationError {
                lexeme: operator.to_string(),
                line: operator.line,
            }),
        }
    }

    fn evaluate_binary(
        &mut self,
        left: Rc<Expr<'source>>,
        operator: &Token,
        right: Rc<Expr<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let left_val = self.evaluate(left.clone())?;
        let right_val = self.evaluate(right.clone())?;
        let lexeme = operator.lexeme.to_string();
        let line = operator.line;

        match operator.kind {
            TokenType::Comma => {
                self.evaluate(left)?;
                self.evaluate(right)
            }
            TokenType::Plus => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                (Value::String(l), Value::Number(r)) => Ok(Value::String(l + &r.to_string())),
                (Value::Number(l), Value::String(r)) => Ok(Value::String(l.to_string() + &r)),
                _ => Err(RuntimeError::BinaryPlus { lexeme, line }),
            },
            TokenType::Minus => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                _ => Err(RuntimeError::BinaryMinus { lexeme, line }),
            },
            TokenType::Star => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                _ => Err(RuntimeError::BinaryMult { lexeme, line }),
            },
            TokenType::Slash => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => {
                    if r == 0.0 {
                        return Err(RuntimeError::BinaryDBZ { line });
                    }
                    Ok(Value::Number(l / r))
                }
                _ => Err(RuntimeError::BinaryDiv { lexeme, line }),
            },
            TokenType::EqualEqual => Ok(Value::Bool(left_val == right_val)),
            TokenType::Greater => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l > r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::Less => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::GreaterEqual => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l >= r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::LessEqual => match (left_val, right_val) {
                (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l <= r)),
                _ => Err(RuntimeError::BinaryComp { lexeme, line }),
            },
            TokenType::BangEqual => Ok(Value::Bool(left_val != right_val)),
            _ => unreachable!("Unknown binary operator"),
        }
    }

    fn evaluate_call(
        &mut self,
        callee: Rc<Expr<'source>>,
        paren: Token<'source>,
        args: Vec<Rc<Expr<'source>>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let callee = self.evaluate(callee)?;
        let mut arguments: Vec<Value<'source>> = Vec::new();

        for argument in args {
            arguments.push(self.evaluate(argument)?);
        }

        match callee {
            Value::Callable(f) => {
                if arguments.len() != f.arity() {
                    return Err(RuntimeError::FunctionError {
                        lexeme: paren.to_string(),
                        line: paren.line,
                        message: "Can only call functions and classes.".to_string(),
                    });
                }
                f.call(self, arguments)
            }
            Value::Class(class) => {
                if arguments.len() != class.arity() {
                    return Err(RuntimeError::FunctionError {
                        lexeme: paren.to_string(),
                        line: paren.line,
                        message: "Ensure your function call matches the function arity.".to_string(),
                    });
                }
                class.call(self, arguments)
            }
            _ => {
                Err(RuntimeError::FunctionError {
                    lexeme: paren.to_string(),
                    line: paren.line,
                    message: "Can only call functions and classes.".to_string(),
                })
            }
        }
    }

    fn evaluate_get(&mut self, object_expr: Rc<Expr<'source>>, name: Token<'source>) -> Result<Value<'source>, RuntimeError<'source>> {  
        let object = self.evaluate(object_expr)?;
        match object {
            Value::Instance(instance) => {
                // Don't drop the borrow too early
                instance.borrow().get(instance.clone(), name)
            }               
            _ => Err(RuntimeError::TypeError { 
                msg: "Only instances have properties.".to_string(), 
                line: name.line 
            })
        }
    }

    fn evaluate_ternary(
        &mut self,
        condition: Rc<Expr<'source>>,
        true_expr: Rc<Expr<'source>>,
        false_expr: Rc<Expr<'source>>,
    ) -> Result<Value<'source>, RuntimeError<'source>> {
        let condition_val = self.evaluate(condition)?;

        if self.is_truthy(&condition_val) {
            self.evaluate(true_expr)
        } else {
            self.evaluate(false_expr)
        }
    }

    fn is_truthy(&self, value: &Value<'source>) -> bool {
        match value {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Callable(c) => write!(f, "{:?}", c),
            Value::Class(class) => write!(f, "{}", class),
            Value::Instance(instance) => {
                    let borrowed = instance.borrow();
                    write!(f, "{} instance", borrowed)
            }

        }
    }
}