use crate::error::ProgramInternalExecuteError;

#[derive(thiserror::Error, Debug)]
pub enum ChainProcessError {
    #[error("Other error: {0}")]
    Other(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
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
        }
    }
}
