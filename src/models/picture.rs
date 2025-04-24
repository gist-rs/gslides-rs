use serde::{Deserialize, Serialize};

// Import common types
use crate::models::common::Size;
// Import color types needed for ColorStop
use crate::models::colors::OpaqueColor;

/// The stretched picture fill. The page or page element is filled entirely with
/// the specified picture. The picture is stretched to fit its container.
/// This is only supported for shapes with rectangular geometry.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#StretchedPictureFill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StretchedPictureFill {
    /// Reading: An URL to a picture with a default lifetime of 30 minutes.
    /// This URL is tagged with the account of the requester.
    /// Writing: The URL of the picture to initially fetch. The picture is fetched
    /// once at insertion time and a copy is stored for display. Max 2 kB length.
    /// Supported formats: PNG, JPEG, GIF. Max size: 50MB, 25 megapixels.
    pub content_url: String,

    /// The original size of the picture fill. Read-only.
    pub size: Option<Size>, // Read-only
}

/// The crop properties of an object enclosed in a container (e.g., an Image).
/// The properties specify the offsets from the edges of the original bounding
/// rectangle. Offsets are relative to the object's original dimensions.
/// This property is read-only for ImageProperties.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#CropProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CropProperties {
    /// The offset specifies the left edge of the crop rectangle relative to the
    /// left edge of the object's original bounding box.
    pub left_offset: Option<f32>,
    /// The offset specifies the right edge of the crop rectangle relative to the
    /// right edge of the object's original bounding box.
    pub right_offset: Option<f32>,
    /// The offset specifies the top edge of the crop rectangle relative to the
    /// top edge of the object's original bounding box.
    pub top_offset: Option<f32>,
    /// The offset specifies the bottom edge of the crop rectangle relative to the
    /// bottom edge of the object's original bounding box.
    pub bottom_offset: Option<f32>,
    /// The rotation angle of the crop window around its center, in radians.
    /// Rotation angle is applied after the offset.
    pub angle: Option<f32>,
}

/// A color and position in a gradient band. Used for Recolor effects.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#ColorStop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorStop {
    /// The color of the gradient stop.
    pub color: Option<OpaqueColor>,
    /// The alpha value of this color in the gradient band. Defaults to 1.0 (fully opaque).
    pub alpha: Option<f32>,
    /// The relative position of the color stop in the gradient band (0.0 to 1.0).
    pub position: Option<f32>,
}

/// A recolor effect applied on an image. This property is read-only.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#Recolor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recolor {
    /// The name of the recolor effect (e.g., "grayscale", "sepia"). Determined
    /// from `recolor_stops` by matching against the page's color scheme. Read-only.
    pub name: Option<String>, // Read-only

    /// The recolor effect represented by a gradient of color stops. Read-only.
    pub recolor_stops: Option<Vec<ColorStop>>, // Read-only
}
