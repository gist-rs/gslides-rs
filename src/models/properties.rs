// src/models/properties.rs

use serde::{Deserialize, Serialize};

// Import common types
use crate::models::common::Dimension;

// Import dependent types
use crate::models::colors::{ColorScheme, OptionalColor};
use crate::models::font::WeightedFontFamily;
use crate::models::link::Link;
use crate::models::page::Page;

use super::page_properties::PageBackgroundFill;

/// The text's vertical offset from its normal position.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#BaselineOffset
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BaselineOffset {
    /// The baseline offset is unspecified or inherited.
    BaselineOffsetUnspecified,
    /// The text is not vertically offset.
    None,
    /// The text is vertically offset upwards (superscript).
    Superscript,
    /// The text is vertically offset downwards (subscript).
    Subscript,
}

/// Represents the styling that can be applied to a TextRun.
/// If properties are unset, they may be inherited from a parent placeholder or the underlying paragraph style.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#TextStyle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextStyle {
    /// The background color of the text. If set, the color is either opaque or
    /// transparent, depending on if the `opaque_color` field in it is set.
    pub background_color: Option<OptionalColor>,

    /// The foreground color of the text. If set, the color is either opaque or
    /// transparent, depending on if the `opaque_color` field in it is set.
    pub foreground_color: Option<OptionalColor>,

    /// The font family of the text. Can be any font from the Font menu in Slides
    /// or from Google Fonts. If unrecognized, rendered in Arial.
    pub font_family: Option<String>,

    /// The size of the text's font. When read, specified in points.
    pub font_size: Option<Dimension>,

    /// Whether the text is rendered as bold.
    pub bold: Option<bool>,

    /// Whether the text is italicized.
    pub italic: Option<bool>,

    /// Whether the text is underlined.
    pub underline: Option<bool>,

    /// Whether the text is struck through.
    pub strikethrough: Option<bool>,

    /// Whether the text is in small capital letters.
    pub small_caps: Option<bool>,

    /// The text's vertical offset from its normal position (superscript, subscript).
    /// Text with superscript/subscript is automatically rendered smaller.
    pub baseline_offset: Option<BaselineOffset>,

    /// The hyperlink destination of the text. If unset, there is no link.
    /// Links are not inherited from parent text.
    pub link: Option<Link>,

    /// Output only. The font family and rendered weight of the text.
    /// This indicates the actual font used to render the text, which may differ
    /// from `font_family`. This property is read-only.
    pub weighted_font_family: Option<WeightedFontFamily>, // Read-only
}

/// The text alignment for a paragraph.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#Alignment
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Alignment {
    /// Alignment is unspecified or inherited from the parent.
    AlignmentUnspecified,
    /// Aligned to the start of the line (left for LTR, right for RTL).
    Start,
    /// Centered.
    Center,
    /// Aligned to the end of the line (right for LTR, left for RTL).
    End,
    /// Text is stretched to fill the line (justified).
    Justified,
}

/// The direction of text paragraphs (Left-to-Right or Right-to-Left).
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#TextDirection
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextDirection {
    /// Text direction is unspecified or inherited. Defaults to LEFT_TO_RIGHT if not inherited.
    TextDirectionUnspecified,
    /// Left-to-right text direction.
    LeftToRight,
    /// Right-to-left text direction.
    RightToLeft,
}

/// The mode for controlling spacing between paragraphs.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#SpacingMode
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpacingMode {
    /// Spacing mode is unspecified or inherited.
    SpacingModeUnspecified,
    /// Prevent spacing between paragraphs from collapsing.
    NeverCollapse,
    /// Collapse spacing between consecutive paragraphs of the same style.
    CollapseLists,
}

