use crate::error::{ChainProcessError, ProgramPanic};

#[derive(thiserror::Error, Debug)]
pub enum ProgramExecuteError {
    #[error("No Dispatcher Found")]
    DispatcherNotFound,

    #[error("No Renderer (`{0}`) Found")]
    RendererNotFound(String),

    #[error("Panic: {0:?}")]
    Panic(#[from] ProgramPanic),

    #[error("Other error: {0}")]
    Other(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ProgramInternalExecuteError {
    #[error("No Dispatcher Found")]
    DispatcherNotFound,

    #[error("No Renderer (`{0}`) Found")]
    RendererNotFound(String),

    #[error("Other error: {0}")]
    Other(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
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
