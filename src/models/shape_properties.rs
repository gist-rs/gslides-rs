use crate::models::colors::OpaqueColor;
use crate::models::common::{AffineTransform, Dimension};
use crate::models::link::Link;
use crate::models::picture::StretchedPictureFill;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PropertyState {
    Rendered,
    NotRendered,
    #[default]
    Inherit,
} // Default is Inherit

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)] // Added Default
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)] // Added Default
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContentAlignment {
    #[default]
    ContentAlignmentUnspecified,
    ContentAlignmentUnsupported,
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AutofitType {
    #[default]
    AutofitTypeUnspecified,
    None,
    TextAutofit,
    ShapeAutofit,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "camelCase")]
pub struct Autofit {
    pub autofit_type: Option<AutofitType>, // Keep Options inside if they truly can be null
    pub font_scale: Option<f32>,
    pub line_spacing_reduction: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "camelCase")]
pub struct SolidFill {
    pub color: Option<OpaqueColor>, // Keep Option
    pub alpha: Option<f32>,         // Keep Option
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // Enum variant, no Default derive needed
#[serde(rename_all = "camelCase")]
pub enum OutlineFillContent {
    SolidFill(SolidFill),
}
// Implement Default manually for the enum if needed, or default the outer struct field
impl Default for OutlineFillContent {
    fn default() -> Self {
        OutlineFillContent::SolidFill(SolidFill::default())
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "camelCase")]
pub struct OutlineFill {
    #[serde(flatten)]
    pub fill_kind: OutlineFillContent,
} // Defaults via OutlineFillContent::default()

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // Enum variant, no Default derive needed
#[serde(rename_all = "camelCase")]
pub enum ShapeBackgroundFillContent {
    SolidFill(SolidFill),
    StretchedPictureFill(StretchedPictureFill),
}
// Manual default needed if Option<ShapeBackgroundFillContent> becomes ShapeBackgroundFillContent
impl Default for ShapeBackgroundFillContent {
    fn default() -> Self {
        ShapeBackgroundFillContent::SolidFill(SolidFill::default())
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "camelCase")]
pub struct ShapeBackgroundFill {
    pub property_state: Option<PropertyState>, // Keep Option
    #[serde(flatten)]
    pub fill_kind: Option<ShapeBackgroundFillContent>, // Keep Option
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "camelCase")]
pub struct Outline {
    pub outline_fill: Option<OutlineFill>,     // Keep Option
    pub weight: Option<Dimension>,             // Keep Option
    pub dash_style: Option<DashStyle>,         // Keep Option
    pub property_state: Option<PropertyState>, // Keep Option
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "camelCase")]
pub struct Shadow {
    #[serde(rename = "type")]
    pub shadow_type: Option<String>,
    pub alignment: Option<RectanglePosition>,
    pub transform: Option<AffineTransform>, // Needs AffineTransform to impl Default
    pub blur_radius: Option<Dimension>,     // Needs Dimension to impl Default
    pub color: Option<OpaqueColor>,         // Needs OpaqueColor to impl Default
    pub alpha: Option<f32>,
    pub rotate_with_shape: Option<bool>,
    pub property_state: Option<PropertyState>,
}
// Note: Need to ensure AffineTransform, Dimension, OpaqueColor also derive/impl Default in their respective files.

// --- ShapeProperties Struct (Using serde(default)) ---
/// The properties of a Shape element. Uses serde(default) to handle missing fields.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)] // Added Default
#[serde(rename_all = "camelCase")]
#[serde(default)] // <<< Add this attribute
pub struct ShapeProperties {
    // --- Fields are NOT Option<> anymore ---
    pub shape_background_fill: ShapeBackgroundFill,
    pub outline: Outline,
    pub shadow: Shadow,
    pub link: Link,                          // Make sure Link derives/impls Default
    pub content_alignment: ContentAlignment, // Make sure ContentAlignment has #[default] variant
    pub autofit: Autofit,
}
