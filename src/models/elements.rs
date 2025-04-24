use crate::models::common::{AffineTransform, Size};
use crate::models::group::Group;
use crate::models::image::Image;
use crate::models::line::Line;
use crate::models::shape::Shape;
use crate::models::sheets_chart::SheetsChart;
use crate::models::speaker_spotlight::SpeakerSpotlight;
use crate::models::table::Table;
use crate::models::video::Video;
use crate::models::wordart::WordArt;
// --- Imports needed for manual Deserialize ---
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;
// ---

/// The specific kind of PageElement represented as an enum with associated data.
/// NOTE: PartialEq removed as it might not be derivable/needed depending on nested types. Add back if necessary and feasible.
#[derive(Debug, Clone, Serialize)] // Deserialize is handled manually for PageElement wrapper
#[serde(rename_all = "camelCase")]
pub enum PageElementKind {
    ElementGroup(Group),
    Shape(Shape),
    Image(Image),
    Video(Video),
    Line(Line),
    Table(Table),
    WordArt(WordArt),
    SheetsChart(SheetsChart),
    SpeakerSpotlight(SpeakerSpotlight),
}

/// A visual element rendered on a page.
/// NOTE: Manual Deserialize implemented below. PartialEq likely removed due to complexity.
#[derive(Debug, Clone, Serialize)] // Removed Deserialize derive
#[serde(rename_all = "camelCase")]
pub struct PageElement {
    /// The object ID for this page element.
    pub object_id: String, // Assuming objectId is always present
    /// The size of the page element.
    pub size: Option<Size>,
    /// The transform of the page element.
    pub transform: Option<AffineTransform>,
    /// The title of the page element.
    pub title: Option<String>,
    /// The description of the page element.
    pub description: Option<String>,
    /// The specific kind of element and its properties.
    // No longer flattened, handled by manual Deserialize below.
    pub element_kind: PageElementKind,
}

// --- Manual Deserialization Implementation for PageElement ---

