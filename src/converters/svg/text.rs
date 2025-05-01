//! Functions for converting text content (TextContent, TextRun, ParagraphMarker)
//! into SVG <text>/<tspan> elements or HTML for <foreignObject>, applying styles.

use super::{
    constants::*,
    error::Result,
    utils::{dimension_to_pt, escape_html_text, escape_svg_text, format_optional_color},
};
use crate::models::{
    colors::ColorScheme,
    properties::{Alignment, BaselineOffset, ParagraphStyle, TextStyle},
    text::TextContent,
    text_element::TextElementKind,
};
use std::fmt::Write;

/// Applies `TextStyle` properties to an SVG element's `style` attribute string.
/// (Used primarily for native SVG text rendering, may be less used if switching to HTML)
///
/// # Arguments
/// * `style` - An optional reference to the `TextStyle` to apply.
/// * `svg_style` - A mutable string buffer to append CSS style properties.
/// * `color_scheme` - An optional reference to the slide's `ColorScheme` for color resolution.
///
/// # Returns
/// A `Result<()>` indicating success or a formatting error.
pub(crate) fn apply_text_style(
    style: Option<&TextStyle>,
    svg_style: &mut String,
    color_scheme: Option<&ColorScheme>,
) -> Result<()> {
    if let Some(ts) = style {
        // Font Family
        write!(
            svg_style,
            "font-family:'{}'; ", // Enclose font family names in quotes
            ts.font_family.as_deref().unwrap_or(DEFAULT_FONT_FAMILY)
        )?;

        // Font Size
        let font_size_pt = dimension_to_pt(ts.font_size.as_ref());
        write!(
            svg_style,
            "font-size:{}pt; ",
            if font_size_pt > 0.0 {
                font_size_pt
            } else {
                DEFAULT_FONT_SIZE_PT // Use default if size is missing or zero
            }
        )?;

        // Foreground Color (SVG 'fill')
        let (fg_color, fg_opacity) =
            format_optional_color(ts.foreground_color.as_ref(), color_scheme);
        write!(
            svg_style,
            "fill:{}; fill-opacity:{}; ",
            fg_color, fg_opacity
        )?;

        // Background Color - Complex for <tspan>, often ignored or requires complex handling (e.g., background rects).
        // Let's ignore it for basic <tspan> styling.
        // let (bg_color, bg_opacity) = format_optional_color(ts.background_color.as_ref(), color_scheme);
        // if bg_color != "none" { /* ... */ }

        // Bold
        write!(
            svg_style,
            "font-weight:{}; ",
            if ts.bold.unwrap_or(false) {
                "bold"
            } else {
                "normal"
            }
        )?;

        // Italic
        write!(
            svg_style,
            "font-style:{}; ",
            if ts.italic.unwrap_or(false) {
                "italic"
            } else {
                "normal"
            }
        )?;

        // Underline / Strikethrough (SVG 'text-decoration')
        let mut decorations = Vec::new();
        if ts.underline.unwrap_or(false) {
            decorations.push("underline");
        }
        if ts.strikethrough.unwrap_or(false) {
            decorations.push("line-through");
        }
        let decorations_string = decorations.join(" ");
        let text_decoration = if decorations.is_empty() {
            "none"
        } else {
            decorations_string.as_str()
        };

        write!(svg_style, "text-decoration:{}; ", text_decoration)?;

        // Baseline Offset (Superscript/Subscript - SVG 'baseline-shift')
        match ts.baseline_offset {
            Some(BaselineOffset::Superscript) => write!(svg_style, "baseline-shift:super; ")?,
            Some(BaselineOffset::Subscript) => write!(svg_style, "baseline-shift:sub; ")?,
            _ => { /* Use default baseline, don't write attribute */ }
        }

        // Small Caps (SVG 'font-variant')
        write!(
            svg_style,
            "font-variant:{}; ",
            if ts.small_caps.unwrap_or(false) {
                "small-caps"
            } else {
                "normal"
            }
        )?;

        // NOTE: Link handling is omitted here as it requires <a href="..."> wrappers,
        // which complicates the basic style application. It might be handled at a higher level.
        // NOTE: weighted_font_family and language_code are also omitted for simplicity.
    } else {
        // Apply default styles if no specific style is provided? Or assume parent styles?
        // For now, if style is None, do nothing, relying on SVG defaults or parent styles.
    }
    Ok(())
}

