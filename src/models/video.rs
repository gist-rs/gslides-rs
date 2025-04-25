use serde::{Deserialize, Serialize};

// Import necessary types
use crate::models::shape_properties::Outline; // Reuse Outline defined earlier

/// The source of the video.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/videos#Source
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VideoSource {
    /// The video source is unspecified.
    SourceUnspecified,
    /// The video source is YouTube.
    Youtube,
    /// The video source is Google Drive.
    Drive,
}

/// The properties of the Video.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/videos#VideoProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoProperties {
    /// The outline of the video. Defaults match new videos in the Slides editor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline: Option<Outline>,
    /// Whether to enable video autoplay when the page is displayed in present mode.
    /// Defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_play: Option<bool>,
    /// The time at which to start playback, in seconds from the beginning.
    /// If set, should be before `end`. If > video length, plays from last second.
    /// If unset, plays from the beginning.
    #[serde(rename = "start")]
    // Use 'startAt' to avoid Rust keyword conflict? API uses 'start'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<i64>, // API uses integer seconds
    /// The time at which to end playback, in seconds from the beginning.
    /// If set, should be after `start`. If > video length or unset, plays until the end.
    #[serde(rename = "end")] // Use 'endAt' to avoid Rust keyword conflict? API uses 'end'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<i64>, // API uses integer seconds
    /// Whether to mute the audio during video playback. Defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mute: Option<bool>,
}

/// A PageElement kind representing a video.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/videos#Video
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    /// The video source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<VideoSource>,
    /// An URL to a video. The URL is valid as long as the source video exists and
    /// sharing settings do not change. Read-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>, // Read-only
    /// The video source's unique identifier for this video.
    /// E.g., YouTube video ID or Drive file ID.
    pub id: String, // Changed to non-optional based on usage context
    /// The properties of the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_properties: Option<VideoProperties>,
}
