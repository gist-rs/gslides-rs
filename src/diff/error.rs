use thiserror::Error;

/// Custom error types for the slides diffing process.
#[derive(Error, Debug)]
pub enum DiffError {
    #[error("Serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Diffing failed: {0}")]
    Diffing(serde_json::Error),

    #[error("Text diff generation failed: {0}")]
    Format(#[from] std::fmt::Error),

    #[error("Failed to retrieve context for path: {0}")]
    ContextRetrieval(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path format: {0}")]
    InvalidPath(String),
}
