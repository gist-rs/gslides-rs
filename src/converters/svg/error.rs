use thiserror::Error;

/// Errors that can occur during the Google Slides to SVG conversion process.
#[derive(Error, Debug)]
pub enum SvgConversionError {
    #[error("Formatting error during SVG generation: {0}")]
    FormatError(#[from] std::fmt::Error),
    #[error("Missing expected data necessary for conversion: {0}")]
    MissingData(String),
    #[error("Feature present in Google Slides is not supported by this SVG converter: {0}")]
    Unsupported(String),
    #[error("An internal error occurred during conversion: {0}")]
    Internal(String),
    // Consider adding more specific errors if needed, e.g., IoError if reading external resources
}

/// A specialized Result type for SVG conversion operations.
pub type Result<T> = std::result::Result<T, SvgConversionError>;
