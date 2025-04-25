use serde::{Deserialize, Serialize};

// Import necessary style and list types
use crate::models::bullet::Bullet;
use crate::models::properties::{ParagraphStyle, TextStyle};

/// Represents a segment of text with consistent styling within a paragraph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRun {
    /// The text content of this run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The styling applied to this run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<TextStyle>,
}

/// Represents the beginning of a new paragraph marker in the text element stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphMarker {
    /// The paragraph's style.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<ParagraphStyle>,
    /// The bullet for this paragraph.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bullet: Option<Bullet>,
}

/// The type of AutoText.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AutoTextType {
    TypeUnspecified,
    SlideNumber,
    #[serde(alias = "SLIDE_COUNT")] // Alias needed if API uses SLIDE_COUNT sometimes
    PageCount,
}

/// A TextElement representing a spot in the text that is dynamically replaced.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoText {
    /// The type of this auto text.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_text_type: Option<AutoTextType>,
    /// Output only. The rendered content of this auto text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>, // Read-only
    /// The styling applied to this auto text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<TextStyle>,
}

/// Represents the specific kind of content within a TextElement.
/// The JSON object containing this will have a key like "textRun", "paragraphMarker", etc.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // PartialEq should be okay here
#[serde(rename_all = "camelCase")]
pub enum TextElementKind {
    /// A run of text with consistent styling.
    TextRun(TextRun),
    /// A marker indicating the beginning of a paragraph and its properties.
    ParagraphMarker(ParagraphMarker),
    /// A placeholder for automatically generated text (e.g., slide number).
    AutoText(AutoText),
}

/// A structural element in a TextContent object. Represents a range of text with
/// specific properties or markers.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#TextElement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // PartialEq depends on Kind
#[serde(rename_all = "camelCase")]
pub struct TextElement {
    /// The zero-based start index of this text element, exclusive, in UTF-16 code units.
    /// Often omitted, indices are implicitly defined by order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u32>, // API uses integer, u32 suitable for indices

    /// The zero-based end index of this text element, exclusive, in UTF-16 code units.
    /// Often omitted, indices are implicitly defined by order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<u32>, // API uses integer

    /// The specific kind of text element. Contains the properties specific to that kind.
    /// Uses flatten to merge the variant key ("textRun", "paragraphMarker", etc.)
    /// alongside the optional startIndex/endIndex fields.
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<TextElementKind>, // Made Option<> in case element is empty/invalid? Check API examples. Usually present. Let's keep it Option for robustness.
}
