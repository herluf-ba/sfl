use std::{collections::HashMap, path::PathBuf};

use crate::{config::Config, message::Message};

type Errors = HashMap<PathBuf, Vec<Message>>;

pub enum PhaseResult<R> {
    Ok(R),
    SoftErr(R, Errors),
    #[allow(dead_code)]
    Err(Errors),
}

/// A compiler phase.
pub trait Phase<I, R> {
    fn new() -> Self;
    fn run(self: &mut Self, config: &Config, input: &I) -> PhaseResult<R>;
}
