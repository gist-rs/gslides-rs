use serde::{Deserialize, Serialize};

/// An RGB color.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RgbColor {
    /// The red component of the color, from 0.0 to 1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub red: Option<f32>,
    /// The green component of the color, from 0.0 to 1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub green: Option<f32>,
    /// The blue component of the color, from 0.0 to 1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blue: Option<f32>,
}

/// Theme color types.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThemeColorType {
    ThemeColorTypeUnspecified,
    Dark1,
    Light1,
    Dark2,
    Light2,
    Accent1,
    Accent2,
    Accent3,
    Accent4,
    Accent5,
    Accent6,
    Hyperlink,
    FollowedHyperlink,
    Text1,
    Background1,
    Text2,
    Background2,
}

// --- REVERTED OpaqueColor Definition ---
/// Enum representing the content of an OpaqueColor union.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OpaqueColorContent {
    /// An opaque RGB color. JSON key: "rgbColor"
    RgbColor(RgbColor),
    /// An opaque theme color. JSON key: "themeColor"
    ThemeColor(ThemeColorType),
}

/// A themeable solid color value. Contains either an RGB color or a theme color.
/// The JSON representation uses the field name ("rgbColor" or "themeColor") as the key.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#OpaqueColor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpaqueColor {
    /// The specific color choice. Uses flatten to represent the union based on JSON key.
    #[serde(flatten)]
    pub color_kind: OpaqueColorContent,
}
// --- END REVERTED Definition ---

/// A color that can either be fully opaque or fully transparent.
/// If opaque, the `opaque_color` field is set. If transparent, the field is absent.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#OptionalColor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalColor {
    /// If set, this will be used as an opaque color. If unset, the color is rendered as transparent.
    // This now uses the OpaqueColor struct again.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opaque_color: Option<OpaqueColor>,
}

/// A pair mapping a theme color type to the concrete color it represents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColorPair {
    /// The type of the theme color.
    #[serde(rename = "type")]
    pub theme_color_type: ThemeColorType,
    /// The concrete RGB color corresponding to the theme color type above.
    pub color: RgbColor,
}

/// A color scheme defines the mapping of theme color types to concrete colors used on a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorScheme {
    /// The ThemeColorType and corresponding concrete color pairs.
    pub colors: Vec<ThemeColorPair>,
}
