use serde::{Deserialize, Serialize};

// Import TextStyle needed for bulletStyle
use crate::models::properties::TextStyle;

/// Describes the bullet of a paragraph.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#Bullet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bullet {
    /// The ID of the list this paragraph belongs to.
    pub list_id: Option<String>,

    /// The nesting level of this paragraph in the list (0-8).
    pub nesting_level: Option<i32>,

    /// The rendered bullet glyph for this paragraph. Read-only.
    pub glyph: Option<String>, // Read-only

    /// The paragraph-specific text style applied to this bullet.
    pub bullet_style: Option<TextStyle>,
}
