use rmcp::Error as McpError;
use thiserror::Error;

// Internal error types for detailed error categorization
#[derive(Error, Debug)]
pub enum BrpMcpError {
    #[error("Mutex poisoned: {0}")]
    MutexPoisoned(String),

    #[error("BRP communication failed: {0}")]
    BrpCommunication(String),

    #[error("Format discovery error: {0}")]
    FormatDiscovery(String),

    #[error("File operation failed: {0}")]
    FileOperation(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Watch operation failed: {0}")]
    WatchOperation(String),

    #[error("Process management error: {0}")]
    ProcessManagement(String),

    #[error("Parameter extraction failed: {0}")]
    ParameterExtraction(String),

    #[error("Log operation failed: {0}")]
    LogOperation(String),
}

// Conversion to McpError for API boundaries
impl From<BrpMcpError> for McpError {
    fn from(err: BrpMcpError) -> Self {
        match err {
            BrpMcpError::MutexPoisoned(msg) => McpError::internal_error(msg, None),
            BrpMcpError::BrpCommunication(msg)
            | BrpMcpError::FormatDiscovery(msg)
            | BrpMcpError::Configuration(msg)
            | BrpMcpError::ParameterExtraction(msg) => McpError::invalid_params(msg, None),
            BrpMcpError::FileOperation(msg)
            | BrpMcpError::InvalidState(msg)
            | BrpMcpError::WatchOperation(msg)
            | BrpMcpError::ProcessManagement(msg)
            | BrpMcpError::LogOperation(msg) => McpError::internal_error(msg, None),
        }
    }
}
