// src/models/page.rs

use serde::{Deserialize, Serialize};

// Import necessary structs
use crate::models::elements::PageElement;
use crate::models::page_properties::PageProperties; // Renamed from 'properties' for clarity if desired, ensure import matches file name
use crate::models::properties::{
    // Assuming specific props remain in properties.rs
    LayoutProperties,
    MasterProperties,
    NotesProperties,
    SlideProperties,
};

/// The type of the page.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#PageType
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PageType {
    /// The page type is unspecified or unknown.
    #[default] // Make Unspecified the default if #[serde(default)] is used on Option<PageType>
    PageTypeUnspecified,
    /// A slide page.
    Slide,
    /// A master slide page.
    Master,
    /// A layout page.
    Layout,
    /// A notes page.
    Notes,
    /// A notes master page.
    NotesMaster,
}

/// A page in a presentation.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#Page
#[derive(Debug, Clone, Serialize, Deserialize)] // Removed PartialEq due to nested complex types (PageElement, SlideProperties->Box<Page>)
#[serde(rename_all = "camelCase")]
pub struct Page {
    /// The object ID for this page. Object IDs used by Page and PageElement share
    /// the same namespace.
    pub object_id: String,

    /// The type of the page. This might be omitted in some contexts (e.g., top-level slides).
    #[serde(default)]
    pub page_type: Option<PageType>,

    /// The page elements rendered on the page. Use `pageElements.get` to retrieve elements.
    pub page_elements: Option<Vec<PageElement>>,

    /// Output only. The revision ID of the presentation containing the page. Can be used in
    /// update requests to assert the page revision hasn't changed since the last
    /// read operation. Only populated if the user has edit access to the
    /// presentation. The revision ID is an opaque string.
    pub revision_id: Option<String>,

    /// The properties of the page.
    pub page_properties: Option<PageProperties>,

    // --- Page type specific properties ---
    // Only one of these will be populated based on `page_type`.
    /// Slide specific properties. Only set if page_type = SLIDE.
    pub slide_properties: Option<SlideProperties>,

    /// Layout specific properties. Only set if page_type = LAYOUT.
    pub layout_properties: Option<LayoutProperties>,

    /// Notes specific properties. Only set if page_type = NOTES.
    pub notes_properties: Option<NotesProperties>,

    /// Master specific properties. Only set if page_type = MASTER.
    /// Note: Masters inherit properties from PageProperties. Specific MasterProperties might be minimal.
    pub master_properties: Option<MasterProperties>,
}
