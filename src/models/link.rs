use serde::{Deserialize, Serialize};

/// Describes the type of relative link between slides.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#RelativeSlideLink
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelativeSlideLink {
    /// An unspecified relative slide link.
    RelativeSlideLinkUnspecified,
    /// A link to the next slide.
    NextSlide,
    /// A link to the previous slide.
    PreviousSlide,
    /// A link to the first slide in the presentation.
    FirstSlide,
    /// A link to the last slide in the presentation.
    LastSlide,
}

/// Represents the specific destination of a Link.
/// The JSON representation uses the field name ("url", "relativeLink", etc.) as the key.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LinkKind {
    /// If set, indicates this is a link to the external web page at this URL.
    Url(String),
    /// If set, indicates this is a link to a slide in this presentation, addressed by its relative position.
    RelativeLink(RelativeSlideLink),
    /// If set, indicates this is a link to the specific page in this presentation with this ID.
    /// A page with this ID may not exist.
    PageObjectId(String),
    /// If set, indicates this is a link to the slide at this zero-based index in the presentation.
    /// There may not be a slide at this index.
    SlideIndex(i32),
}

/// A hypertext link.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#Link
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    /// The destination of the link. Uses flatten to represent the union based on JSON key.
    #[serde(flatten)]
    pub destination: LinkKind,
}
