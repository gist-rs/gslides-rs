use serde::{Deserialize, Serialize};

// Import necessary types for properties
use crate::models::shape_properties::{Outline, Shadow};

/// The properties of the Speaker Spotlight shape.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#SpeakerSpotlightProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerSpotlightProperties {
    /// The outline of the Speaker Spotlight shape. If not set, it has no outline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline: Option<Outline>,
    /// The shadow of the Speaker Spotlight shape. If not set, it has no shadow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<Shadow>,
    // Note: Fill properties are not specified for SpeakerSpotlight in the current API docs.
    // It likely uses a default video feed fill.
}

/// A PageElement kind representing a Speaker Spotlight shape.
/// This shape displays the presenter's video feed during presentations.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#SpeakerSpotlight
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerSpotlight {
    /// The properties of the Speaker Spotlight.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaker_spotlight_properties: Option<SpeakerSpotlightProperties>,
}
