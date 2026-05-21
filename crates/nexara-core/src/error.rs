use thiserror::Error;

pub type NexaraResult<T> = Result<T, NexaraError>;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum NexaraError {
    #[error("tool not found")]
    ToolNotFound,
    #[error("tool not allowed")]
    ToolNotAllowed,
    #[error("trust policy denied: {0}")]
    TrustPolicyDenied(String),
    #[error("confirmation required: {0}")]
    ConfirmationRequired(String),
    #[error("invalid params: {0}")]
    InvalidParams(String),
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("execution failed: {0}")]
    ExecutionFailed(String),
    #[error("payload too large: {0} bytes")]
    PayloadTooLarge(usize),
    #[error("concurrency limit exceeded")]
    ConcurrencyLimitExceeded,
    #[error("invalid descriptor: {0}")]
    InvalidDescriptor(String),
}
