pub mod client;
pub mod errors;
pub mod models;

// Re-export key items for easier use
pub use client::get_presentation_sa;
pub use errors::{Result, SlidesApiError};
pub use models::presentation::Presentation; // Re-export core Presentation model

// features
pub mod diff;
pub use diff::comparer::ComparerBuilder;
