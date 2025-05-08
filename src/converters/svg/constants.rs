//! Defines constants used throughout the SVG conversion process.

// Conversion factors (assuming 96 DPI for px equivalence, but primarily using pt)
pub const PT_PER_INCH: f64 = 72.0;
pub const EMU_PER_INCH: f64 = 914400.0;
// EMU (English Metric Unit) per Point (standard 72 DPI)
pub const EMU_PER_PT: f64 = EMU_PER_INCH / PT_PER_INCH; // Approx 12700
// EMU per SVG User Unit (based on common 96 DPI assumption for root SVG size)
pub const EMU_PER_SVG_UNIT: f64 = 9525.0;

// Default values used when specific properties are missing or cannot be resolved.
pub const DEFAULT_FONT_SIZE_PT: f64 = 11.0; // Default fallback font size in points
pub const DEFAULT_FONT_FAMILY: &str = "Arial"; // Default fallback font family
pub const DEFAULT_TEXT_COLOR: &str = "#000000"; // Default text/stroke color (black)
pub const DEFAULT_BACKGROUND_COLOR: &str = "#ffffff"; // Default page background (white)
