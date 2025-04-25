use serde::{ser::SerializeMap, Deserialize, Serialize, Serializer};

impl Serialize for Link {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.destination {
            LinkKind::None => {
                // If the destination is LinkKind::None, serialize an empty JSON object {}
                let map = serializer.serialize_map(Some(0))?;
                map.end()
            }
            other_kind => other_kind.serialize(serializer),
        }
    }
}

/// Describes the type of relative link between slides.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#RelativeSlideLink
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelativeSlideLink {
    /// An unspecified relative slide link.
    #[default]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // Removed Default derive
#[serde(rename_all = "camelCase")]
pub enum LinkKind {
    /// Represents no link destination. Added for Default impl.
    /// Note: The API might represent "no link" by omitting the 'link' field entirely,
    /// rather than using a specific 'none' variant key. This 'None' variant
    /// primarily serves the Default trait implementation.
    None, // New variant for Default

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

#[allow(clippy::derivable_impls)]
impl Default for LinkKind {
    fn default() -> Self {
        LinkKind::None // Explicitly define 'None' as the default
    }
}

/// A hypertext link.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/other#Link
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    /// The destination of the link. Uses flatten to represent the union based on JSON key.
    #[serde(flatten)]
    pub destination: LinkKind,
}
