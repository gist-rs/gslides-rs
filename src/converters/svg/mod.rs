//! Converts Google Slides presentations to Scalable Vector Graphics (SVG) format.
//!
//! This module provides the main entry point `convert_presentation_to_svg` and organizes
//! the conversion logic into submodules for clarity and maintainability.
//!
//! # Current Features & Limitations:
//! *   Converts slides to individual SVG files.
//! *   Handles basic shapes (text content only, no geometry yet), tables (via HTML `foreignObject`), lines, and images.
//! *   Supports text styling (font, size, color, bold, italic, underline, etc.).
//! *   Handles text alignment (start, center, end).
//! *   Resolves theme colors based on slide/layout/master hierarchy.
//! *   Handles placeholder inheritance for text styles.
//! *   Renders placeholders for unsupported element types (Video, WordArt, etc.).
//! *   Limited support for complex features like gradients, advanced line endings/arrows, precise text wrapping, charts, animations.

// Declare the submodules
mod constants;
mod elements;
mod error;
mod structure;
mod text;
mod utils;

// Re-export the main error type and result alias for consumers of this module
pub use error::{Result, SvgConversionError};

// Import necessary items from submodules and models
use crate::models::presentation::Presentation;
use structure::{build_lookup_maps, convert_slide_to_svg}; // Import internal functions

/// Converts a Google Slides `Presentation` object into a vector of SVG strings,
/// with each string representing one slide.
///
/// Builds necessary lookup maps for efficient processing and then iterates through
/// each slide, calling the slide conversion logic.
///
/// # Arguments
/// * `presentation` - A reference to the `Presentation` object fetched from the Google Slides API.
///
/// # Returns
/// A `Result<Vec<String>>` containing the SVG content for each slide upon success,
/// or an `SvgConversionError` if a critical error occurs during conversion. Errors
/// during individual slide conversion will halt the process and return the error.
pub fn convert_presentation_to_svg(presentation: &Presentation) -> Result<Vec<String>> {
    let mut svg_slides = Vec::new();

    // 1. Build lookup maps for efficient access to layouts, masters, and elements.
    let (layouts_map, masters_map, elements_map) = build_lookup_maps(presentation);

    // 2. Iterate through slides and convert each one.
    if let Some(slides) = &presentation.slides {
        svg_slides.reserve(slides.len()); // Pre-allocate vector capacity

        for (index, slide) in slides.iter().enumerate() {
            // Convert a single slide using the pre-built context.
            match convert_slide_to_svg(
                slide,
                presentation.page_size.as_ref(), // Pass presentation size context
                &layouts_map,
                &masters_map,
                &elements_map,
            ) {
                Ok(svg_content) => svg_slides.push(svg_content),
                Err(e) => {
                    // Log the error and return immediately, halting the conversion.
                    // Consider alternative strategies like collecting errors or skipping problematic slides.
                    eprintln!(
                        "Error converting slide {} (ID: {}): {}",
                        index + 1,
                        slide.object_id,
                        e
                    );
                    return Err(SvgConversionError::Internal(format!(
                        "Failed to convert slide {} (ID: {}): {}",
                        index + 1,
                        slide.object_id,
                        e
                    )));
                }
            }
        }
    } else {
        // Presentation has no slides, return an empty vector.
        // Optionally, could return an error or warning if this is unexpected.
        eprintln!("Warning: Presentation has no slides to convert.");
    }

    // 3. Return the collected SVG strings.
    Ok(svg_slides)
}
