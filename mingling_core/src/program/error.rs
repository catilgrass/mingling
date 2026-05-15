use std::any::Any;
use std::fmt;
use thiserror::Error;

/// Error type returned when a panic occurs during execution.
#[derive(Error)]
pub struct ProgramPanic {
    pub payload: Box<dyn Any + Send>,
}

impl fmt::Display for ProgramPanic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(s) = self.payload.downcast_ref::<&str>() {
            write!(f, "{}", s)
        } else if let Some(s) = self.payload.downcast_ref::<String>() {
            write!(f, "{}", s)
        } else {
            write!(f, "")
        }
    }
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
