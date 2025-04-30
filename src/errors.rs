use thiserror::Error;

/// Represents errors that can occur when interacting with the Google Slides API client.
#[derive(Error, Debug)]
pub enum SlidesApiError {
    /// Error originating from the underlying HTTP client (`reqwest`).
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    /// Error occurred during the deserialization of the JSON response from the API.
    #[error("Failed to deserialize JSON response: {0}")]
    JsonDeserialization(#[from] serde_json::Error),

    /// An error reported by the Google Slides API itself (e.g., 4xx or 5xx status code).
    #[error("API returned an error: Status {status}, Message: {message}")]
    ApiError {
        status: reqwest::StatusCode,
        message: String,
    },

    /// An error related to authentication or authorization setup.
    #[error("Authentication setup/configuration error: {0}")]
    AuthSetupError(String),

    /// An error specifically from the authentication library (yup-oauth2) during token fetching/validation.
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(feature = "yup-oauth2")]
    #[error("Authentication library error: {0}")]
    AuthLibError(#[from] yup_oauth2::Error),

    /// An error indicating invalid input was provided to a client function.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// An error related to reading environment variables.
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),

    /// An I/O error occurred, often related to file access (e.g., reading the service account key).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error), // Added this variant

    /// An unexpected or unknown error occurred.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// A type alias for `Result<T, SlidesApiError>` for convenience within the crate.
pub type Result<T> = std::result::Result<T, SlidesApiError>;
