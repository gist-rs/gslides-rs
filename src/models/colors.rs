use serde::{Deserialize, Serialize};

/// An RGB color.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#RgbColor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RgbColor {
    /// The red component of the color, from 0.0 to 1.0.
    pub red: Option<f32>, // API spec uses 'number', f32 seems appropriate
    /// The green component of the color, from 0.0 to 1.0.
    pub green: Option<f32>,
    /// The blue component of the color, from 0.0 to 1.0.
    pub blue: Option<f32>,
}

/// Theme color types. A PageProperties contains a ColorScheme that defines the
/// mapping of these types to concrete colors.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#ThemeColorType
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ThemeColorType {
    /// Unspecified theme color. Should not be used.
    ThemeColorTypeUnspecified,
    /// First dark color.
    Dark1,
    /// First light color.
    Light1,
    /// Second dark color.
    Dark2,
    /// Second light color.
    Light2,
    /// First accent color.
    Accent1,
    /// Second accent color.
    Accent2,
    /// Third accent color.
    Accent3,
    /// Fourth accent color.
    Accent4,
    /// Fifth accent color.
    Accent5,
    /// Sixth accent color.
    Accent6,
    /// Hyperlink color.
    Hyperlink,
    /// Visited hyperlink color.
    FollowedHyperlink,
    /// First text color.
    Text1,
    /// First background color.
    Background1,
    /// Second text color.
    Text2,
    /// Second background color.
    Background2,
}

/// A themeable solid color value. Contains either an RGB color or a theme color.
/// The JSON representation uses the field name ("rgbColor" or "themeColor") as the key.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#OpaqueColor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OpaqueColorContent {
    /// An opaque RGB color.
    RgbColor(RgbColor),
    /// An opaque theme color.
    ThemeColor(ThemeColorType),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpaqueColor {
    /// The specific color choice. Uses flatten to represent the union based on JSON key.
    #[serde(flatten)]
    pub color_kind: OpaqueColorContent,
}

/// A color that can either be fully opaque or fully transparent.
/// If opaque, the `opaque_color` field is set. If transparent, the field is absent.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#OptionalColor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalColor {
    /// If set, this will be used as an opaque color. If unset, the color is rendered as transparent.
    pub opaque_color: Option<OpaqueColor>,
}

/// A pair mapping a theme color type to the concrete color it represents.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#ThemeColorPair
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColorPair {
    /// The type of the theme color.
    #[serde(rename = "type")]
    pub theme_color_type: ThemeColorType, // Renamed to avoid conflict with Rust keyword

    /// The concrete color corresponding to the theme color type above.
    pub color: RgbColor, // API shows RgbColor here, not OpaqueColor
}

/// A color scheme defines the mapping of theme color types to concrete colors used on a page.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#ColorScheme
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorScheme {
    /// The ThemeColorType and corresponding concrete color pairs.
    pub colors: Vec<ThemeColorPair>,
}
