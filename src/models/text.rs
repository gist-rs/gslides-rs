// src/models/text.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import necessary types
use crate::models::list::List;
use crate::models::text_element::TextElement;

/// Represents the textual content of a Shape or TableCell.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#TextContent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // PartialEq depends on HashMap and Vec<TextElement>
#[serde(rename_all = "camelCase")]
pub struct TextContent {
    /// Output only. The text contents broken down into its component parts (TextElements),
    /// including styling information. This property is read-only. To update text content,
    /// use specific requests like InsertTextRequest, DeleteTextRequest, etc.
    pub text_elements: Option<Vec<TextElement>>, // Read-only

    /// The bulleted lists used in this text, keyed by list ID. A `List` defines
    /// the properties applying to bullets at various nesting levels.
    pub lists: Option<HashMap<String, List>>,
}
