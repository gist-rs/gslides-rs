use serde::{Deserialize, Serialize};

// Import necessary types
use crate::models::image_properties::ImageProperties; // Assuming defined below/elsewhere
use crate::models::placeholder::Placeholder;

/// A PageElement kind representing an image.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/images#Image
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// An URL to an image with a default lifetime of 30 minutes. Read-only.
    /// This URL is tagged with the account of the requester. Access may be lost if
    /// sharing settings change.
    pub content_url: Option<String>, // Read-only

    /// The source URL is the URL used to insert the image. The source URL can be empty.
    pub source_url: Option<String>,

    /// The properties of the image.
    pub image_properties: Option<ImageProperties>,

    /// The placeholder information for the image. If set, the image is a placeholder image.
    pub placeholder: Option<Placeholder>,
}
