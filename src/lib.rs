#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "yup-oauth2")]
pub mod client;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "yup-oauth2")]
pub use client::get_presentation_sa;

pub mod converters;
pub mod errors;
pub mod models;

pub use converters::markdown;
pub use errors::{Result, SlidesApiError};
pub use models::presentation::Presentation;

// features
pub mod diff;
pub use diff::comparer::ComparerBuilder;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello {name} from Rust!!")
}
