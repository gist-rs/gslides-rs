use serde::{Deserialize, Serialize};

// Import PageElement as Group contains children of this type
use crate::models::elements::PageElement;

/// A PageElement kind representing a joined collection of PageElements.
/// The minimum size of a group is 2.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#Group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    /// The collection of elements in the group.
    pub children: Vec<PageElement>,
}
