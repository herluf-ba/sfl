use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::{
    language::{
        ast::Ast,
        token::TokenKind,
        types::{Context, FreeVar, Substitutable, Substitution, TType, TypeFunc},
    },
    message::{Content, Message, Severity},
    phase::{Phase, PhaseResult},
};

pub struct Interpreter {
    errors: Vec<Message>,
    variable_counter: usize,
}

pub enum Value {
    Bool(bool),
    Int(usize),
}

impl Interpreter {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            variable_counter: 0,
        }
    }

    fn interpret() -> Result<Value, ()> {}
}

pub type Input = crate::phase::ast_builder::Output;
pub type Output = Value;
impl Phase<Input, Output> for Interpreter {
    fn new() -> Self {
        Interpreter::new()
    }

    fn run(self: &mut Self, _config: &crate::config::Config, input: &Input) -> PhaseResult<Output> {
        let entry_point = input.get("./main.sfl");

        PhaseResult::Ok(Value::Bool(true))
    }
}