/// Styles that apply to a whole paragraph.
/// If properties are unset, they may be inherited from a parent placeholder.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages/text#ParagraphStyle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParagraphStyle {
    /// The text alignment for this paragraph.
    pub alignment: Option<Alignment>,

    /// The text direction of this paragraph. Defaults to LEFT_TO_RIGHT if unset and not inherited.
    pub direction: Option<TextDirection>,

    /// The amount indentation for the paragraph on the side that corresponds to
    /// the end of the text, based on the current text direction.
    pub indent_end: Option<Dimension>,

    /// The amount of indentation for the start of the first line of the paragraph.
    pub indent_first_line: Option<Dimension>,

    /// The amount indentation for the paragraph on the side that corresponds to
    /// the start of the text, based on the current text direction.
    pub indent_start: Option<Dimension>,

    /// The amount of space between lines, as a percentage of normal (100.0 corresponds to 100%).
    pub line_spacing: Option<f32>,

    /// The amount of extra space above the paragraph.
    pub space_above: Option<Dimension>,

    /// The amount of extra space below the paragraph.
    pub space_below: Option<Dimension>,

    /// The spacing mode for the paragraph (COLLAPSE_LISTS or NEVER_COLLAPSE).
    pub spacing_mode: Option<SpacingMode>,
}

/// The properties of a Page common to all page types.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#PageProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageProperties {
    /// The background fill of the page. If unset, the background fill is inherited
    /// from a parent page if it exists. If the page has no parent, the fill defaults
    /// to the Slides editor default.
    pub page_background_fill: Option<PageBackgroundFill>,

    /// The color scheme of the page. If unset, the color scheme is inherited from
    /// a parent page. If the page has no parent, the color scheme uses a default
    /// Slides color scheme. Only the first 12 `ThemeColorType`s are editable.
    pub color_scheme: Option<ColorScheme>,
}

// --- Structs for Specific Page Types ---

/// The properties specific to a page with type `SLIDE`.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#SlideProperties
#[derive(Debug, Clone, Serialize, Deserialize)] // Removed PartialEq due to Box<Page>
#[serde(rename_all = "camelCase")]
pub struct SlideProperties {
    /// Output only. The object ID of the layout that this slide is based on.
    pub layout_object_id: Option<String>, // Read-only

    /// Output only. The object ID of the master that this slide is based on.
    pub master_object_id: Option<String>, // Read-only

    /// Output only. The notes page that this slide is associated with. Defines appearance
    /// for printing/exporting with speaker notes. Inherits properties from the notes master.
    /// The `BODY` placeholder shape contains the speaker notes (see `NotesProperties.speakerNotesObjectId`).
    /// The text content/styles of the notes shape are editable, but the page itself is read-only.
    /// Boxed to handle recursive type (`SlideProperties` -> `Page` -> `SlideProperties`).
    pub notes_page: Option<Box<Page>>, // Use Box for indirection; Read-only aspects within Page

    /// Whether the slide is skipped in the presentation mode. Defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_skipped: Option<bool>,
}

/// The properties specific to a page with type `LAYOUT`.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#LayoutProperties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutProperties {
    /// The object ID of the master that this layout is based on.
    pub master_object_id: Option<String>,
    /// The name of the layout (e.g., "TITLE_AND_BODY").
    pub name: Option<String>,
    /// Output only. The human-readable name of the layout (e.g., "Title and body").
    pub display_name: Option<String>, // Read-only
}

/// The properties specific to a page with type `NOTES`.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#NotesProperties
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash
#[serde(rename_all = "camelCase")]
pub struct NotesProperties {
    /// The object ID of the shape on this notes page that contains the speaker notes
    /// for the corresponding slide. The actual shape may not always exist until text is inserted.
    /// The `GetPresentation` or `GetPage` action will always return the latest object ID for this shape.
    pub speaker_notes_object_id: Option<String>,
}

/// The properties specific to a page with type `MASTER`.
/// Derived from: https://developers.google.com/slides/api/reference/rest/v1/presentations.pages#MasterProperties
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] // Added Eq, Hash
#[serde(rename_all = "camelCase")]
pub struct MasterProperties {
    /// Output only. The human-readable name of the master.
    pub display_name: Option<String>, // Read-only
}
