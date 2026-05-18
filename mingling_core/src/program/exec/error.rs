use crate::error::{ChainProcessError, ProgramPanic};
use std::fmt;

/// Errors that can occur during program execution.
///
/// This enum represents the various error conditions that may arise
/// when executing a program, including missing dispatchers/renderers,
/// panics, and other miscellaneous errors.
#[derive(Debug)]
pub enum ProgramExecuteError {
    /// No dispatcher was found to handle the program execution.
    DispatcherNotFound,

    /// No renderer was found for the given name.
    RendererNotFound(String),

    /// The program encountered a panic during execution.
    Panic(ProgramPanic),

    /// An other error occurred.
    Other(String),
}

impl fmt::Display for ProgramExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramExecuteError::DispatcherNotFound => write!(f, "No Dispatcher Found"),
            ProgramExecuteError::RendererNotFound(s) => {
                write!(f, "No Renderer (`{}`) Found", s)
            }
            ProgramExecuteError::Panic(p) => write!(f, "Panic: {:?}", p),
            ProgramExecuteError::Other(s) => write!(f, "Other error: {}", s),
        }
    }
}

impl std::error::Error for ProgramExecuteError {}

impl From<ProgramPanic> for ProgramExecuteError {
    fn from(value: ProgramPanic) -> Self {
        ProgramExecuteError::Panic(value)
    }
}

/// Errors that can occur during internal program execution.
///
/// This enum represents error conditions that arise specifically within
/// the internal execution pipeline of a program, including missing
/// dispatchers/renderers, I/O errors, and other miscellaneous failures.
/// These errors are typically not exposed directly to the end user but
/// are used internally and can be converted into [`ProgramExecuteError`].
#[derive(Debug)]
pub enum ProgramInternalExecuteError {
    /// No dispatcher was found to handle the program execution.
    DispatcherNotFound,

    /// No renderer was found for the given name.
    RendererNotFound(String),

    /// An other internal error occurred.
    Other(String),

    /// A single REPL execution failed
    REPLPanic(ProgramPanic),

    /// An I/O error occurred during execution.
    IO(std::io::Error),
}

impl fmt::Display for ProgramInternalExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramInternalExecuteError::DispatcherNotFound => {
                write!(f, "No Dispatcher Found")
            }
            ProgramInternalExecuteError::RendererNotFound(s) => {
                write!(f, "No Renderer (`{}`) Found", s)
            }
            ProgramInternalExecuteError::Other(s) => write!(f, "Other error: {}", s),
            ProgramInternalExecuteError::IO(e) => write!(f, "IO error: {}", e),
            ProgramInternalExecuteError::REPLPanic(panic) => {
                write!(f, "A single REPL execution failed: {}", panic)
            }
        }
    }
}

impl std::error::Error for ProgramInternalExecuteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProgramInternalExecuteError::IO(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProgramInternalExecuteError {
    fn from(e: std::io::Error) -> Self {
        ProgramInternalExecuteError::IO(e)
    }
}

impl From<ProgramInternalExecuteError> for ProgramExecuteError {
    fn from(value: ProgramInternalExecuteError) -> Self {
        match value {
            ProgramInternalExecuteError::DispatcherNotFound => {
                ProgramExecuteError::DispatcherNotFound
            }
            ProgramInternalExecuteError::RendererNotFound(s) => {
                ProgramExecuteError::RendererNotFound(s)
            }
            ProgramInternalExecuteError::Other(s) => ProgramExecuteError::Other(s),
            ProgramInternalExecuteError::IO(e) => ProgramExecuteError::Other(format!("{}", e)),
            ProgramInternalExecuteError::REPLPanic(p) => ProgramExecuteError::Other(format!(
                "A single REPL execution failed: {}",
                p
            )),
        }
    }
}

impl From<ChainProcessError> for ProgramInternalExecuteError {
    fn from(value: ChainProcessError) -> Self {
        match value {
            ChainProcessError::Other(s) => ProgramInternalExecuteError::Other(s),
            ChainProcessError::IO(error) => ProgramInternalExecuteError::IO(error),
        }
    }
}
