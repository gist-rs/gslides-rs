use serde::{Deserialize, Serialize};

/// Represents a font family and weight used to style a TextRun.
/// This is often read-only, reflecting the actual font used for rendering.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#WeightedFontFamily
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeightedFontFamily {
    /// The font family of the text.
    /// Can be any font from the Font menu in Slides or from Google Fonts.
    /// If unrecognized, the text is rendered in Arial.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,

    /// The rendered weight of the text.
    /// Values are multiples of 100 between 100 and 900 (inclusive). Corresponds
    /// to CSS font-weight values. Default is 400 ("normal"). Weights >= 700 are bold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i32>,
}