/// Applies `ParagraphStyle` properties (mainly alignment) to an SVG `<text>` element's attributes.
/// Adjusts the 'x' coordinate based on text alignment and calculates the SVG `text-anchor`.
/// (Used primarily for native SVG text rendering)
///
/// # Arguments
/// * `style` - An optional reference to the `ParagraphStyle`.
/// * `svg_attrs` - A mutable string buffer to append SVG attributes like `text-anchor`.
/// * `x` - The original starting x-coordinate (usually the left edge) in points.
/// * `width` - The width of the text box in points.
///
/// # Returns
/// A `Result<f64>` containing the *adjusted* x-coordinate based on the text anchor,
/// or a formatting error.
pub(crate) fn apply_paragraph_style(
    style: Option<&ParagraphStyle>,
    svg_attrs: &mut String,
    x: f64,
    width: f64,
) -> Result<f64> {
    let mut adjusted_x = x;
    let mut text_anchor = "start"; // SVG default: text starts at the specified 'x'

    if let Some(ps) = style {
        match ps.alignment {
            Some(Alignment::Center) => {
                text_anchor = "middle"; // Anchor text horizontally centered on 'x'
                adjusted_x = x + width / 2.0; // Adjust x to be the midpoint
            }
            Some(Alignment::End) => {
                text_anchor = "end"; // Anchor text with its end at 'x'
                adjusted_x = x + width; // Adjust x to be the right edge
            }
            Some(Alignment::Justified) => {
                // Justification is complex in SVG and often poorly supported.
                // CSS `text-align: justify` exists but might not work reliably within SVG <text>.
                // Fallback to 'start' alignment for broader compatibility.
                text_anchor = "start";
                adjusted_x = x;
                // Optionally, could add 'text-align:justify;' to the style attribute, but results vary.
            }
            _ => {
                // Alignment::Start or None
                text_anchor = "start";
                adjusted_x = x;
            }
        }
        // Other ParagraphStyle properties like indentation and spacing are difficult to map
        // directly to SVG <text> attributes without complex manual line breaking and positioning.
        // These are ignored in this basic conversion.
    }

    write!(svg_attrs, r#" text-anchor="{}""#, text_anchor)?;
    Ok(adjusted_x) // Return the x-coordinate that corresponds to the calculated anchor
}

/// Merges two `TextStyle` instances, where `specific_style` overrides `inherited_style`.
/// Properties set in `specific_style` take precedence. If a property is `None` in
/// `specific_style`, the value from `inherited_style` is used.
///
/// # Arguments
/// * `specific_style` - The overriding style (e.g., from a TextRun).
/// * `inherited_style` - The base style (e.g., from a ParagraphMarker bullet or placeholder).
///
/// # Returns
/// A new `TextStyle` instance representing the merged style.
pub(crate) fn merge_text_styles(
    specific_style: Option<&TextStyle>,
    inherited_style: Option<&TextStyle>,
) -> TextStyle {
    // Start with the inherited style or a default TextStyle if none provided.
    let mut merged = inherited_style.cloned().unwrap_or_default();

    if let Some(specific) = specific_style {
        // Iterate through specific style properties and override if they are Some
        if specific.background_color.is_some() {
            merged.background_color = specific.background_color.clone();
        }
        if specific.baseline_offset.is_some() {
            merged.baseline_offset = specific.baseline_offset.clone();
        }
        if specific.bold.is_some() {
            merged.bold = specific.bold;
        }
        if specific.font_family.is_some() {
            merged.font_family = specific.font_family.clone();
        }
        if specific.font_size.is_some() {
            merged.font_size = specific.font_size.clone();
        }
        if specific.foreground_color.is_some() {
            merged.foreground_color = specific.foreground_color.clone();
        }
        if specific.italic.is_some() {
            merged.italic = specific.italic;
        }
        if specific.link.is_some() {
            merged.link = specific.link.clone();
        }
        if specific.small_caps.is_some() {
            merged.small_caps = specific.small_caps;
        }
        if specific.strikethrough.is_some() {
            merged.strikethrough = specific.strikethrough;
        }
        if specific.underline.is_some() {
            merged.underline = specific.underline;
        }
        if specific.weighted_font_family.is_some() {
            merged.weighted_font_family = specific.weighted_font_family.clone();
        }
        // language_code is typically set explicitly, merging might not be desired.
        // if specific.language_code.is_some() { merged.language_code = specific.language_code.clone(); }
    }

    merged
}

// Keep convert_text_content_to_svg for potential future use or other element types
// that might benefit from native SVG text, but it's no longer called by convert_shape_to_svg.
/// Converts the `TextContent` of a shape or table cell into SVG `<text>` and `<tspan>` elements.
/// Handles basic paragraph breaks, text runs with styling, and alignment.
/// Applies inheritance logic for text styles (placeholder -> paragraph -> text run).
///
/// Note: This implementation uses a simplified approach for line breaks and positioning.
/// It creates a new `<text>` element for the start of each paragraph (after a ParagraphMarker
/// or implicit start) and assumes subsequent runs *within the same line* are not handled
/// accurately (they might overprint or be skipped). Newlines within a TextRun also force a
/// new line, potentially breaking style continuity if not handled carefully.
///
/// # Arguments
/// * `text_content` - Reference to the `TextContent` containing text elements.
/// * `effective_paragraph_style` - The initial `ParagraphStyle` (alignment) inherited from the container/placeholder.
/// * `effective_text_style_base` - The base `TextStyle` (font, color) inherited from the container/placeholder.
/// * `transform_x`, `transform_y` - Top-left corner coordinates (in points) for the text block.
/// * `element_width`, `element_height` - Dimensions (in points) of the text block container. `element_height` is currently unused.
/// * `color_scheme` - The active `ColorScheme` for resolving theme colors.
/// * `svg_output` - Mutable string buffer to append the generated SVG markup.
///
/// # Returns
/// `Result<()>` indicating success or a formatting error.
#[allow(clippy::too_many_arguments, dead_code)] // Keep but mark as dead code for now
pub(crate) fn convert_text_content_to_svg(
    text_content: &TextContent,
    effective_paragraph_style: Option<&ParagraphStyle>, // Initial alignment etc.
    effective_text_style_base: &TextStyle,              // Base font, color etc.
    transform_x: f64,
    transform_y: f64,
    element_width: f64,
    _element_height: f64, // Currently unused, could be used for vertical alignment/clipping
    color_scheme: Option<&ColorScheme>,
    svg_output: &mut String,
) -> Result<()> {
    let text_elements = match &text_content.text_elements {
        Some(elements) => elements,
        None => return Ok(()), // No text elements, nothing to render
    };

    // --- State for tracking paragraphs and vertical position ---
    // Base text style for the *current* paragraph (can be modified by bullets)
    let mut current_paragraph_base_style = effective_text_style_base.clone();
    // Paragraph style (alignment) for the *current* paragraph
    let mut current_para_style_ref = effective_paragraph_style;

    let mut current_y = transform_y; // Tracks the baseline Y position for the next line/paragraph
    let mut first_line_in_paragraph = true; // Flag to control creation of new <text> vs <tspan>

    for element in text_elements {
        // Estimate line height based on the *current paragraph's base* font size.
        // This allows line height to adapt if a bullet point changes the base size.
        let current_base_font_size_pt =
            dimension_to_pt(current_paragraph_base_style.font_size.as_ref());
        let line_height_pt = if current_base_font_size_pt > 0.0 {
            current_base_font_size_pt * 1.2 // Simple line height estimate (120%)
        } else {
            DEFAULT_FONT_SIZE_PT * 1.2 // Fallback if base size is unknown
        };

        match &element.kind {
            Some(TextElementKind::ParagraphMarker(pm)) => {
                // Reached the end of a paragraph (or start of a new one).
                if !first_line_in_paragraph {
                    // Move Y position down for the next paragraph if this wasn't the very first marker.
                    // TODO: Add paragraph spacing from ParagraphStyle if needed.
                    current_y += line_height_pt;
                }
                first_line_in_paragraph = true; // The next TextRun will start a new <text> element

                // Update paragraph style (alignment) based on this marker.
                current_para_style_ref = pm.style.as_ref().or(effective_paragraph_style);

                // Update the base text style for this paragraph if the bullet has its own style.
                if let Some(bullet) = &pm.bullet {
                    if let Some(bullet_style) = &bullet.bullet_style {
                        // Merge bullet style onto the original placeholder base style
                        current_paragraph_base_style =
                            merge_text_styles(Some(bullet_style), Some(effective_text_style_base));
                    } else {
                        // No specific bullet style, reset to the original placeholder base
                        current_paragraph_base_style = effective_text_style_base.clone();
                    }
                } else {
                    // No bullet, reset to the original placeholder base
                    current_paragraph_base_style = effective_text_style_base.clone();
                }
            }
            Some(TextElementKind::TextRun(tr)) => {
                let content = tr.content.as_deref().unwrap_or("");
                if content.is_empty() {
                    continue;
                } // Skip empty runs silently

                // Determine the final style for this specific run by merging its style
                // onto the current paragraph's base style (which might include bullet styling).
                let final_run_style =
                    merge_text_styles(tr.style.as_ref(), Some(&current_paragraph_base_style));

                // Get the font size for this specific run for vertical alignment adjustment.
                let final_font_size_pt = dimension_to_pt(final_run_style.font_size.as_ref());
                let effective_font_size_pt = if final_font_size_pt > 0.0 {
                    final_font_size_pt
                } else {
                    DEFAULT_FONT_SIZE_PT
                };

                // Apply the final style to SVG attributes
                let mut text_style_attr = String::new();
                apply_text_style(Some(&final_run_style), &mut text_style_attr, color_scheme)?;

                if first_line_in_paragraph {
                    // Start a new <text> element for the first run in a paragraph.
                    let mut para_attrs = String::new();
                    // Apply alignment (text-anchor) and get the adjusted X coordinate.
                    let adjusted_x = apply_paragraph_style(
                        current_para_style_ref,
                        &mut para_attrs,
                        transform_x,
                        element_width,
                    )?;

                    // Adjust y position for baseline alignment. SVG <text> y attribute sets the baseline.
                    // Adding the font size shifts the baseline down, placing the text visually near the top.
                    // This is a common convention but might need refinement based on font metrics.
                    let y_pos = current_y + effective_font_size_pt;

                    // Write the opening <text> tag with position, alignment, and style.
                    write!(
                        svg_output,
                        r#"<text x="{}" y="{}"{}"#, // Use adjusted X and baseline Y
                        adjusted_x, y_pos, para_attrs
                    )?;
                    write!(svg_output, r#" style="{}">"#, text_style_attr.trim_end())?; // Apply run-specific styles

                    // Write the escaped text content. Handle newlines within the run.
                    write_escaped_text_with_newlines(content, svg_output)?;

                    write!(svg_output, "</text>")?; // Close the <text> element

                    first_line_in_paragraph = false; // Subsequent runs in this paragraph (if handled) would be tspans.

                    // If the content ended with a newline, prepare Y for the next line.
                    if content.ends_with('\n') {
                        current_y += line_height_pt;
                        first_line_in_paragraph = true; // Newline forces next run to start a new <text>
                    }
                } else {
                    // --- Handling of subsequent runs within the same line ---
                    // This part is tricky in SVG without manual layout.
                    // Option 1: Skip subsequent runs (current behavior). Leads to missing text.
                    // Option 2: Append as <tspan> without explicit positioning. Might overlap or look wrong.
                    // Option 3: Attempt to calculate dx/dy (complex).
                    eprintln!("Warning: Subsequent TextRuns on the same line currently skipped (Object ID context missing). Content: '{}'", content);
                    // Example for Option 2 (uncomment if needed, but likely imperfect):
                    // write!(svg_output, r#"<tspan style="{}">"#, text_style_attr.trim_end())?;
                    // write_escaped_text_with_newlines(content, svg_output)?;
                    // write!(svg_output, "</tspan>")?;
                    if content.ends_with('\n') {
                        current_y += line_height_pt;
                        first_line_in_paragraph = true;
                    }
                }
            }
            Some(TextElementKind::AutoText(at)) => {
                // AutoText (like slide numbers) is treated similarly to TextRun.
                let content = at.content.as_deref().unwrap_or("");
                if content.is_empty() {
                    continue;
                }

                let final_autotext_style =
                    merge_text_styles(at.style.as_ref(), Some(&current_paragraph_base_style));

                let final_autotext_font_size_pt =
                    dimension_to_pt(final_autotext_style.font_size.as_ref());
                let effective_font_size_pt = if final_autotext_font_size_pt > 0.0 {
                    final_autotext_font_size_pt
                } else {
                    DEFAULT_FONT_SIZE_PT
                };

                let mut text_style_attr = String::new();
                apply_text_style(
                    Some(&final_autotext_style),
                    &mut text_style_attr,
                    color_scheme,
                )?;

                if first_line_in_paragraph {
                    let mut para_attrs = String::new();
                    let adjusted_x = apply_paragraph_style(
                        current_para_style_ref,
                        &mut para_attrs,
                        transform_x,
                        element_width,
                    )?;
                    let y_pos = current_y + effective_font_size_pt; // Baseline adjustment

                    write!(
                        svg_output,
                        r#"<text x="{}" y="{}"{}"#,
                        adjusted_x, y_pos, para_attrs
                    )?;
                    write!(svg_output, r#" style="{}">"#, text_style_attr.trim_end())?;
                    write_escaped_text_with_newlines(content, svg_output)?;
                    write!(svg_output, "</text>")?;
                    first_line_in_paragraph = false;
                    if content.ends_with('\n') {
                        current_y += line_height_pt;
                        first_line_in_paragraph = true;
                    }
                } else {
                    eprintln!("Warning: Subsequent AutoText on the same line currently skipped. Content: '{}'", content);
                    if content.ends_with('\n') {
                        current_y += line_height_pt;
                        first_line_in_paragraph = true;
                    }
                }
            }
            None => { /* Element kind is None, skip silently */ }
        }
    }

    Ok(())
}

/// Helper function to write escaped text, handling internal newlines by creating <tspan> elements.
/// This is a very basic way to handle newlines within a single TextRun/AutoText.
/// (Used primarily for native SVG text rendering)
#[allow(dead_code)] // Keep but mark as dead code for now
fn write_escaped_text_with_newlines(text: &str, svg_output: &mut String) -> Result<()> {
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            // For subsequent lines, create a tspan with dy to move down.
            // Using "1.2em" assumes line height is roughly 1.2 times font size.
            // TODO: Use calculated line_height_pt if available and convert to em or use absolute dy.
            write!(svg_output, r#"<tspan x="{}" dy="1.2em">"#, 0)?; // Reset x=0 relative to parent <text>
        }
        write!(svg_output, "{}", escape_svg_text(line))?;
        if i > 0 {
            write!(svg_output, "</tspan>")?;
        }
    }
    // Handle case where text ends with newline(s) - lines() might omit trailing empty strings.
    if text.ends_with('\n') && lines.last().map_or(true, |l| !l.is_empty()) {
        write!(svg_output, r#"<tspan x="{}" dy="1.2em"></tspan>"#, 0)?;
    }
    Ok(())
}

/// Converts the `TextContent` of a shape or table cell into basic, styled HTML
/// suitable for embedding within an SVG `<foreignObject>`.
/// Styles TextRuns using inline CSS within `<span>` elements.
/// Paragraph markers create `<p>` tags with alignment.
/// Handles style inheritance (placeholder/base -> paragraph -> text run).
///
/// # Arguments
/// * `text_content` - Reference to the `TextContent` containing text elements.
/// * `effective_paragraph_style` - The initial `ParagraphStyle` (alignment) inherited.
/// * `effective_text_style_base` - The base `TextStyle` (font, color) inherited.
/// * `color_scheme` - The active `ColorScheme` for resolving theme colors.
/// * `html_output` - Mutable string buffer to append the generated HTML markup.
///
/// # Returns
/// `Result<()>` indicating success or a formatting error.
#[allow(unused_assignments)]
pub(crate) fn convert_text_content_to_html(
    text_content: &TextContent,
    effective_paragraph_style: Option<&ParagraphStyle>, // Inherited alignment etc.
    effective_text_style_base: &TextStyle,              // Inherited base style
    color_scheme: Option<&ColorScheme>,
    html_output: &mut String,
) -> Result<()> {
    let text_elements = match &text_content.text_elements {
        Some(elements) => elements,
        None => return Ok(()),
    };

    // --- State for tracking paragraphs and styles ---
    let mut paragraph_open = false;
    let mut first_element_processed = false;
    // Base text style for the *current* paragraph (can be modified by bullets)
    let mut current_paragraph_base_style = effective_text_style_base.clone();
    // Paragraph style (alignment) for the *current* paragraph
    let mut current_para_style_ref = effective_paragraph_style;

    for element in text_elements {
        match &element.kind {
            Some(TextElementKind::ParagraphMarker(pm)) => {
                // Close previous paragraph if open
                if paragraph_open {
                    write!(html_output, "</p>")?;
                    paragraph_open = false;
                }
                // Start new paragraph. Add newline before unless it's the very first element.
                if first_element_processed {
                    writeln!(html_output)?;
                }

                // Update paragraph style (alignment) based on this marker.
                current_para_style_ref = pm.style.as_ref().or(effective_paragraph_style);
                // Update the base text style for this paragraph if the bullet has its own style.
                if let Some(bullet) = &pm.bullet {
                    if let Some(bullet_style) = &bullet.bullet_style {
                        current_paragraph_base_style =
                            merge_text_styles(Some(bullet_style), Some(effective_text_style_base));
                    } else {
                        current_paragraph_base_style = effective_text_style_base.clone();
                    }
                } else {
                    current_paragraph_base_style = effective_text_style_base.clone();
                }

                // Build paragraph style string (including alignment)
                let mut p_style = "margin:0; padding:0;".to_string(); // Base style
                if let Some(ps) = current_para_style_ref {
                    let text_align = match ps.alignment {
                        Some(Alignment::Center) => "center",
                        Some(Alignment::End) => "end", // or "right" if better browser support needed
                        Some(Alignment::Justified) => "justify",
                        _ => "start", // Start or None
                    };
                    write!(p_style, " text-align:{};", text_align)?;
                    // TODO: Handle indentation, spacingBefore/After etc. from ParagraphStyle
                }

                write!(html_output, "<p style=\"{}\">", p_style)?;
                writeln!(html_output)?; // Newline after opening tag for readability
                paragraph_open = true;
                first_element_processed = true;
            }
            Some(TextElementKind::TextRun(tr)) => {
                let content = tr.content.as_deref().unwrap_or("");
                if content.is_empty() {
                    continue;
                } // Skip empty runs

                // Ensure we are inside a paragraph tag (start one if needed)
                if !paragraph_open {
                    if first_element_processed {
                        writeln!(html_output)?;
                    }
                    // Apply current paragraph style if starting a new one implicitly
                    let mut p_style = "margin:0; padding:0;".to_string();
                    if let Some(ps) = current_para_style_ref {
                        let text_align = match ps.alignment {
                            Some(Alignment::Center) => "center",
                            Some(Alignment::End) => "end",
                            Some(Alignment::Justified) => "justify",
                            _ => "start",
                        };
                        write!(p_style, " text-align:{};", text_align)?;
                    }
                    write!(html_output, "<p style=\"{}\">", p_style)?;
                    writeln!(html_output)?;
                    paragraph_open = true;
                }

                // Determine the final style for this specific run by merging
                let final_run_style =
                    merge_text_styles(tr.style.as_ref(), Some(&current_paragraph_base_style));

                // Build inline CSS style for the span based on the final run style
                let mut span_style = String::new();
                apply_html_text_style(Some(&final_run_style), &mut span_style, color_scheme)?; // Use helper

                // Escape HTML content and replace internal newlines with <br> tags
                let html_content = escape_html_text(content).replace('\n', "<br/>\n        "); // Add newline+indent after <br>

                // Write the span with its style and content
                write!(html_output, "      ")?; // Indent span within paragraph
                if !span_style.is_empty() {
                    write!(
                        html_output,
                        r#"<span style="{}">{}</span>"#,
                        span_style.trim_end(),
                        html_content
                    )?;
                } else {
                    // No specific style, just write content directly
                    write!(html_output, "{}", html_content)?;
                }
                writeln!(html_output)?; // Newline after closing span
                first_element_processed = true;
            }
            Some(TextElementKind::AutoText(at)) => {
                // Treat AutoText similarly to TextRun for HTML conversion
                let content = at.content.as_deref().unwrap_or("");
                if content.is_empty() {
                    continue;
                }

                // Ensure inside paragraph
                if !paragraph_open {
                    if first_element_processed {
                        writeln!(html_output)?;
                    }
                    let mut p_style = "margin:0; padding:0;".to_string();
                    if let Some(ps) = current_para_style_ref {
                        let text_align = match ps.alignment {
                            Some(Alignment::Center) => "center",
                            Some(Alignment::End) => "end",
                            Some(Alignment::Justified) => "justify",
                            _ => "start",
                        };
                        write!(p_style, " text-align:{};", text_align)?;
                    }
                    write!(html_output, "<p style=\"{}\">", p_style)?;
                    writeln!(html_output)?;
                    paragraph_open = true;
                }

                // Merge styles and apply
                let final_autotext_style =
                    merge_text_styles(at.style.as_ref(), Some(&current_paragraph_base_style));
                let mut span_style = String::new();
                apply_html_text_style(Some(&final_autotext_style), &mut span_style, color_scheme)?;

                let html_content = escape_html_text(content).replace('\n', "<br/>\n        ");

                write!(html_output, "      ")?; // Indent
                if !span_style.is_empty() {
                    write!(
                        html_output,
                        r#"<span style="{}">{}</span>"#,
                        span_style.trim_end(),
                        html_content
                    )?;
                } else {
                    write!(html_output, "{}", html_content)?;
                }
                writeln!(html_output)?;
                first_element_processed = true;
            }
            None => {}
        }
    }

    // Close the last paragraph tag if it was open
    if paragraph_open {
        write!(html_output, "</p>")?;
        // Do not add a final newline here, let the caller handle overall formatting
    }

    // Trim potential leading/trailing whitespace that might affect layout in foreignObject
    // Trim the final string *before* returning using trim().to_string() approach.
    let trimmed_output = html_output.trim().to_string();
    html_output.clear();
    write!(html_output, "{}", trimmed_output)?; // Write back the trimmed version

    Ok(())
}

/// Helper to apply TextStyle properties to an HTML element's inline `style` attribute.
///
/// # Arguments
/// * `style` - An optional reference to the `TextStyle` to apply.
/// * `html_style` - A mutable string buffer to append CSS style properties.
/// * `color_scheme` - An optional reference to the slide's `ColorScheme`.
///
/// # Returns
/// A `Result<()>` indicating success or a formatting error.
fn apply_html_text_style(
    style: Option<&TextStyle>,
    html_style: &mut String,
    color_scheme: Option<&ColorScheme>,
) -> Result<()> {
    if let Some(ts) = style {
        // Font Family
        write!(
            html_style,
            "font-family:'{}'; ",
            ts.font_family.as_deref().unwrap_or(DEFAULT_FONT_FAMILY)
        )?;
        // Font Size
        let font_size_pt = dimension_to_pt(ts.font_size.as_ref());
        write!(
            html_style,
            "font-size:{}pt; ",
            if font_size_pt > 0.0 {
                font_size_pt
            } else {
                DEFAULT_FONT_SIZE_PT
            }
        )?;
        // Foreground Color (HTML 'color')
        let (fg_color, _) = format_optional_color(ts.foreground_color.as_ref(), color_scheme);
        if fg_color != "none" {
            // Avoid writing color:none;
            write!(html_style, "color:{}; ", fg_color)?;
        }
        // Background Color (HTML 'background-color')
        let (bg_color, _) = format_optional_color(ts.background_color.as_ref(), color_scheme);
        if bg_color != "none" {
            write!(html_style, "background-color:{}; ", bg_color)?;
        }
        // Bold
        if ts.bold.unwrap_or(false) {
            write!(html_style, "font-weight:bold; ")?;
        }
        // Italic
        if ts.italic.unwrap_or(false) {
            write!(html_style, "font-style:italic; ")?;
        }
        // Underline/Strikethrough
        let mut decorations = Vec::new();
        if ts.underline.unwrap_or(false) {
            decorations.push("underline");
        }
        if ts.strikethrough.unwrap_or(false) {
            decorations.push("line-through");
        }
        if !decorations.is_empty() {
            write!(html_style, "text-decoration:{}; ", decorations.join(" "))?;
        }
        // Baseline Offset (HTML 'vertical-align' + font-size adjustment)
        match ts.baseline_offset {
            Some(BaselineOffset::Superscript) => {
                write!(html_style, "vertical-align:super; font-size:smaller; ")?
            }
            Some(BaselineOffset::Subscript) => {
                write!(html_style, "vertical-align:sub; font-size:smaller; ")?
            }
            _ => {}
        }
        // Small Caps
        if ts.small_caps.unwrap_or(false) {
            write!(html_style, "font-variant:small-caps; ")?;
        }
        // Link - Add specific handling if links should be rendered as <a> tags
        // if let Some(link) = &ts.link { ... }
    }
    Ok(())
}
