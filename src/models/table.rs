use serde::{Deserialize, Serialize};

// Import necessary types from other modules
use crate::models::common::Dimension;
use crate::models::table_properties::{
    TableBorderRow, TableCellProperties, TableColumnProperties, TableRowProperties,
};
use crate::models::text::TextContent;

/// A location of a single table cell within a table.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableCellLocation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash for potential Map keys
#[serde(rename_all = "camelCase")]
pub struct TableCellLocation {
    /// The 0-based row index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_index: Option<i32>,
    /// The 0-based column index.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_index: Option<i32>,
}

/// Properties and contents of each cell.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableCell
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableCell {
    /// The location of the cell within the table. Read-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<TableCellLocation>, // Read-only

    /// Row span of the cell. Read-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_span: Option<i32>, // Read-only

    /// Column span of the cell. Read-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_span: Option<i32>, // Read-only

    /// The text content of the cell.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextContent>,

    /// The properties of the table cell.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_cell_properties: Option<TableCellProperties>,
}

/// Properties and contents of each row in a table.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#TableRow
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRow {
    /// Height of the row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_height: Option<Dimension>,
    /// Properties and contents of each cell. Cells spanning multiple columns are
    /// represented only once with a column_span > 1. Cells spanning multiple
    /// rows are contained in only the topmost row and have a row_span > 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_cells: Option<Vec<TableCell>,>,
    /// Properties of the row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_row_properties: Option<TableRowProperties>,
}

/// A PageElement kind representing a table.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/tables#Table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    /// Number of rows in the table.
    pub rows: i32, // API shows required integer
    /// Number of columns in the table.
    pub columns: i32, // API shows required integer

    /// Properties of each column.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_columns: Option<Vec<TableColumnProperties>,>,

    /// Properties and contents of each row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_rows: Option<Vec<TableRow>,>,

    /// Properties of horizontal cell borders. A grid with `rows + 1` rows and `columns` columns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horizontal_border_rows: Option<Vec<TableBorderRow>,>,

    /// Properties of vertical cell borders. A grid with `rows` rows and `columns + 1` columns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_border_rows: Option<Vec<TableBorderRow>,>,
}
