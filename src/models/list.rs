use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

// Import TextStyle needed for NestingLevel's bulletStyle
use crate::models::properties::TextStyle;

/// Contains properties describing the look and feel of bullets at a given level of nesting.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#NestingLevel
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NestingLevel {
    /// The style of a bullet at this level of nesting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bullet_style: Option<TextStyle>,
}

/// A List describes the look and feel of bullets belonging to paragraphs associated with a list ID.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#List
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct List {
    /// The ID of the list.
    pub list_id: String,

    /// A map of nesting levels (0-8) to the properties of bullets at the associated level.
    /// The keys are integers representing the nesting level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nesting_level: Option<IndexMap<i32, NestingLevel>>,
}
