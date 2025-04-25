use serde::{Deserialize, Serialize};

/// A PageElement kind representing word art.
/// Text rendered with special effects.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#WordArt
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WordArt {
    /// The text rendered as word art.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered_text: Option<String>,
    // Note: WordArt styling is complex and not directly exposed via simple properties.
    // Modifications usually involve replacing the WordArt element.
}
