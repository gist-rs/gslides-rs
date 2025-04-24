use serde::{Deserialize, Serialize};
// Import common types from the common.rs file
use crate::models::common::Size;
// Import the Page struct (defined in src/models/page.rs)
use crate::models::page::Page;

/// Represents a Google Slides presentation.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations
#[derive(Debug, Clone, Serialize, Deserialize)] // Removed PartialEq due to nested complex types
#[serde(rename_all = "camelCase")]
pub struct Presentation {
    /// The ID of the presentation.
    pub presentation_id: String,

    /// The size of pages in the presentation.
    pub page_size: Option<Size>,

    /// The slides in the presentation. A slide inherits properties from a slide layout.
    pub slides: Option<Vec<Page>>,

    /// The title of the presentation.
    pub title: Option<String>,

    /// The slide masters in the presentation. A slide master contains all common
    /// page elements and the common properties for a set of layouts. They serve three purposes:
    /// - Placeholder shapes on a master contain the default text styles and shape properties
    ///   of all placeholder shapes on pages that use that master.
    /// - The master page properties define the common page properties inherited by its layouts.
    /// - Any other shapes on the master slide appear on all slides using that master,
    ///   regardless of their layout.
    pub masters: Option<Vec<Page>>,

    /// The layouts in the presentation. A layout is a template that determines
    /// how content is arranged and styled on the slides that inherit from that
    /// layout.
    pub layouts: Option<Vec<Page>>,

    /// The locale of the presentation, as an IETF BCP 47 language tag (e.g., "en-US").
    pub locale: Option<String>,

    /// Output only. The revision ID of the presentation. Can be used in update requests
    /// to assert the presentation revision hasn't changed since the last read
    /// operation. Only populated if the user has edit access to the presentation.
    /// The revision ID is not a sequential number but an opaque string.
    pub revision_id: Option<String>,

    /// The notes master in the presentation. It serves three purposes:
    /// - Placeholder shapes on a notes master contain the default text styles and
    ///   shape properties of all placeholder shapes on notes pages. Specifically, a
    ///   `SLIDE_IMAGE` placeholder shape contains the slide thumbnail, and a `BODY`
    ///   placeholder shape contains the speaker notes.
    /// - The notes master page properties define the common page properties
    ///   inherited by all notes pages.
    /// - Any other shapes on the notes master appear on all notes pages.
    ///
    /// The notes master is read-only.
    pub notes_master: Option<Page>,
}
