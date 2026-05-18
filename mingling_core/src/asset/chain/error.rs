use crate::error::ProgramInternalExecuteError;

/// Represents errors that can occur during chain processing.
#[derive(Debug)]
pub enum ChainProcessError {
    /// An error with a custom description.
    Other(String),

    /// An I/O error that occurred during chain processing.
    IO(std::io::Error),
}

impl std::fmt::Display for ChainProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainProcessError::Other(s) => write!(f, "Other error: {}", s),
            ChainProcessError::IO(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for ChainProcessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ChainProcessError::IO(e) => Some(e),
            ChainProcessError::Other(_) => None,
        }
    }
}

impl From<std::io::Error> for ChainProcessError {
    fn from(e: std::io::Error) -> Self {
        ChainProcessError::IO(e)
    }
}

impl From<ProgramInternalExecuteError> for ChainProcessError {
    fn from(value: ProgramInternalExecuteError) -> Self {
        match value {
            ProgramInternalExecuteError::DispatcherNotFound => {
                ChainProcessError::Other("DispatcherNotFound".into())
            }
            ProgramInternalExecuteError::RendererNotFound(r) => {
                ChainProcessError::Other(format!("RendererNotFound: {}", r))
            }
            ProgramInternalExecuteError::Other(e) => ChainProcessError::Other(e),
            ProgramInternalExecuteError::IO(e) => {
                ChainProcessError::Other(format!("IOError: {:?}", e))
            }
            ProgramInternalExecuteError::REPLPanic(program_panic) => {
                ChainProcessError::Other(format!("REPLPanic: {}", program_panic))
            }
        }
    }
}
