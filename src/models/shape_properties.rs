// src/models/shape_properties.rs

use serde::{Deserialize, Serialize};

// Import necessary types from other modules
use crate::models::colors::OpaqueColor;
use crate::models::common::{AffineTransform, Dimension};
use crate::models::link::Link;
use crate::models::picture::StretchedPictureFill;

// --- Enums specific to Shape Properties ---

/// Describes the state of a property for rendering.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PropertyState {
    /// The property is explicitly rendered using the supplied values.
    Rendered,
    /// The property is not rendered. The appearance depends on inheritance or defaults.
    NotRendered,
    /// The property state is inherited from the parent placeholder. (Implicit default)
    Inherit, // Useful for modeling, though API uses absence of RENDERED/NOT_RENDERED
}

/// The dash style of an outline or line. Corresponds to ECMA-376 ST_PresetLineDashVal values.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#DashStyle
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DashStyle {
    /// Unspecified dash style. Rendered as SOLID.
    DashStyleUnspecified,
    /// Solid line.
    Solid,
    /// Dotted line.
    Dot,
    /// Dashed line.
    Dash,
    /// Alternating dash/dot line.
    DashDot,
    /// Long dashed line.
    LongDash,
    /// Alternating long dash/dot line.
    LongDashDot,
}

/// The alignment point for a shadow or other rectangular positioning, relative to the shape's bounds.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#RectanglePosition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RectanglePosition {
    /// Unspecified position. Behavior may depend on context.
    RectanglePositionUnspecified,
    /// Top left corner.
    TopLeft,
    /// Top center.
    TopCenter,
    /// Top right corner.
    TopRight,
    /// Left center.
    LeftCenter,
    /// Center.
    Center,
    /// Right center.
    RightCenter,
    /// Bottom left corner.
    BottomLeft,
    /// Bottom center.
    BottomCenter,
    /// Bottom right corner.
    BottomRight,
}

/// The content alignment for text within a Shape or TableCell. Corresponds to ECMA-376 ST_TextAnchoringType.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#ContentAlignment
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContentAlignment {
    /// Unspecified content alignment. Inherited from parent placeholder if it exists, otherwise defaults to MIDDLE.
    ContentAlignmentUnspecified,
    /// An unsupported content alignment.
    ContentAlignmentUnsupported, // Should not typically be used or encountered.
    /// Align content to the top of the content holder.
    Top,
    /// Align content to the middle of the content holder.
    Middle,
    /// Align content to the bottom of the content holder.
    Bottom,
}

/// The autofit type of a shape. Determines whether and how the shape text is automatically resized.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#AutofitType
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AutofitType {
    /// Unspecified autofit type. Inherited from parent placeholder if exists.
    AutofitTypeUnspecified,
    /// Autofit is not applied. Text may overflow the shape.
    None,
    /// Shrinks text font size on overflow.
    TextAutofit,
    /// Resizes the shape to fit the text. This is the default for new shapes.
    ShapeAutofit,
}

/// The autofit properties of a Shape. Only set for shapes that allow text.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#Autofit
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Autofit {
    /// The autofit type of the shape.
    pub autofit_type: Option<AutofitType>,
    /// Output only. The font scale applied to the shape. Default is 1.0 for NONE/SHAPE_AUTOFIT.
    /// For TEXT_AUTOFIT, this * `fontSize` gives the rendered size. Read-only.
    pub font_scale: Option<f32>, // Read-only
    /// Output only. The line spacing reduction applied. Default is 0 for NONE/SHAPE_AUTOFIT.
    /// For TEXT_AUTOFIT, this subtracted from `lineSpacing` gives the rendered spacing. Read-only.
    pub line_spacing_reduction: Option<f32>, // Read-only
}

// --- Fill and Outline Structs ---

/// A solid color fill. Used for shape backgrounds, outlines, text, etc.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#SolidFill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolidFill {
    /// The color value of the solid fill.
    pub color: Option<OpaqueColor>,
    /// The alpha value (opacity) of the color. Defaults to 1.0 (fully opaque).
    /// Value must be in the interval [0.0, 1.0].
    pub alpha: Option<f32>,
}

