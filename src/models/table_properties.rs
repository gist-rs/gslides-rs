use serde::{Deserialize, Serialize};

// Import necessary types from other modules
use crate::models::common::Dimension;
use crate::models::shape_properties::{ContentAlignment, DashStyle, PropertyState, SolidFill}; // Reusing enums and SolidFill
use crate::models::table::TableCellLocation; // Defined in table.rs

/// The fill of the border. Currently only solid fill is supported for table borders.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableBorderFill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TableBorderFillContent {
    /// Solid color fill.
    SolidFill(SolidFill),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBorderFill {
    /// The specific fill type.
    #[serde(flatten)]
    pub fill_kind: TableBorderFillContent,
}

/// The border styling properties of a TableBorderCell.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableBorderProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBorderProperties {
    /// The fill of the table border.
    pub table_border_fill: Option<TableBorderFill>,
    /// The thickness of the border.
    pub weight: Option<Dimension>,
    /// The dash style of the border.
    pub dash_style: Option<DashStyle>,
}

/// The properties of each border cell.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableBorderCell
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBorderCell {
    /// The location of the border within the border table.
    pub location: Option<TableCellLocation>,
    /// The border properties.
    pub table_border_properties: Option<TableBorderProperties>,
}

/// Contents of each border row in a table. A TableBorderRow corresponds to a
/// horizontal or vertical border between cells and contains the properties of
/// the border cells spanning the row.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableBorderRow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableBorderRow {
    /// Properties of each border cell. When a border's adjacent table cells are
    /// merged, it is not included in the response.
    pub table_border_cells: Option<Vec<TableBorderCell>>,
}

/// The background fill of a table cell.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableCellBackgroundFill
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCellBackgroundFill {
    /// The background fill property state.
    pub property_state: Option<PropertyState>,
    /// Solid color fill. Only solid fill is currently supported.
    pub solid_fill: Option<SolidFill>,
    // Note: Similar to PageBackgroundFill, represented as optional fields.
}

/// Properties of a TableCell.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableCellProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCellProperties {
    /// The background fill of the table cell. Default matches editor defaults.
    pub table_cell_background_fill: Option<TableCellBackgroundFill>,
    /// The alignment of the content in the table cell. Default matches editor defaults.
    pub content_alignment: Option<ContentAlignment>,
}

/// Properties of each column in a table.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableColumnProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableColumnProperties {
    /// Width of a column.
    pub column_width: Option<Dimension>,
}

/// Properties of each row in a table.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableRowProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRowProperties {
    /// Minimum height of the row. The row will be rendered this tall, but may be
    /// taller if content requires it.
    pub min_row_height: Option<Dimension>,
}

/// A rectangular range of table cells.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableRange
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRange {
    /// The starting location of the range.
    pub location: Option<TableCellLocation>,
    /// The row span of the table range.
    pub row_span: Option<i32>,
    /// The column span of the table range.
    pub column_span: Option<i32>,
}
