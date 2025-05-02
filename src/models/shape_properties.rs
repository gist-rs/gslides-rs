use serde::{Deserialize, Serialize};

// Import necessary types from other modules
use crate::models::colors::OpaqueColor; // Assuming struct+flatten version is correct
use crate::models::common::{AffineTransform, Dimension};
use crate::models::link::Link;
use crate::models::picture::StretchedPictureFill;

// --- Enums (AutofitType, PropertyState, DashStyle, RectanglePosition, ContentAlignment) ---
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PropertyState {
    Rendered,
    NotRendered,
    #[default]
    Inherit,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DashStyle {
    #[default]
    DashStyleUnspecified,
    Solid,
    Dot,
    Dash,
    DashDot,
    LongDash,
    LongDashDot,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RectanglePosition {
    #[default]
    RectanglePositionUnspecified,
    TopLeft,
    TopCenter,
    TopRight,
    LeftCenter,
    Center,
    RightCenter,
    BottomLeft,
    BottomCenter,
    BottomRight,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContentAlignment {
    #[default]
    ContentAlignmentUnspecified,
    ContentAlignmentUnsupported,
    Top,
    Middle,
    Bottom,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AutofitType {
    #[default]
    AutofitTypeUnspecified,
    None,
    TextAutofit,
    ShapeAutofit,
}

// --- Structs (Autofit, SolidFill, OutlineFillContent, OutlineFill, ShapeBackgroundFillContent, ShapeBackgroundFill, Shadow) ---
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Autofit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autofit_type: Option<AutofitType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_spacing_reduction: Option<f32>,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolidFill {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<OpaqueColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f32>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutlineFillContent {
    SolidFill(SolidFill),
}
impl Default for OutlineFillContent {
    fn default() -> Self {
        OutlineFillContent::SolidFill(SolidFill::default())
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineFill {
    #[serde(flatten)]
    pub fill_kind: OutlineFillContent,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapeBackgroundFillContent {
    SolidFill(SolidFill),
    StretchedPictureFill(StretchedPictureFill),
}
impl Default for ShapeBackgroundFillContent {
    fn default() -> Self {
        ShapeBackgroundFillContent::SolidFill(SolidFill::default())
    }
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeBackgroundFill {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_state: Option<PropertyState>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_kind: Option<ShapeBackgroundFillContent>,
}
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shadow {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment: Option<RectanglePosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<AffineTransform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blur_radius: Option<Dimension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<OpaqueColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotate_with_shape: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_state: Option<PropertyState>,
}

// --- Outline Struct ---
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "camelCase")]
pub struct Outline {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline_fill: Option<OutlineFill>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<Dimension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dash_style: Option<DashStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_state: Option<PropertyState>,
}

// --- ShapeProperties Struct (Restored) ---
/// The properties of a Shape element. Uses serde(default) to handle missing fields.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] // Added Default, PartialEq back
#[serde(rename_all = "camelCase")]
#[serde(default)] // Tells serde to use Default for missing fields
pub struct ShapeProperties {
    // --- Fields are NOT Option<> anymore ---
    pub shape_background_fill: ShapeBackgroundFill,
    pub outline: Outline,
    pub shadow: Shadow,
    pub link: Link,                          // Make sure Link derives/impls Default
    pub content_alignment: ContentAlignment, // Make sure ContentAlignment has #[default] variant
    pub autofit: Autofit,
}