impl<'de> Deserialize<'de> for PageElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Define field names as constants
        const FIELD_OBJECT_ID: &str = "objectId";
        const FIELD_SIZE: &str = "size";
        const FIELD_TRANSFORM: &str = "transform";
        const FIELD_TITLE: &str = "title";
        const FIELD_DESCRIPTION: &str = "description";
        // Element kind fields
        const FIELD_ELEMENT_GROUP: &str = "elementGroup";
        const FIELD_SHAPE: &str = "shape";
        const FIELD_IMAGE: &str = "image";
        const FIELD_VIDEO: &str = "video";
        const FIELD_LINE: &str = "line";
        const FIELD_TABLE: &str = "table";
        const FIELD_WORD_ART: &str = "wordArt";
        const FIELD_SHEETS_CHART: &str = "sheetsChart";
        const FIELD_SPEAKER_SPOTLIGHT: &str = "speakerSpotlight";

        // Visitor implementation
        struct PageElementVisitor;

        impl<'de> Visitor<'de> for PageElementVisitor {
            type Value = PageElement;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct PageElement")
            }

            fn visit_map<V>(self, mut map: V) -> Result<PageElement, V::Error>
            where
                V: MapAccess<'de>,
            {
                // Local variables captured from outer scope
                let mut object_id: Option<String> = None;
                let mut size: Option<Size> = None;
                let mut transform: Option<AffineTransform> = None;
                let mut title: Option<String> = None;
                let mut description: Option<String> = None;
                let mut element_kind: Option<PageElementKind> = None;

                // Iterate over map keys
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        FIELD_OBJECT_ID => {
                            if object_id.is_some() {
                                return Err(de::Error::duplicate_field(FIELD_OBJECT_ID));
                            }
                            object_id = Some(map.next_value()?);
                        }
                        FIELD_SIZE => {
                            if size.is_some() {
                                return Err(de::Error::duplicate_field(FIELD_SIZE));
                            }
                            size = Some(map.next_value()?);
                        }
                        FIELD_TRANSFORM => {
                            if transform.is_some() {
                                return Err(de::Error::duplicate_field(FIELD_TRANSFORM));
                            }
                            transform = Some(map.next_value()?);
                        }
                        FIELD_TITLE => {
                            if title.is_some() {
                                return Err(de::Error::duplicate_field(FIELD_TITLE));
                            }
                            title = Some(map.next_value()?);
                        }
                        FIELD_DESCRIPTION => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field(FIELD_DESCRIPTION));
                            }
                            description = Some(map.next_value()?);
                        }
                        // Element Kind handling: Check if already found, then deserialize value
                        FIELD_ELEMENT_GROUP => {
                            if element_kind.is_some() {
                                return Err(de::Error::custom("Multiple element kinds found"));
                            }
                            element_kind = Some(PageElementKind::ElementGroup(map.next_value()?));
                        }
                        FIELD_SHAPE => {
                            if element_kind.is_some() {
                                return Err(de::Error::custom("Multiple element kinds found"));
                            }
                            element_kind = Some(PageElementKind::Shape(map.next_value()?));
                        }
                        FIELD_IMAGE => {
                            if element_kind.is_some() {
                                return Err(de::Error::custom("Multiple element kinds found"));
                            }
                            element_kind = Some(PageElementKind::Image(map.next_value()?));
                        }
                        FIELD_VIDEO => {
                            if element_kind.is_some() {
                                return Err(de::Error::custom("Multiple element kinds found"));
                            }
                            element_kind = Some(PageElementKind::Video(map.next_value()?));
                        }
                        FIELD_LINE => {
                            if element_kind.is_some() {
                                return Err(de::Error::custom("Multiple element kinds found"));
                            }
                            element_kind = Some(PageElementKind::Line(map.next_value()?));
                        }
                        FIELD_TABLE => {
                            if element_kind.is_some() {
                                return Err(de::Error::custom("Multiple element kinds found"));
                            }
                            element_kind = Some(PageElementKind::Table(map.next_value()?));
                        }
                        FIELD_WORD_ART => {
                            if element_kind.is_some() {
                                return Err(de::Error::custom("Multiple element kinds found"));
                            }
                            element_kind = Some(PageElementKind::WordArt(map.next_value()?));
                        }
                        FIELD_SHEETS_CHART => {
                            if element_kind.is_some() {
                                return Err(de::Error::custom("Multiple element kinds found"));
                            }
                            element_kind = Some(PageElementKind::SheetsChart(map.next_value()?));
                        }
                        FIELD_SPEAKER_SPOTLIGHT => {
                            if element_kind.is_some() {
                                return Err(de::Error::custom("Multiple element kinds found"));
                            }
                            element_kind =
                                Some(PageElementKind::SpeakerSpotlight(map.next_value()?));
                        }
                        // Ignore unknown fields if necessary, or return an error
                        _ => {
                            let _ = map.next_value::<serde_json::Value>()?; // Consume the value to advance map
                                                                            // Optionally log unknown field: log::debug!("Ignoring unknown field: {}", key);
                        }
                    }
                }

                // Check required fields and construct PageElement
                let object_id =
                    object_id.ok_or_else(|| de::Error::missing_field(FIELD_OBJECT_ID))?;
                let element_kind = element_kind.ok_or_else(|| {
                    de::Error::custom("Missing element kind field (e.g., shape, image)")
                })?;

                Ok(PageElement {
                    object_id,
                    size,
                    transform,
                    title,
                    description,
                    element_kind,
                })
            }
        }

        // Define the fields PageElement expects
        const FIELDS: &[&str] = &[
            FIELD_OBJECT_ID,
            FIELD_SIZE,
            FIELD_TRANSFORM,
            FIELD_TITLE,
            FIELD_DESCRIPTION,
            FIELD_ELEMENT_GROUP,
            FIELD_SHAPE,
            FIELD_IMAGE,
            FIELD_VIDEO,
            FIELD_LINE,
            FIELD_TABLE,
            FIELD_WORD_ART,
            FIELD_SHEETS_CHART,
            FIELD_SPEAKER_SPOTLIGHT,
        ];
        deserializer.deserialize_struct("PageElement", FIELDS, PageElementVisitor)
    }
}
