use serde::{Deserialize, Serialize};

// Import necessary types
use crate::models::colors::ColorScheme; // Defined in colors.rs
use crate::models::picture::StretchedPictureFill; // Defined in picture.rs
use crate::models::shape_properties::{PropertyState, SolidFill}; // Defined in shape_properties.rs

/// The background fill of a Page.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#PageBackgroundFill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageBackgroundFill {
    /// The background fill property state. Updating the fill on a page will
    /// implicitly update this field to `RENDERED`, unless another value is
    /// specified in the same request. To have no fill, set this field to
    /// `NOT_RENDERED`. In this case, any other fill fields set in the same
    /// request will be ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_state: Option<PropertyState>,

    // The kind of background fill. Only one of these will be present.
    /// Solid color fill.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solid_fill: Option<SolidFill>,

    /// Stretched picture fill.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stretched_picture_fill: Option<StretchedPictureFill>,
    // Note: The API represents this as optional fields rather than a strict union key.
}

/// The properties of a Page. Inherited properties are represented as unset fields.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#PageProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageProperties {
    /// The background fill of the page. If unset, the background fill is inherited
    /// from a parent page if it exists. If the page has no parent, then the
    /// background fill defaults to the corresponding fill in the Slides editor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_background_fill: Option<PageBackgroundFill>,

    /// The color scheme of the page. If unset, the color scheme is inherited from
    /// a parent page. If the page has no parent, the color scheme uses a default
    /// Slides color scheme, matching the defaults in the Slides editor.
    /// Only the concrete colors of the first 12 ThemeColorTypes are editable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<ColorScheme>,
}
