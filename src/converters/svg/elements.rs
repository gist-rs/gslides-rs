//! Handles the conversion of specific `PageElement` types (Shape, Table, Group, Line, Image)
//! into their corresponding SVG representations.

use super::{
    constants::*,
    error::Result, // Keep SvgConversionError if needed for specific errors here
    structure::{
        find_placeholder_element, get_placeholder_default_text_style, ElementsMap, LayoutsMap,
        MastersMap,
    },
    text::{convert_text_content_to_html, convert_text_content_to_svg},
    utils::{apply_transform, dimension_to_pt, escape_svg_text, format_color, AsShape},
};
use crate::models::{
    colors::ColorScheme,
    common::{AffineTransform, Dimension, Size},
    elements::{PageElement, PageElementKind},
    image::Image,
    line::{Line, LineFillContent},
    properties::{ParagraphStyle, TextStyle},
    shape::Shape,
    shape_properties::DashStyle,
    table::Table,
    text_element::TextElementKind, // Required for checking ParagraphMarker in shape style override
};
use std::fmt::Write;

/// Converts a Shape element (including text boxes) to an SVG fragment.
/// Handles applying transform, size, resolving placeholder styles, and rendering text content.
///
/// # Arguments
/// * `shape` - The `Shape` data.
/// * `transform` - The element's `AffineTransform`.
/// * `size` - The element's `Size`.
/// * `slide_layout_id` - The object ID of the slide's layout (for placeholder lookup).
/// * `layouts_map`, `masters_map`, `elements_map` - Lookup maps.
/// * `color_scheme` - The active `ColorScheme`.
/// * `svg_output` - Mutable string buffer for SVG output.
///
/// # Returns
/// `Result<()>`
#[allow(clippy::too_many_arguments)]
fn convert_shape_to_svg(
    shape: &Shape,
    transform: Option<&AffineTransform>,
    size: Option<&Size>,
    slide_layout_id: Option<&str>,
    layouts_map: &LayoutsMap,
    masters_map: &MastersMap,
    elements_map: &ElementsMap,
    color_scheme: Option<&ColorScheme>, // <-- Added color_scheme argument
    svg_output: &mut String,
) -> Result<()> {
    let mut shape_attrs = String::new();
    let (tx, ty) = apply_transform(transform, &mut shape_attrs)?;
    let width = dimension_to_pt(size.and_then(|s| s.width.as_ref()));
    let height = dimension_to_pt(size.and_then(|s| s.height.as_ref()));

    // Resolve effective styles
    let mut effective_text_style_base = TextStyle::default();
    // Separate paragraph style (for alignment, etc.) from text style (font, color, etc.)
    let mut effective_paragraph_style: Option<ParagraphStyle> = None;

    if let Some(placeholder) = &shape.placeholder {
        if let Some(layout_id) = slide_layout_id {
            if let Some(placeholder_element) = find_placeholder_element(
                placeholder,
                layout_id,
                layouts_map,
                masters_map,
                elements_map,
            ) {
                // [+] Get the *default text style* from the placeholder shape
                if let Some(placeholder_base_style) =
                    get_placeholder_default_text_style(placeholder_element)
                {
                    effective_text_style_base = placeholder_base_style;
                }

                // [+] Get the *default paragraph style* (alignment, etc.) from the placeholder
                if let Some(placeholder_shape) = placeholder_element.element_kind.as_shape() {
                    if let Some(text) = &placeholder_shape.text {
                        if let Some(elements) = &text.text_elements {
                            for element in elements {
                                // Find the first paragraph marker to get default para style
                                if let Some(TextElementKind::ParagraphMarker(pm)) = &element.kind {
                                    if let Some(style) = &pm.style {
                                        effective_paragraph_style = Some(style.clone());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Render text content if present
    // Pass the resolved base text style, the initial paragraph style, AND color scheme
    if let Some(text) = &shape.text {
        // Check if the shape itself defines a paragraph style (overriding placeholder)
        if let Some(elements) = &text.text_elements {
            for element in elements {
                if let Some(TextElementKind::ParagraphMarker(pm)) = &element.kind {
                    if let Some(style) = &pm.style {
                        effective_paragraph_style = Some(style.clone());
                        break; // Use the first one found in the shape itself
                    }
                }
            }
        }

        // Pass the resolved base text style and the (potentially overridden) paragraph style
        convert_text_content_to_svg(
            text,
            effective_paragraph_style.as_ref(),
            &effective_text_style_base,
            tx,
            ty,
            width,
            height,
            color_scheme, // Pass scheme
            svg_output,
        )?;
    }

    Ok(())
}

/// Converts a Table element to SVG using `<foreignObject>` to embed styled HTML content.
/// Handles transform, size, basic cell styling (border, background), and cell text content.
///
/// # Arguments
/// * `table` - The `Table` data.
/// * `transform`, `size` - Element's transform and size.
/// * `color_scheme` - Active `ColorScheme`.
/// * `svg_output` - Mutable string buffer.
///
/// # Returns
/// `Result<()>`
#[allow(clippy::too_many_arguments)]
fn convert_table_to_svg(
    table: &Table,
    transform: Option<&AffineTransform>,
    size: Option<&Size>,
    color_scheme: Option<&ColorScheme>,
    svg_output: &mut String,
) -> Result<()> {
    let mut foreign_object_attrs = String::new();
    let (tx, ty) = apply_transform(transform, &mut foreign_object_attrs)?;
    let width = dimension_to_pt(size.and_then(|s| s.width.as_ref()));
    let height = dimension_to_pt(size.and_then(|s| s.height.as_ref()));

    // Avoid creating empty or invalid foreignObjects
    if width <= 0.0 || height <= 0.0 {
        eprintln!(
            "Warning: Skipping table with zero or negative dimensions ({}x{}pt).",
            width, height
        );
        return Ok(());
    }

    // --- <foreignObject> Setup ---
    write!(
        svg_output,
        r#"<foreignObject x="{}" y="{}" width="{}" height="{}"{}>"#, // Apply transform to foreignObject
        tx, ty, width, height, foreign_object_attrs
    )?;
    writeln!(svg_output)?; // Newline after opening tag

    // --- HTML Content within <foreignObject> ---
    // XHTML namespace is crucial for proper rendering within SVG
    write!(
        svg_output,
        r#"  <div xmlns="http://www.w3.org/1999/xhtml" style="width:100%; height:100%; overflow:hidden;">"#
    )?;
    writeln!(svg_output)?; // Newline after opening div

    // Basic table styling
    write!(
        svg_output,
        r#"    <table style="border-collapse: collapse; width:100%; height:100%; border: 1px solid #ccc; table-layout: fixed;">"#
    )?; // fixed layout helps with sizing

    // --- Table Rows and Cells ---
    if let Some(rows) = &table.table_rows {
        for row in rows {
            writeln!(svg_output)?; // Newline before <tr>
            write!(svg_output, "      <tr>")?; // No newline after opening <tr> yet

            if let Some(cells) = &row.table_cells {
                if !cells.is_empty() {
                    writeln!(svg_output)?;
                } // Newline before first <td> if cells exist

                for cell in cells {
                    // Handle colspan and rowspan
                    let colspan = cell.column_span.unwrap_or(1);
                    let rowspan = cell.row_span.unwrap_or(1);
                    let mut td_attrs = String::new();
                    if colspan > 1 {
                        write!(td_attrs, r#" colspan="{}""#, colspan)?;
                    }
                    if rowspan > 1 {
                        write!(td_attrs, r#" rowspan="{}""#, rowspan)?;
                    }

                    // Basic cell styling + background fill
                    let mut cell_style = "border: 1px solid #eee; padding: 3pt; vertical-align: top; overflow: hidden;".to_string(); // Added overflow:hidden
                    if let Some(props) = &cell.table_cell_properties {
                        if let Some(bg_fill) = &props.table_cell_background_fill {
                            // Only handle solid fill for now
                            if let Some(solid) = &bg_fill.solid_fill {
                                // Use format_color which handles theme colors
                                let bg_color_hex = format_color(solid.color.as_ref(), color_scheme);
                                // TODO: Handle solid_fill.alpha if present (using rgba?)
                                write!(cell_style, " background-color:{};", bg_color_hex)?;
                            }
                        }
                        // TODO: Handle contentAlignment (vertical-align: middle/bottom)
                        // match props.content_alignment { ... }
                    }

                    // Write opening <td> tag
                    write!(
                        svg_output,
                        "        <td{} style=\"{}\">",
                        td_attrs, cell_style
                    )?; // Indent <td>

                    // Convert and write cell text content using HTML converter
                    if let Some(text) = &cell.text {
                        // Assuming convert_text_content_to_html adds necessary internal structure (<p>, <span>)
                        // and handles its own indentation/newlines appropriately relative to the <td>.
                        convert_text_content_to_html(text, color_scheme, svg_output)?;
                    } else {
                        // Ensure non-breaking space for empty cells to maintain borders/layout
                        write!(svg_output, "&nbsp;")?;
                    }

                    // Write closing </td> tag
                    write!(svg_output, "</td>")?;
                    writeln!(svg_output)?; // Newline after closing <td>
                }
                // Add indentation before closing </tr> if cells existed
                if !cells.is_empty() {
                    write!(svg_output, "      ")?;
                }
            }
            // Close </tr> tag (already indented if cells existed)
            write!(svg_output, "</tr>")?;
        }
        // Add newline+indentation before closing </table> if rows existed
        if !rows.is_empty() {
            writeln!(svg_output)?;
            write!(svg_output, "    ")?;
        }
    } else {
        // Handle empty table? Maybe add a placeholder row/cell?
    }

    // --- Closing Tags ---
    write!(svg_output, "</table>")?;
    writeln!(svg_output)?; // Newline after closing table
    write!(svg_output, "  </div>")?;
    writeln!(svg_output)?; // Newline after closing div
    write!(svg_output, "</foreignObject>")?; // No final newline here, let the caller add it.

    Ok(())
}

/// Converts an Image element to an SVG `<image>` tag.
/// Handles transform, size, and uses `contentUrl` for the image source.
/// Includes a fallback rectangle if the URL is missing.
///
/// # Arguments
/// * `image_data` - The `Image` data containing the `contentUrl`.
/// * `element_id` - The object ID (for potential logging).
/// * `transform`, `size` - Element's transform and size.
/// * `svg_output` - Mutable string buffer.
///
/// # Returns
/// `Result<()>`
fn convert_image_to_svg(
    image_data: &Image,
    element_id: &str,
    transform: Option<&AffineTransform>,
    size: Option<&Size>,
    svg_output: &mut String,
) -> Result<()> {
    let mut img_attrs = String::new();
    let (tx, ty) = apply_transform(transform, &mut img_attrs)?; // Transform applied to the <image> tag
    let width = dimension_to_pt(size.and_then(|s| s.width.as_ref()));
    let height = dimension_to_pt(size.and_then(|s| s.height.as_ref()));

    if width <= 0.0 || height <= 0.0 {
        eprintln!(
            "Warning: Skipping image element {} with zero dimensions.",
            element_id
        );
        return Ok(());
    }

    if let Some(url) = &image_data.content_url {
        // Ensure URL is properly escaped for XML attribute context if needed (though standard URLs are usually safe)
        // let safe_url = escape_xml_attribute(url); // You'd need an escape function for ",&,<,> etc.
        let safe_url = url; // Assuming URL is safe for now

        // Render the actual image using the contentUrl.
        // `preserveAspectRatio="xMidYMid meet"` scales the image to fit while preserving aspect ratio.
        write!(
            svg_output,
            r#"<image x="{}" y="{}" width="{}" height="{}" xlink:href="{}"{} preserveAspectRatio="xMidYMid meet" />"#, // Use xlink:href for broader compatibility
            tx, ty, width, height, safe_url, img_attrs
        )?;
    } else {
        // Fallback if no URL is provided - render a placeholder rectangle with text.
        eprintln!(
            "Warning: Image element {} is missing contentUrl.",
            element_id
        );
        // Apply transform to the placeholder group
        write!(svg_output, "<g{}>", img_attrs)?;
        write!(
            svg_output,
            r#"<rect width="{}" height="{}" style="fill:#e0e0e0; stroke:gray; fill-opacity:0.5;" />"#,
            width,
            height // Positioned at (0,0) within the transformed group
        )?;
        write!(
            svg_output,
            r#"<text x="2" y="2" dy="1em" style="font-family:sans-serif; font-size:8pt; fill:gray;">Image Missing URL</text>"#
        )?;
        write!(svg_output, "</g>")?;
    }
    Ok(())
}

/// Converts a Line element to an SVG `<line>` tag.
/// Calculates start/end points based on transform and size, and applies line styling.
///
/// # Arguments
/// * `line_data` - The `Line` data containing properties.
/// * `element_id` - The object ID (for potential logging).
/// * `transform`, `size` - Element's transform and size.
/// * `color_scheme` - Active `ColorScheme`.
/// * `svg_output` - Mutable string buffer.
///
/// # Returns
/// `Result<()>`
#[allow(unused_assignments)]
fn convert_line_to_svg(
    line_data: &Line,
    element_id: &str,
    transform: Option<&AffineTransform>,
    size: Option<&Size>,
    color_scheme: Option<&ColorScheme>,
    svg_output: &mut String,
) -> Result<()> {
    let mut line_style = String::new();
    let mut x1 = 0.0;
    let mut y1 = 0.0;
    let mut x2 = 0.0;
    let mut y2 = 0.0;

    // 1. Calculate Transformed Coordinates
    // The line exists in a local coordinate system defined by 'size', typically from (0,0)
    // to (width, height) where width or height might be zero for horizontal/vertical lines.
    // The 'transform' maps this local system to page coordinates.
    let local_width_pt = dimension_to_pt(size.and_then(|s| s.width.as_ref()));
    let local_height_pt = dimension_to_pt(size.and_then(|s| s.height.as_ref()));

    // Apply the affine transformation matrix [a c e / b d f / 0 0 1]
    // to the start point (local 0, 0) and end point (local W, H).
    if let Some(tf) = transform {
        let a = tf.scale_x.unwrap_or(0.0);
        let b = tf.shear_y.unwrap_or(0.0); // b = shearY
        let c = tf.shear_x.unwrap_or(0.0); // c = shearX
        let d = tf.scale_y.unwrap_or(0.0);
        let translate_unit = tf
            .unit
            .as_ref()
            .cloned()
            .unwrap_or(crate::models::common::Unit::Emu);
        let e = dimension_to_pt(Some(&Dimension {
            magnitude: Some(tf.translate_x.unwrap_or(0.0)),
            unit: Some(translate_unit.clone()),
        }));
        let f = dimension_to_pt(Some(&Dimension {
            magnitude: Some(tf.translate_y.unwrap_or(0.0)),
            unit: Some(translate_unit),
        }));

        // Transformed start point (local 0, 0) -> (e, f)
        x1 = e;
        y1 = f;

        // Transformed end point (local W, H) -> (aW + cH + e, bW + dH + f)
        x2 = a * local_width_pt + c * local_height_pt + e;
        y2 = b * local_width_pt + d * local_height_pt + f;
    } else {
        // Defensive: If no transform, assume line starts at (0,0) and size defines end point.
        x1 = 0.0;
        y1 = 0.0;
        x2 = local_width_pt;
        y2 = local_height_pt;
        eprintln!(
            "Warning: Line element {} lacks a transform. Coordinates might be incorrect.",
            element_id
        );
    }

    // Handle zero-length line segments resulting from transform/size (maybe skip rendering?)
    if (x1 - x2).abs() < 1e-6 && (y1 - y2).abs() < 1e-6 {
        eprintln!("Warning: Skipping zero-length line element {}.", element_id);
        return Ok(());
    }

    // 2. Apply Line Properties to SVG style
    if let Some(props) = &line_data.line_properties {
        // Stroke Color and Opacity using lineFill
        let mut stroke_color = DEFAULT_TEXT_COLOR.to_string();
        let mut stroke_opacity = 1.0;

        if let Some(line_fill) = &props.line_fill {
            // Only handle SolidFill currently
            match &line_fill.fill_kind {
                LineFillContent::SolidFill(solid_fill) => {
                    stroke_color = format_color(solid_fill.color.as_ref(), color_scheme);
                    stroke_opacity = solid_fill.alpha.unwrap_or(1.0);
                }
            }
            // TODO: Handle other LineFill types (None, Gradient) if needed.
        }
        write!(line_style, "stroke:{}; ", stroke_color)?;
        write!(line_style, "stroke-opacity:{}; ", stroke_opacity)?;

        // Stroke Weight
        let stroke_width_pt = dimension_to_pt(props.weight.as_ref());
        let effective_stroke_width = if stroke_width_pt > 0.0 {
            stroke_width_pt
        } else {
            1.0
        }; // Min 1pt width
        write!(line_style, "stroke-width:{}pt; ", effective_stroke_width)?;

        // Dash Style
        if let Some(dash_style) = &props.dash_style {
            let dash_array = match dash_style {
                DashStyle::Solid => "none",
                DashStyle::Dash => "4 4", // Example: 4pt dash, 4pt gap
                DashStyle::Dot => "1 4",  // Example: 1pt dot, 4pt gap
                DashStyle::DashDot => "4 4 1 4", // Example: Dash, gap, dot, gap
                DashStyle::LongDash => "8 4", // Example: 8pt dash, 4pt gap
                DashStyle::LongDashDot => "8 4 1 4",
                _ => "none", // Default to solid for unsupported/unspecified styles
            };
            if dash_array != "none" {
                write!(line_style, "stroke-dasharray:{}; ", dash_array)?;
            }
        }

        // Stroke Cap / Line Join (Defaults are usually fine: butt, miter)
        // write!(line_style, "stroke-linecap:round; ")?; // E.g. "butt", "round", "square"
        // write!(line_style, "stroke-linejoin:round; ")?; // E.g. "miter", "round", "bevel"

        // Arrow Heads (Requires SVG <marker> definitions in <defs>)
        // Example placeholder logic:
        // let needs_defs = false;
        // if props.start_arrow.is_some() && props.start_arrow != Some(ArrowStyle::None) {
        //     write!(line_style, "marker-start:url(#ArrowStart); ")?; needs_defs = true;
        // }
        // if props.end_arrow.is_some() && props.end_arrow != Some(ArrowStyle::None) {
        //     write!(line_style, "marker-end:url(#ArrowEnd); ")?; needs_defs = true;
        // }
        // If needs_defs, ensure <defs> section exists and contains marker definitions.
    } else {
        // Default style if no lineProperties are defined
        write!(
            line_style,
            "stroke:{}; stroke-width:1pt; stroke-opacity:1.0; ",
            DEFAULT_TEXT_COLOR
        )?;
    }

    // 3. Write the SVG <line> element
    // Coordinates are already transformed, so no 'transform' attribute needed on the <line> itself.
    write!(
        svg_output,
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" style="{}" />"#,
        x1,
        y1,
        x2,
        y2,
        line_style.trim_end() // Trim trailing space
    )?;

    Ok(())
}

/// Renders a placeholder for unsupported element types.
fn render_placeholder(
    element_type: &str,
    transform: Option<&AffineTransform>,
    size: Option<&Size>,
    svg_output: &mut String,
) -> Result<()> {
    let mut ph_attrs = String::new();
    let (_tx, _ty) = apply_transform(transform, &mut ph_attrs)?;
    let width = dimension_to_pt(size.and_then(|s| s.width.as_ref())).max(20.0); // Min width
    let height = dimension_to_pt(size.and_then(|s| s.height.as_ref())).max(10.0); // Min height

    // Apply transform to a group containing the placeholder visuals
    write!(svg_output, "<g{}>", ph_attrs)?;
    // Dashed rectangle
    write!(
        svg_output,
        r#"<rect width="{}" height="{}" style="fill:#f0f0f0; stroke:lightgray; stroke-dasharray:3 3; fill-opacity:0.5;" />"#,
        width,
        height // Positioned at (0,0) within the transformed group
    )?;
    // Text label
    write!(
        svg_output,
        r#"<text x="2" y="2" dy="0.8em" style="font-family:sans-serif; font-size:8pt; fill:gray;">{}</text>"#, // Use dy for better positioning
        escape_svg_text(&format!("{} Placeholder", element_type))
    )?;
    write!(svg_output, "</g>")?;
    Ok(())
}

/// Converts a single `PageElement` to an SVG fragment, dispatching to specific conversion functions
/// based on the `element_kind`. Handles groups recursively.
///
/// # Arguments
/// * `element` - The `PageElement` to convert.
/// * `slide_layout_id` - Optional layout ID for context (placeholder resolution).
/// * `layouts_map`, `masters_map`, `elements_map` - Lookup maps.
/// * `color_scheme` - Active `ColorScheme`.
/// * `svg_output` - Mutable string buffer.
///
/// # Returns
/// `Result<()>`
pub(crate) fn convert_page_element_to_svg(
    element: &PageElement,
    slide_layout_id: Option<&str>,
    layouts_map: &LayoutsMap,
    masters_map: &MastersMap,
    elements_map: &ElementsMap,
    color_scheme: Option<&ColorScheme>,
    svg_output: &mut String,
) -> Result<()> {
    // Add a comment for easier debugging in the SVG output
    // writeln!(svg_output, "<!-- Element ID: {} -->", element.object_id)?; // Uncomment if useful

    // Use a <g> wrapper *only if* the element itself doesn't handle its transform
    // (e.g., Shape/Table/Image/Line handle transform internally or apply it directly).
    // Groups apply transform to their own <g>. Placeholders might need one.
    // Let's decide based on kind.

    match &element.element_kind {
        PageElementKind::Shape(shape) => {
            convert_shape_to_svg(
                shape,
                element.transform.as_ref(),
                element.size.as_ref(),
                slide_layout_id,
                layouts_map,
                masters_map,
                elements_map,
                color_scheme,
                svg_output,
            )?;
        }
        PageElementKind::Table(table) => {
            convert_table_to_svg(
                table,
                element.transform.as_ref(),
                element.size.as_ref(),
                color_scheme,
                svg_output,
            )?;
        }
        PageElementKind::Image(image_data) => {
            convert_image_to_svg(
                image_data,
                &element.object_id,
                element.transform.as_ref(),
                element.size.as_ref(),
                svg_output,
            )?;
        }
        PageElementKind::Line(line_data) => {
            convert_line_to_svg(
                line_data,
                &element.object_id,
                element.transform.as_ref(),
                element.size.as_ref(),
                color_scheme,
                svg_output,
            )?;
        }
        PageElementKind::ElementGroup(group) => {
            let mut group_attrs = String::new();
            // Apply the group's transform to its own <g> tag
            apply_transform(element.transform.as_ref(), &mut group_attrs)?;
            writeln!(
                svg_output,
                "<g data-object-id=\"{}_group\" {}>",
                element.object_id, group_attrs
            )?; // Add data-id for clarity

            for child_element in &group.children {
                // Recursively convert child elements, passing down the context
                convert_page_element_to_svg(
                    child_element,
                    slide_layout_id,
                    layouts_map,
                    masters_map,
                    elements_map,
                    color_scheme,
                    svg_output,
                )?;
                writeln!(svg_output)?; // Newline between children
            }
            write!(svg_output, "</g>")?; // Close the group's <g> tag
        }
        // --- Unsupported Element Types -> Render Placeholders ---
        PageElementKind::Video(_) => render_placeholder(
            "Video",
            element.transform.as_ref(),
            element.size.as_ref(),
            svg_output,
        )?,
        PageElementKind::WordArt(_) => render_placeholder(
            "WordArt",
            element.transform.as_ref(),
            element.size.as_ref(),
            svg_output,
        )?,
        PageElementKind::SheetsChart(_) => render_placeholder(
            "SheetsChart",
            element.transform.as_ref(),
            element.size.as_ref(),
            svg_output,
        )?,
        PageElementKind::SpeakerSpotlight(_) => render_placeholder(
            "SpeakerSpotlight",
            element.transform.as_ref(),
            element.size.as_ref(),
            svg_output,
        )?,
    }

    Ok(())
}
