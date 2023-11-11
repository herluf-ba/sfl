use crate::{config::Config, message::Message};

pub enum PhaseResult<R> {
    Ok(R),
    SoftErr(R, Vec<Message>),
    #[allow(dead_code)]
    Err(Vec<Message>),
}

/// A compiler phase.
pub trait Phase<I, R> {
    fn new() -> Self;
    fn run(self: &mut Self, config: &Config, input: &I) -> PhaseResult<R>;
}
