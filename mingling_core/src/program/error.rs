use std::any::Any;
use std::fmt;
use thiserror::Error;

/// Error type returned when a panic occurs during execution.
#[derive(Error)]
#[error("execution panicked: {payload:?}")]
pub struct ProgramPanic {
    pub payload: Box<dyn Any + Send>,
}

impl ProgramPanic {
    pub fn new(payload: Box<dyn Any + Send>) -> Self {
        ProgramPanic { payload }
    }
}

impl fmt::Debug for ProgramPanic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.payload)
    }
}
