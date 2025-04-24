use serde::{Deserialize, Serialize};

/// Specifies a unit of length.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/Dimension#Unit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Unit {
    /// The units are unknown. Should not be used.
    #[serde(rename = "UNIT_UNSPECIFIED")] // Explicit rename to match potential JSON value
    UnitUnspecified,
    /// An English Metric Unit (EMU). 1 EMU = 1/914400 inch = 1/360000 cm.
    Emu,
    /// A point (pt). 1 pt = 1/72 inch.
    Pt,
}

/// A magnitude in a specific unit.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/Dimension
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dimension {
    /// The magnitude.
    pub magnitude: Option<f64>,
    /// The units for magnitude.
    pub unit: Option<Unit>,
}

/// A width and height.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/Size
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Size {
    /// The width. Missing width does not inherit from parents.
    pub width: Option<Dimension>,
    /// The height. Missing height does not inherit from parents.
    pub height: Option<Dimension>,
}

/// AffineTransform uses a 3x3 matrix with an implied last row of [ 0 0 1 ]
/// to transform source coordinates (x,y) into destination coordinates (x', y').
/// This message is composed of the six matrix elements that can be manipulated.
///
/// Formula:
/// x' = scaleX * x + shearX * y + translateX;
/// y' = shearY * x + scaleY * y + translateY;
///
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/AffineTransform
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AffineTransform {
    /// The X coordinate scaling element.
    pub scale_x: Option<f64>,
    /// The Y coordinate scaling element.
    pub scale_y: Option<f64>,
    /// The X coordinate shearing element.
    pub shear_x: Option<f64>,
    /// The Y coordinate shearing element.
    pub shear_y: Option<f64>,
    /// The X coordinate translation element.
    pub translate_x: Option<f64>,
    /// The Y coordinate translation element.
    pub translate_y: Option<f64>,
    /// The units for the translation elements.
    pub unit: Option<Unit>,
}
