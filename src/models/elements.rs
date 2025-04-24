// src/models/elements.rs

use serde::{Deserialize, Serialize};

// Import common types
use crate::models::common::{AffineTransform, Size};

// Import specific element types
use crate::models::group::Group;
use crate::models::image::Image;
use crate::models::line::Line;
use crate::models::shape::Shape;
use crate::models::sheets_chart::SheetsChart;
use crate::models::speaker_spotlight::SpeakerSpotlight;
use crate::models::table::Table;
use crate::models::video::Video;
use crate::models::wordart::WordArt;

/// The specific kind of PageElement represented as an enum with associated data.
/// The JSON representation uses the field name as the key (e.g., "shape": {...}, "image": {...}).
/// Derived from the union field `element_kind` in:
/// https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#PageElement
#[derive(Debug, Clone, Serialize, Deserialize)] // Removed PartialEq due to nested complexity (e.g., Group->Vec<PageElement>)
#[serde(rename_all = "camelCase")]
pub enum PageElementKind {
    /// A collection of page elements joined as a single unit.
    ElementGroup(Group),
    /// A generic shape.
    Shape(Shape),
    /// An image page element.
    Image(Image),
    /// A video page element.
    Video(Video),
    /// A line page element.
    Line(Line),
    /// A table page element.
    Table(Table),
    /// A word art page element. Text rendered with special styles.
    WordArt(WordArt),
    /// A linked chart embedded from Google Sheets. Unlinked charts are represented as Images.
    SheetsChart(SheetsChart),
    /// A Speaker Spotlight element. Renders the presenter's video feed.
    SpeakerSpotlight(SpeakerSpotlight),
}

/// A visual element rendered on a page.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#PageElement
#[derive(Debug, Clone, Serialize, Deserialize)] // Removed PartialEq due to PageElementKind
#[serde(rename_all = "camelCase")]
pub struct PageElement {
    /// The object ID for this page element. Object IDs used by Page and PageElement
    /// share the same namespace.
    pub object_id: String,

    /// The size of the page element.
    pub size: Option<Size>,

    /// The transform of the page element. The visual appearance is determined by
    /// its absolute transform (pre-concatenated with transforms of parent groups).
    pub transform: Option<AffineTransform>,

    /// The title of the page element. Combined with description for alt text.
    /// Not supported for Group elements.
    pub title: Option<String>,

    /// The description of the page element. Combined with title for alt text.
    /// Not supported for Group elements.
    pub description: Option<String>,

    /// The specific kind of element and its properties.
    /// The `flatten` attribute merges the fields of the specific element struct
    /// (e.g., Shape, Image) into this PageElement during deserialization,
    /// based on the corresponding JSON key (e.g., "shape", "image").
    #[serde(flatten)]
    pub element_kind: PageElementKind,
}
