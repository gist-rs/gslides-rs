//! Utility functions for SVG conversion, including escaping, unit conversion, color formatting,
//! transform application, and model helper traits.

use super::{constants::*, error::Result};
use crate::models::{
    colors::{ColorScheme, OpaqueColor, OpaqueColorContent, OptionalColor},
    common::{AffineTransform, Dimension, Unit},
    elements::PageElementKind,
    page_properties::PageBackgroundFill,
    shape::Shape, // Keep only if GetColorScheme stays here
};
use std::fmt::Write;

// --- Text Escaping ---

/// Escapes special XML characters (`&`, `<`, `>`) for use in SVG text content.
pub fn escape_svg_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escapes special XML characters (`&`, `<`, `>`) for use in HTML content within SVG `<foreignObject>`.
/// Note: Attribute values in HTML might need further escaping (e.g., quotes), but this is for content.
pub fn escape_html_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// --- Unit Conversion ---

/// Converts an optional `Dimension` to points (pt).
/// Returns 0.0 if the dimension is `None`, has no magnitude, or uses an unknown/unsupported unit.
pub fn dimension_to_pt(dim: Option<&Dimension>) -> f64 {
    match dim {
        Some(d) => {
            let magnitude = d.magnitude.unwrap_or(0.0);
            match d.unit.as_ref() {
                Some(Unit::Pt) => magnitude,
                Some(Unit::Emu) => magnitude / EMU_PER_PT,
                // Add other units like Inches if necessary:
                // Some(Unit::Inch) => magnitude * PT_PER_INCH,
                _ => 0.0, // Treat unspecified or unknown units as 0 points
            }
        }
        None => 0.0, // Treat missing Dimension as 0 points
    }
}

// --- Color Formatting ---

/// Converts an `OpaqueColor` to an SVG color string (e.g., `#RRGGBB`).
/// Resolves `ThemeColor` types using the provided `ColorScheme` if available.
/// Falls back to `DEFAULT_TEXT_COLOR` if resolution fails or the color is inherently invalid.
///
/// # Arguments
/// * `color_opt` - An optional reference to the `OpaqueColor` to format.
/// * `color_scheme` - An optional reference to the slide's `ColorScheme` for theme color lookup.
///
/// # Returns
/// An SVG color string (e.g., "#FF0000") or a fallback color.
pub fn format_color(color_opt: Option<&OpaqueColor>, color_scheme: Option<&ColorScheme>) -> String {
    match color_opt {
        Some(opaque_color) => match &opaque_color.color_kind {
            OpaqueColorContent::RgbColor(rgb) => {
                let r = (rgb.red.unwrap_or(0.0) * 255.0).round() as u8;
                let g = (rgb.green.unwrap_or(0.0) * 255.0).round() as u8;
                let b = (rgb.blue.unwrap_or(0.0) * 255.0).round() as u8;
                format!("#{:02x}{:02x}{:02x}", r, g, b)
            }
            OpaqueColorContent::ThemeColor(theme_color_type) => {
                // Attempt to resolve theme color using the provided scheme
                if let Some(scheme) = color_scheme {
                    if let Some(theme_pair) = scheme
                        .colors
                        .iter()
                        .find(|pair| pair.theme_color_type == *theme_color_type)
                    {
                        // Found the matching theme color pair. Format its RGB value.
                        // Construct a temporary OpaqueColor to reuse the RGB formatting logic.
                        let resolved_opaque_color = OpaqueColor {
                            color_kind: OpaqueColorContent::RgbColor(theme_pair.color.clone()),
                        };
                        // Recursively call format_color, but pass None for scheme to prevent loops
                        // in case a theme color somehow points back to another theme color (unlikely).
                        return format_color(Some(&resolved_opaque_color), None);
                    }
                }
                // Fallback if scheme is missing or color type not found in the scheme
                eprintln!("Warning: Theme color {:?} not found in scheme or scheme missing. Falling back to default.", theme_color_type);
                DEFAULT_TEXT_COLOR.to_string()
            }
        },
        None => DEFAULT_TEXT_COLOR.to_string(), // Fallback if OpaqueColor itself is missing
    }
}

