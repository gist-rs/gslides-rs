use serde::{Deserialize, Serialize};

// Import necessary types
use crate::models::link::Link;
use crate::models::picture::{CropProperties, Recolor};
use crate::models::shape_properties::{Outline, Shadow}; // Reusing Outline and Shadow // Assuming CropProperties, Recolor defined elsewhere

/// The properties of an Image page element.
/// Many fields are read-only and correspond to image effects applied in the Slides editor.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#ImageProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageProperties {
    /// The brightness effect of the image. Value between -1.0 and 1.0. Read-only.
    pub brightness: Option<f32>, // Read-only

    /// The contrast effect of the image. Value between -1.0 and 1.0. Read-only.
    pub contrast: Option<f32>, // Read-only

    /// The transparency effect of the image. Value between 0.0 and 1.0. Read-only.
    pub transparency: Option<f32>, // Read-only

    /// The crop properties of the image. If not set, image is not cropped. Read-only.
    pub crop_properties: Option<CropProperties>, // Read-only

    /// The outline of the image. If not set, image has no outline.
    pub outline: Option<Outline>,

    /// The shadow of the image. If not set, image has no shadow. Read-only.
    pub shadow: Option<Shadow>, // Read-only

    /// The hyperlink destination of the image. If unset, there is no link.
    pub link: Option<Link>,

    /// The recolor effect of the image. If not set, image is not recolored. Read-only.
    pub recolor: Option<Recolor>, // Read-only
}

// NOTE: Placeholder structs CropProperties and Recolor need definition,
// assuming they would go in src/models/picture.rs or similar.
