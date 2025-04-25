use serde::{Deserialize, Serialize};

/// The type of placeholder. Helps identify the relationship between a shape on a slide
/// and the corresponding placeholder shape on the layout or master slide.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#Type_4
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlaceholderType {
    /// Default value, signifies it is not a placeholder.
    None,
    /// Body text placeholder.
    Body,
    /// Chart placeholder.
    Chart,
    /// Clip art placeholder.
    ClipArt,
    /// Centered title placeholder.
    CenteredTitle,
    /// Diagram placeholder.
    Diagram,
    /// Date and time placeholder.
    DateAndTime,
    /// Footer placeholder.
    Footer,
    /// Header placeholder.
    Header,
    /// Media placeholder.
    Media,
    /// Any content type placeholder.
    Object,
    /// Picture placeholder.
    Picture,
    /// Slide number placeholder.
    SlideNumber,
    /// Subtitle placeholder.
    Subtitle,
    /// Table placeholder.
    Table,
    /// Title placeholder.
    Title,
    /// Slide image placeholder (usually on notes master).
    SlideImage,
}

/// The placeholder information that uniquely identifies a placeholder shape.
/// Inherited properties are resolved based on this information.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#Placeholder
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placeholder {
    /// The type of the placeholder.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder_type: Option<PlaceholderType>,
    /// The index of the placeholder. If the same placeholder types are present on the
    /// same page, they would have different index values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
    /// The object ID of this shape's parent placeholder. If unset, the parent
    /// placeholder shape does not exist, so the shape does not inherit properties
    /// from any other shape.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_object_id: Option<String>,
}
