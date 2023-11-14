use std::{collections::HashMap, fmt::Display, path::PathBuf, str::FromStr};

use crate::{
    language::{
        ast::Ast,
        token::{Token, TokenKind},
    },
    message::Message,
    phase::{Phase, PhaseResult},
};

pub struct Interpreter {
    errors: Vec<Message>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Func(Token, Ast),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Bool(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Func(_, _) => write!(f, "func",), // TODO: Can we do better here?
        }
    }
}

impl Interpreter {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }

    fn interpret(
        &mut self,
        ast: &Ast,
        environment: &mut HashMap<String, Value>,
    ) -> Result<Value, ()> {
        match ast {
            Ast::Err => Err(()),
            Ast::Expr(e) => self.interpret(e, environment),
            Ast::Literal(t) => match t.kind {
                TokenKind::LiteralNumber(v) => Ok(Value::Number(v)),
                TokenKind::LiteralBool(v) => Ok(Value::Bool(v)),
                _ => panic!("SFL ERROR: unhandled literal '{:?}'", t),
            },
            Ast::Name(t) => {
                let Some(v) = environment.get(&t.text()) else {
                    panic!("TODO: add unknown name error")
                };
                Ok((*v).clone())
            }
            Ast::Let(t, e1, e2) => {
                let r1 = self.interpret(e1, environment)?;
                environment.insert(t.text(), r1);
                self.interpret(e2, environment)
            }
            Ast::BinaryOp(t, e1, e2) => {
                let r1 = self.interpret(e1, environment)?;
                let r2 = self.interpret(e2, environment)?;
                match t.kind {
                    TokenKind::Plus => {
                        let Value::Number(n1) = r1 else {
                            panic!("SFL ERROR: Typechecker missed binary op '{:?}'", t);
                        };
                        let Value::Number(n2) = r2 else {
                            panic!("SFL ERROR: Typechecker missed binary op '{:?}'", t);
                        };
                        Ok(Value::Number(n1 + n2))
                    }
                    TokenKind::Minus => {
                        let Value::Number(n1) = r1 else {
                            panic!("SFL ERROR: Typechecker missed binary op '{:?}'", t);
                        };
                        let Value::Number(n2) = r2 else {
                            panic!("SFL ERROR: Typechecker missed binary op '{:?}'", t);
                        };
                        Ok(Value::Number(n1 - n2))
                    }
                    _ => panic!("SFL ERROR: unhandled binary operator '{:?}'", t),
                }
            }
            Ast::Abstraction(t, e) => Ok(Value::Func(t.clone(), (**e).clone())),
            Ast::Application(e1, e2) => match self.interpret(e1, environment)? {
                Value::Func(t, ast) => {
                    let v2 = self.interpret(e2, environment)?;
                    environment.insert(t.text(), v2);
                    self.interpret(&ast, environment)
                }
                _ => panic!("SFL ERROR: cannot apply {:?}", e1),
            },
        }
    }
}

pub type Input = crate::phase::ast_builder::Output;
pub type Output = Value;
impl Phase<Input, Output> for Interpreter {
    fn new() -> Self {
        Interpreter::new()
    }

    fn run(self: &mut Self, _config: &crate::config::Config, input: &Input) -> PhaseResult<Output> {
        let entry_point = input
            .get(&PathBuf::from_str("./main.sfl").unwrap())
            .unwrap();

        let result = self.interpret(entry_point, &mut HashMap::new());
        if self.errors.len() == 0 {
            PhaseResult::Ok(result.unwrap())
        } else {
            PhaseResult::Err(self.errors.clone())
        }
    }
}
