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
    pub outline: Option<Outline>,
    /// Whether to enable video autoplay when the page is displayed in present mode.
    /// Defaults to false.
    pub auto_play: Option<bool>,
    /// The time at which to start playback, in seconds from the beginning.
    /// If set, should be before `end`. If > video length, plays from last second.
    /// If unset, plays from the beginning.
    #[serde(rename = "start")]
    // Use 'startAt' to avoid Rust keyword conflict? API uses 'start'.
    pub start_at: Option<i64>, // API uses integer seconds
    /// The time at which to end playback, in seconds from the beginning.
    /// If set, should be after `start`. If > video length or unset, plays until the end.
    #[serde(rename = "end")] // Use 'endAt' to avoid Rust keyword conflict? API uses 'end'.
    pub end_at: Option<i64>, // API uses integer seconds
    /// Whether to mute the audio during video playback. Defaults to false.
    pub mute: Option<bool>,
}

/// A PageElement kind representing a video.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/videos#Video
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    /// The video source.
    pub source: Option<VideoSource>,
    /// An URL to a video. The URL is valid as long as the source video exists and
    /// sharing settings do not change. Read-only.
    pub url: Option<String>, // Read-only
    /// The video source's unique identifier for this video.
    /// E.g., YouTube video ID or Drive file ID.
    pub id: String, // Changed to non-optional based on usage context
    /// The properties of the video.
    pub video_properties: Option<VideoProperties>,
}
