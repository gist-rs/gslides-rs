use serde::{Deserialize, Serialize};

// Import necessary types
use crate::models::image_properties::ImageProperties; // For chart image properties

/// The properties of the SheetsChart.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#SheetsChartProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetsChartProperties {
    /// The properties of the embedded chart image. Read-only.
    pub chart_image_properties: Option<ImageProperties>, // Read-only
}

/// A PageElement kind representing a linked chart embedded from Google Sheets.
/// Unlinked charts are represented as Images.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#SheetsChart
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetsChart {
    /// The ID of the Google Sheets spreadsheet that contains the source chart.
    pub spreadsheet_id: Option<String>,

    /// The ID of the specific chart in the Google Sheets spreadsheet that is embedded.
    pub chart_id: Option<i32>, // API spec uses integer

    /// The properties of the Sheets chart. Read-only.
    pub sheets_chart_properties: Option<SheetsChartProperties>, // Read-only

    /// The URL of an image of the embedded chart, with a default lifetime of 30 minutes. Read-only.
    /// This URL is tagged with the account of the requester.
    pub content_url: Option<String>, // Read-only
}
