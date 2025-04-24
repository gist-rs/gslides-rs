use serde::{Deserialize, Serialize};

// Import necessary types from other modules
use crate::models::common::Dimension;
use crate::models::link::Link;
use crate::models::shape_properties::{DashStyle, SolidFill}; // DashStyle already defined

/// The style of an arrow head.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/lines#ArrowStyle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArrowStyle {
    /// Unspecified arrow style.
    ArrowStyleUnspecified,
    /// No arrow head.
    None,
    /// Arrow head with notched back.
    StealthArrow,
    /// Filled arrow head.
    FillArrow,
    /// Filled circle arrow head.
    FillCircle,
    /// Filled square arrow head.
    FillSquare,
    /// Filled diamond arrow head.
    FillDiamond,
    /// Hollow arrow head.
    OpenArrow,
    /// Hollow circle arrow head.
    OpenCircle,
    /// Hollow square arrow head.
    OpenSquare,
    /// Hollow diamond arrow head.
    OpenDiamond,
}

/// Properties for one end of a Line connection.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/lines#LineConnection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineConnection {
    /// The object ID of the connected page element.
    pub connected_object_id: Option<String>,
    /// The index of the connection site on the connected page element.
    /// Refer to the API guide for connection site indices.
    pub connection_site_index: Option<i32>,
}

/// The fill properties for a Line. Currently only solid fill is supported.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/lines#LineFill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineFillContent {
    /// Solid color fill.
    SolidFill(SolidFill),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineFill {
    /// The specific fill type. Uses flatten to represent the union based on JSON key.
    #[serde(flatten)]
    pub fill_kind: LineFillContent,
}

/// The type of the line. Corresponds to ECMA-376 ST_ShapeType connector types.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/lines#Type_3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LineType {
    /// Unspecified line type.
    TypeUnspecified,
    /// Straight connector 1 form.
    StraightConnector1,
    /// Bent connector 2 form.
    BentConnector2,
    /// Bent connector 3 form.
    BentConnector3,
    /// Bent connector 4 form.
    BentConnector4,
    /// Bent connector 5 form.
    BentConnector5,
    /// Curved connector 2 form.
    CurvedConnector2,
    /// Curved connector 3 form.
    CurvedConnector3,
    /// Curved connector 4 form.
    CurvedConnector4,
    /// Curved connector 5 form.
    CurvedConnector5,
    /// Straight line (not a connector).
    StraightLine,
}

/// The category of the line. Matches the category specified in CreateLineRequest.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/lines#LineCategory
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LineCategory {
    /// Unspecified line category.
    LineCategoryUnspecified,
    /// Straight connectors (including STRAIGHT_CONNECTOR_1).
    Straight,
    /// Bent connectors (BENT_CONNECTOR_2 to 5).
    Bent,
    /// Curved connectors (CURVED_CONNECTOR_2 to 5).
    Curved,
}

/// The properties of the Line. Default values match new lines in the Slides editor.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/lines#LineProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineProperties {
    /// The fill of the line.
    pub line_fill: Option<LineFill>,
    /// The thickness of the line.
    pub weight: Option<Dimension>,
    /// The dash style of the line.
    pub dash_style: Option<DashStyle>,
    /// The style of the arrow at the beginning of the line.
    pub start_arrow: Option<ArrowStyle>,
    /// The style of the arrow at the end of the line.
    pub end_arrow: Option<ArrowStyle>,
    /// The hyperlink destination of the line. If unset, there is no link.
    pub link: Option<Link>,
    /// The connection at the beginning of the line. If unset, no connection.
    /// Only valid for connector types.
    pub start_connection: Option<LineConnection>,
    /// The connection at the end of the line. If unset, no connection.
    /// Only valid for connector types.
    pub end_connection: Option<LineConnection>,
}

/// A PageElement kind representing a line (connector or non-connector).
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/lines#Line
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    /// The properties of the line.
    pub line_properties: Option<LineProperties>,
    /// The type of the line.
    pub line_type: Option<LineType>,
    // Note: lineCategory is available in the API but primarily for creation/update requests.
    // It might not always be populated in GET responses, hence optional here.
    pub line_category: Option<LineCategory>,
}
