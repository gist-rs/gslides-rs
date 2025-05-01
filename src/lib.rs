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

#[cfg(test)]
mod tests {
    use crate::models::presentation::Presentation;
    use std::fs;
    // Remove Write import if not used directly here
    // use std::io::Write;

    #[test]
    fn test_svg_conversion_from_json() {
        // Rename test if needed
        // Initialize logger for this test run
        // Use try_init to avoid panic if logger is already initialized (e.g., by another test)
        // Set default level to info, but allow overriding with RUST_LOG
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info) // Default level
            .parse_default_env() // Allow RUST_LOG override
            .try_init();

        // Load a sample presentation JSON (replace with your actual path)
        let json_path = "changed_presentation.json"; // Use the JSON with the placeholder issue

        let json_string =
            fs::read_to_string(json_path).expect("Should have been able to read the file");

        let presentation: Presentation = match serde_json::from_str(&json_string) {
            Ok(p) => p,
            Err(e) => {
                log::error!("Deserialization failed: {}", e); // Use log::error
                let snippet_len = json_string.len().min(500);
                log::error!("JSON Snippet:\n{}", &json_string[..snippet_len]); // Use log::error
                panic!("Failed to deserialize presentation JSON");
            }
        };

        log::info!("Attempting SVG conversion..."); // Use log::info
        match crate::converters::svg::convert_presentation_to_svg(&presentation) {
            Ok(svg_slides) => {
                log::info!(
                    "SVG Conversion successful. Got {} slides.",
                    svg_slides.len()
                ); // Use log::info
                assert!(!svg_slides.is_empty(), "Expected at least one SVG slide");

                // Optional: Write SVG files for inspection
                for (i, svg_content) in svg_slides.iter().enumerate() {
                    let output_path = format!("slide_{}.svg", i + 1);
                    match fs::write(&output_path, svg_content) {
                        Ok(_) => log::info!("Written SVG to {}", output_path), // Use log::info
                        Err(e) => log::error!("Failed to write SVG to {}: {}", output_path, e), // Use log::error
                    }
                    // Print the first slide's SVG for quick check in logs
                    if i == 0 {
                        log::debug!(
                            "--- SVG Slide 1 ---:\n{}\n--- End SVG Slide 1 ---",
                            svg_content
                        ); // Use log::debug
                    }
                }
            }
            Err(e) => {
                log::error!("SVG Conversion failed: {}", e); // Use log::error
                panic!("SVG Conversion failed");
            }
        }
    }

    // Keep the markdown test if needed
    #[test]
    fn test_markdown_extraction_from_json() {
        // ... (markdown test code, maybe add logging init here too if run separately) ...
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .parse_default_env()
            .try_init(); // Add logger init

        let json_path = "changed_presentation.json";
        // ... rest of markdown test ...
        log::info!("--- Extracted Markdown ---"); // Use log::info
                                                  // ...
    }
}