/// The fill properties for an outline. Only solid fill is currently supported.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#OutlineFill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutlineFillContent {
    /// Solid color fill.
    SolidFill(SolidFill),
    // PatternFill, GradientFill etc. are not supported for outlines via API.
}

/// Represents the fill style of an outline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlineFill {
    /// The specific fill type (currently only SolidFill). Uses flatten for the union key.
    #[serde(flatten)]
    pub fill_kind: OutlineFillContent,
}

/// The fill properties for a shape background. Can be solid or stretched picture.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#ShapeBackgroundFill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapeBackgroundFillContent {
    /// Solid color fill.
    SolidFill(SolidFill),
    /// Stretched picture fill. Only supported for shapes with rectangular geometry.
    StretchedPictureFill(StretchedPictureFill),
}

/// Represents the background fill of a shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeBackgroundFill {
    /// The background fill property state. If `NOT_RENDERED`, the shape has no fill.
    /// If unspecified or `RENDERED`, the fill is described by `fill_kind`.
    pub property_state: Option<PropertyState>,
    /// The specific fill type. Uses flatten for the union key (solidFill or stretchedPictureFill).
    #[serde(flatten)]
    pub fill_kind: Option<ShapeBackgroundFillContent>,
}

// --- Outline and Shadow Structs ---

/// The outline of a PageElement (e.g., Shape, Image, Video).
/// If properties are unset, they may be inherited from a parent placeholder.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#Outline
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Outline {
    /// The fill of the outline.
    pub outline_fill: Option<OutlineFill>,
    /// The thickness of the outline.
    pub weight: Option<Dimension>,
    /// The dash style of the outline.
    pub dash_style: Option<DashStyle>,
    /// The outline property state. If `NOT_RENDERED`, the element has no outline.
    /// Updating the outline implicitly sets this to `RENDERED`.
    pub property_state: Option<PropertyState>,
}

/// The shadow properties of a PageElement (e.g., Shape, Image).
/// If properties are unset, they may be inherited from a parent placeholder.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#Shadow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shadow {
    /// Output only. The type of the shadow (e.g., "outer").
    #[serde(rename = "type")]
    pub shadow_type: Option<String>, // Read-only

    /// Output only. The alignment point of the shadow relative to the shape's bounds.
    pub alignment: Option<RectanglePosition>, // Read-only

    /// Transform that encodes the translation, scale, and skew of the shadow,
    /// relative to the alignment position.
    pub transform: Option<AffineTransform>,

    /// The radius of the shadow blur. Larger radius means more diffuse shadow.
    pub blur_radius: Option<Dimension>,

    /// The shadow color value.
    pub color: Option<OpaqueColor>,

    /// The alpha (opacity) of the shadow's color, from 0.0 (transparent) to 1.0 (opaque).
    pub alpha: Option<f32>,

    /// Output only. Whether the shadow should rotate with the shape.
    pub rotate_with_shape: Option<bool>, // Read-only

    /// The shadow property state. If `NOT_RENDERED`, the element has no shadow.
    /// Updating the shadow implicitly sets this to `RENDERED`.
    pub property_state: Option<PropertyState>,
}

// --- ShapeProperties Struct ---

/// The properties of a Shape element.
/// If properties are unset, they may be inherited from a parent placeholder if it exists.
/// Otherwise, they default to the values used for new shapes created in the Slides editor.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#ShapeProperties
/// The properties of a Shape element.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#ShapeProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShapeProperties {
    /// The background fill of the shape.
    pub shape_background_fill: Option<ShapeBackgroundFill>,
    /// The outline of the shape.
    pub outline: Option<Outline>,
    /// The shadow properties of the shape. Read-only aspects are ignored on update.
    pub shadow: Option<Shadow>,
    /// The hyperlink destination of the shape. If unset, there is no link.
    pub link: Option<Link>,
    /// The alignment of content in the shape. If unspecified, inherited or defaults to MIDDLE.
    pub content_alignment: Option<ContentAlignment>,
    /// The autofit properties of the shape. Only set for shapes that allow text.
    pub autofit: Option<Autofit>, // <<< Added this field
}