/// Converts an `OptionalColor` (often used for backgrounds/foregrounds) to SVG fill/opacity attributes.
/// `OptionalColor` wraps an `Option<OpaqueColor>`; if the inner option is `None`, it signifies transparency.
/// If the `OptionalColor` itself is `None`, it typically means "use default" (often black).
/// Resolves theme colors using the provided `ColorScheme`.
///
/// # Arguments
/// * `optional_color` - An optional reference to the `OptionalColor`.
/// * `color_scheme` - An optional reference to the slide's `ColorScheme`.
///
/// # Returns
/// A tuple containing the SVG `fill` attribute value (e.g., "#RRGGBB", "none")
/// and the SVG `fill-opacity` attribute value ("1" or "0").
pub fn format_optional_color(
    optional_color: Option<&OptionalColor>,
    color_scheme: Option<&ColorScheme>,
) -> (String, String) {
    match optional_color {
        Some(opt_color) => {
            // Check if the optional color actually contains an opaque color definition
            match &opt_color.opaque_color {
                Some(opaque_color) => {
                    // An opaque color is defined, format it (handles RGB and ThemeColor lookup)
                    let color_hex = format_color(Some(opaque_color), color_scheme);
                    // Assume full opacity unless alpha is introduced later in OpaqueColor model
                    (color_hex, "1".to_string())
                }
                // The opaque_color field was explicitly null/None in the API response, meaning transparent.
                None => ("none".to_string(), "0".to_string()),
            }
        }
        // The OptionalColor structure itself was absent from the parent object.
        // Default to opaque black as a fallback (or could be context-dependent).
        None => (DEFAULT_TEXT_COLOR.to_string(), "1".to_string()),
    }
}

// --- Transformation ---

/// Applies an `AffineTransform` to an SVG element's `transform` attribute.
/// Converts translation units (assumed to be EMU if not specified otherwise in the transform) to points.
///
/// # Arguments
/// * `transform` - An optional reference to the `AffineTransform`.
/// * `svg_attrs` - A mutable string to which the `transform="matrix(...)"` attribute will be appended.
///
/// # Returns
/// A `Result` containing a tuple `(tx_pt, ty_pt, width_pt)` representing the translation in points
/// and a placeholder width (width calculation is usually separate). Returns an error on formatting failure.
pub fn apply_transform(
    transform: Option<&AffineTransform>,
    svg_attrs: &mut String,
) -> Result<(f64, f64)> {
    let mut tx_pt = 0.0;
    let mut ty_pt = 0.0;

    if let Some(tf) = transform {
        // Use provided scale/shear values, defaulting to identity transform components if missing
        let scale_x = tf.scale_x.unwrap_or(1.0);
        let scale_y = tf.scale_y.unwrap_or(1.0);
        let shear_x = tf.shear_x.unwrap_or(0.0);
        let shear_y = tf.shear_y.unwrap_or(0.0);

        // Translations require unit conversion. Use the unit specified *in the transform*
        // or default to EMU if the transform unit itself is missing (based on API typical behavior).
        let translate_unit = tf.unit.as_ref().cloned().unwrap_or(Unit::Emu);

        tx_pt = dimension_to_pt(Some(&Dimension {
            magnitude: Some(tf.translate_x.unwrap_or(0.0)), // Default magnitude to 0 if missing
            unit: Some(translate_unit.clone()),
        }));
        ty_pt = dimension_to_pt(Some(&Dimension {
            magnitude: Some(tf.translate_y.unwrap_or(0.0)), // Default magnitude to 0 if missing
            unit: Some(translate_unit),
        }));

        // Construct the SVG transform matrix: matrix(a, b, c, d, e, f)
        // Where: a=scaleX, b=shearY, c=shearX, d=scaleY, e=translateX, f=translateY
        write!(
            svg_attrs,
            r#" transform="matrix({} {} {} {} {} {})""#,
            scale_x, shear_y, shear_x, scale_y, tx_pt, ty_pt
        )?;
    } else {
        // No transform provided, leave svg_attrs unchanged and return (0, 0) translation.
    }
    Ok((tx_pt, ty_pt)) // Return translation in points
}

// --- Model Helper Traits ---

/// A helper trait to safely access the `Shape` data within a `PageElementKind`.
pub(crate) trait AsShape {
    /// Returns an `Option<&Shape>` if the `PageElementKind` is a `Shape`.
    fn as_shape(&self) -> Option<&Shape>;
}

impl AsShape for PageElementKind {
    fn as_shape(&self) -> Option<&Shape> {
        match self {
            PageElementKind::Shape(s) => Some(s),
            _ => None,
        }
    }
}

/// A helper trait (potentially unnecessary if logic is simple) to extract ColorScheme.
/// NOTE: This implementation assumes ColorScheme is NOT typically within PageBackgroundFill.
/// The primary location is PageProperties. It's kept here for structural context but might be removed.
#[allow(dead_code)]
pub(crate) trait GetColorScheme {
    fn get_color_scheme(&self) -> Option<&ColorScheme>;
}

impl GetColorScheme for PageBackgroundFill {
    fn get_color_scheme(&self) -> Option<&ColorScheme> {
        // Currently, the API schema places ColorScheme in PageProperties, not directly
        // within the background fill types (SolidFill, etc.).
        // If the schema changes, this implementation would need updating.
        None
    }
}
