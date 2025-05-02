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
// Function to setup logging for WASM (call this once from JS)
#[wasm_bindgen(start)]
pub fn init_wasm_logging() {
    // Only install the panic hook in release builds
    #[cfg(not(debug_assertions))]
    console_error_panic_hook::set_once();
    // Use `fern` or `console_log` for logging
    // Example using `console_log`:
    let _ = console_log::init_with_level(log::Level::Info); // Or Level::Debug, Warn, Error
    log::info!("WASM logging initialized.");
}

/// Converts a Google Slides presentation JSON string into an SVG string (first slide only).
/// Returns the SVG content of the first slide upon success.
/// Returns a JsValue error object on failure (deserialization or conversion error).
#[wasm_bindgen]
pub fn convert_json_to_svg(presentation_json_string: &str) -> std::result::Result<String, JsValue> {
    log::info!("Received presentation JSON, attempting deserialization...");

    // 1. Deserialize the JSON string into a Presentation object
    let presentation: Presentation =
        serde_json::from_str(presentation_json_string).map_err(|e| {
            let error_msg = format!("JSON Deserialization Error: {}", e);
            log::error!("{}", error_msg);
            JsValue::from_str(&error_msg)
        })?;

    log::info!("Deserialization successful. Starting SVG conversion...");

    // 2. Convert the Presentation object to SVG slides
    let svg_slides = converters::svg::convert_presentation_to_svg(&presentation).map_err(|e| {
        let error_msg = format!("SVG Conversion Error: {}", e);
        log::error!("{}", error_msg);
        JsValue::from_str(&error_msg)
    })?;

    log::info!(
        "SVG Conversion successful. Found {} slides.",
        svg_slides.len()
    );

    // 3. Return the first slide's SVG content, or an error if there are no slides
    if let Some(first_slide_svg) = svg_slides.into_iter().next() {
        log::info!("Returning SVG for the first slide.");
        Ok(first_slide_svg)
    } else {
        let error_msg = "SVG Conversion succeeded, but no slides were found in the output.";
        log::warn!("{}", error_msg); // Log as warning as conversion itself didn't fail
        Err(JsValue::from_str(error_msg))
    }
}
