// src/models/text_element.rs

use serde::{Deserialize, Serialize};

// Import necessary style and list types
use crate::models::bullet::Bullet;
use crate::models::properties::{ParagraphStyle, TextStyle};

/// Represents a segment of text with consistent styling within a paragraph.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#TextRun
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // PartialEq likely okay here
#[serde(rename_all = "camelCase")]
pub struct TextRun {
    /// The text content of this run. Newline characters ('\n') are implicitly represented.
    pub content: Option<String>,
    /// The styling applied to this run. If unset, the value is inherited from
    /// the parent paragraph's style or placeholder.
    pub style: Option<TextStyle>,
}

/// Represents the beginning of a new paragraph marker in the text element stream.
/// The range of the paragraph (start/end index) is implicitly defined by its position
/// relative to other markers.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#ParagraphMarker
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // PartialEq likely okay here
#[serde(rename_all = "camelCase")]
pub struct ParagraphMarker {
    /// The paragraph's style. If unset, the value is inherited from the parent
    /// placeholder or the default style.
    pub style: Option<ParagraphStyle>,
    /// The bullet for this paragraph. If unset, the paragraph doesn't have a
    /// bullet and the value is inherited from the parent placeholder or list style.
    pub bullet: Option<Bullet>,
}

/// The type of AutoText.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#Type_1
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AutoTextType {
    /// Type unspecified.
    TypeUnspecified,
    /// Represents the current slide number.
    SlideNumber,
    /// Represents the total number of slides in the presentation.
    PageCount, // API docs call this SLIDE_COUNT, but JSON examples often use PAGE_COUNT. Verify.
}

/// A TextElement representing a spot in the text that is dynamically
/// replaced with content such as the current slide number or page count.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#AutoText
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // PartialEq likely okay here
#[serde(rename_all = "camelCase")]
pub struct AutoText {
    /// The type of this auto text.
    #[serde(rename = "type")]
    pub auto_text_type: Option<AutoTextType>,
    /// Output only. The rendered content of this auto text (e.g., the slide number).
    pub content: Option<String>, // Read-only
    /// The styling applied to this auto text. If unset, the value is inherited.
    pub style: Option<TextStyle>,
}

/// A union representing a single logical element in the text stream of a Shape or TableCell.
/// The actual text content is stored within `TextRun` variants.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#TextElement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // PartialEq depends on variants
#[serde(rename_all = "camelCase")]
pub enum TextElement {
    /// A run of text with consistent styling.
    TextRun(TextRun),
    /// A marker indicating the beginning of a paragraph and its properties.
    ParagraphMarker(ParagraphMarker),
    /// A placeholder for automatically generated text (e.g., slide number).
    AutoText(AutoText),
    // The API documentation mentions start/end indices, but these are typically implicit
    // based on the order and content length, not explicit fields in the JSON struct.
}
