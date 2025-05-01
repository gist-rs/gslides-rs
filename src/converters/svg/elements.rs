//! Handles the conversion of specific `PageElement` types (Shape, Table, Group, Line, Image)
//! into their corresponding SVG representations.

use super::{
    constants::*,
    error::Result, // Keep SvgConversionError if needed for specific errors here
    structure::{
        find_placeholder_element, get_placeholder_default_text_style, ElementsMap, LayoutsMap,
        MastersMap,
    },
    // Remove unused import 'convert_text_content_to_svg'
    text::convert_text_content_to_html,
    // [+] Add import for Dimension and Unit
    utils::{apply_transform, dimension_to_pt, escape_svg_text, format_color, AsShape},
};
use crate::models::{
    colors::ColorScheme,
    // [+] Add import for Dimension and Unit
    common::{AffineTransform, Dimension, Size, Unit},
    elements::{PageElement, PageElementKind},
    image::Image,
    line::{Line, LineFillContent},
    properties::{ParagraphStyle, TextStyle},
    shape::Shape,
    shape_properties::*,
    table::Table,
    text_element::TextElementKind, // Required for checking ParagraphMarker in shape style override
};
use std::fmt::Write;

/// Helper function to build the SVG `style` attribute string for shape geometry (fill, stroke).
///
/// # Arguments
/// * `props` - The `ShapeProperties` of the shape.
/// * `color_scheme` - The active `ColorScheme` for resolving theme colors.
///
/// # Returns
/// `Result<String>` containing the CSS style string.
fn build_shape_style(
    props: &ShapeProperties,
    color_scheme: Option<&ColorScheme>,
) -> Result<String> {
    let mut shape_style = String::new();

    // --- Background Fill ---
    // Check property state first. If not rendered, treat as transparent.
    let render_fill =
        props.shape_background_fill.property_state.as_ref() != Some(&PropertyState::NotRendered);

    let (fill_color, fill_opacity_str) = if render_fill {
        // Access the fill_kind Option within shape_background_fill
        match props.shape_background_fill.fill_kind.as_ref() {
            Some(fill_content) => {
                // Match on the enum variant inside fill_kind
                match fill_content {
                    ShapeBackgroundFillContent::SolidFill(solid) => {
                        let color = format_color(solid.color.as_ref(), color_scheme);
                        let opacity = solid.alpha.unwrap_or(1.0);
                        (color, format!("{:.2}", opacity)) // Format opacity to 2 decimal places
                    }
                    ShapeBackgroundFillContent::StretchedPictureFill(_) => {
                        // TODO: Handle picture fill (e.g., create a pattern in <defs> or skip)
                        eprintln!("Warning: StretchedPictureFill background not yet supported.");
                        ("grey".to_string(), "0.5".to_string()) // Placeholder visually
                    } // Add other fill types here if the enum grows
                }
            }
            None => ("none".to_string(), "0".to_string()), // fill_kind is None means transparent
        }
    } else {
        // property_state is NOT_RENDERED
        ("none".to_string(), "0".to_string())
    };

    // Only write fill attributes if fill is not "none"
    if fill_color != "none" {
        write!(
            shape_style,
            "fill:{}; fill-opacity:{}; ",
            fill_color, fill_opacity_str
        )?;
    } else {
        write!(shape_style, "fill:none; ")?;
    }

    // --- Outline ---
    // Access outline directly since it's not Option in ShapeProperties
    let outline = &props.outline;

    // Check if outline should be rendered based on propertyState
    let render_outline = outline.property_state.as_ref() != Some(&PropertyState::NotRendered);

    if render_outline {
        // Get outline weight (stroke width)
        let stroke_width_pt = dimension_to_pt(outline.weight.as_ref());

        // Only apply stroke styling if width is visually significant (> 0)
        if stroke_width_pt > 0.0 {
            // Outline Fill (Stroke Color/Opacity)
            // Access outline_fill Option within Outline struct
            let (stroke_color, stroke_opacity_str) = match outline.outline_fill.as_ref() {
                Some(outline_fill_container) => {
                    // Access fill_kind enum within OutlineFill struct
                    match &outline_fill_container.fill_kind {
                        OutlineFillContent::SolidFill(solid) => {
                            let color = format_color(solid.color.as_ref(), color_scheme);
                            let opacity = solid.alpha.unwrap_or(1.0);
                            (color, format!("{:.2}", opacity))
                        } // Add other outline fill types here if the enum grows
                    }
                }
                None => ("none".to_string(), "0".to_string()), // No outline fill defined
            };

            // Write stroke properties only if color is not "none"
            if stroke_color != "none" {
                write!(
                    shape_style,
                    "stroke:{}; stroke-opacity:{}; ",
                    stroke_color, stroke_opacity_str
                )?;
                write!(shape_style, "stroke-width:{}pt; ", stroke_width_pt)?;

                // Outline Dash Style
                // Access dash_style Option within Outline struct
                if let Some(dash_style) = &outline.dash_style {
                    let dash_array = match dash_style {
                        // Use the correct enum variants from DashStyle
                        DashStyle::Solid => "none",
                        DashStyle::Dash => "4 4",
                        DashStyle::Dot => "1 4",
                        DashStyle::DashDot => "4 4 1 4",
                        DashStyle::LongDash => "8 4",
                        DashStyle::LongDashDot => "8 4 1 4",
                        // Handle potential unknown enum variants defensively
                        DashStyle::DashStyleUnspecified => "none", // Treat unspecified as solid
                    };
                    if dash_array != "none" {
                        write!(shape_style, "stroke-dasharray:{}; ", dash_array)?;
                    }
                    // If dash_array is "none", we don't need to write stroke-dasharray as solid is the default
                }
                // If outline.dash_style is None, default is SOLID (DashStyleUnspecified maps to solid), so no dasharray needed.
            } else {
                // Stroke color resolved to "none", so treat as no stroke
                write!(shape_style, "stroke:none; ")?;
            }
        } else {
            // If stroke width is 0 or less, explicitly set stroke to none
            write!(shape_style, "stroke:none; ")?;
        }
    } else {
        // PropertyState is NOT_RENDERED, treat as no stroke
        write!(shape_style, "stroke:none; ")?;
    }

    // TODO: Handle shadow if needed (complex, requires SVG filters defined in <defs>)
    // if let Some(shadow) = &props.shadow { ... }

    Ok(shape_style.trim_end().to_string()) // Trim trailing space
}

/// Converts a Shape element (geometry and text content) to an SVG fragment.
/// Handles transform differently based on shape type:
/// - **TextBox:** Applies only translation to the outer group. Geometry is scaled/sheared
///   internally. Text (via `<foreignObject>`) is placed in the translated group and is *not* scaled/sheared.
/// - **Other Shapes:** Applies the full transform (scale, shear, translate) to the outer group.
///   Geometry and `<foreignObject>` (if text exists) inherit the full transform.
/// Resolves placeholder styles and applies them to the HTML text.
///
/// # Arguments
/// * `element_id` - The object ID of the PageElement containing this shape.
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
    element_id: &str, // Added element_id
    shape: &Shape,
    transform: Option<&AffineTransform>,
    size: Option<&Size>,
    slide_layout_id: Option<&str>,
    layouts_map: &LayoutsMap,
    masters_map: &MastersMap,
    elements_map: &ElementsMap,
    color_scheme: Option<&ColorScheme>,
    svg_output: &mut String,
) -> Result<()> {
    // Calculate base dimensions in points
    let width_pt = dimension_to_pt(size.and_then(|s| s.width.as_ref()));
    let height_pt = dimension_to_pt(size.and_then(|s| s.height.as_ref()));

    // Determine shape type
    let shape_type = shape
        .shape_type
        .as_ref()
        .unwrap_or(&crate::models::shape::ShapeType::TypeUnspecified);

    // --- Handle Transform based on Shape Type ---
    let mut outer_group_attrs = String::new();
    let mut geometry_transform_attrs = String::new();
    let tx_pt;
    let ty_pt;
    let scale_x;
    let scale_y;
    let shear_x;
    let shear_y;

    if let Some(tf) = transform {
        scale_x = tf.scale_x.unwrap_or(1.0);
        scale_y = tf.scale_y.unwrap_or(1.0);
        shear_x = tf.shear_x.unwrap_or(0.0);
        shear_y = tf.shear_y.unwrap_or(0.0);

        let translate_unit = tf.unit.as_ref().cloned().unwrap_or(Unit::Emu);
        tx_pt = dimension_to_pt(Some(&Dimension {
            magnitude: Some(tf.translate_x.unwrap_or(0.0)),
            unit: Some(translate_unit.clone()),
        }));
        ty_pt = dimension_to_pt(Some(&Dimension {
            magnitude: Some(tf.translate_y.unwrap_or(0.0)),
            unit: Some(translate_unit),
        }));

        if *shape_type == crate::models::shape::ShapeType::TextBox {
            // TextBox: Outer group only gets translation. Geometry gets scale/shear.
            write!(
                outer_group_attrs,
                r#" transform="translate({} {})""#,
                tx_pt, ty_pt
            )?;
            if scale_x != 1.0 || scale_y != 1.0 || shear_x != 0.0 || shear_y != 0.0 {
                write!(
                    geometry_transform_attrs,
                    r#" transform="matrix({} {} {} {} 0 0)""#,
                    scale_x, shear_y, shear_x, scale_y
                )?;
            }
        } else {
            // Other shapes: Outer group gets the full transform.
            write!(
                outer_group_attrs,
                r#" transform="matrix({} {} {} {} {} {})""#,
                scale_x, shear_y, shear_x, scale_y, tx_pt, ty_pt
            )?;
            // Geometry transform is identity (handled by outer group)
        }
    }

    // --- Start Outer Group ---
    writeln!(
        svg_output,
        "<g data-object-id=\"{}\"{}>", // Add objectId and appropriate transform
        element_id, outer_group_attrs
    )?;

    // --- Render Shape Geometry ---
    // Geometry is rendered at (0,0) relative to its transform context.
    let default_props = ShapeProperties::default();
    let shape_props_ref = shape.shape_properties.as_ref().unwrap_or(&default_props);

    if width_pt > 0.0 && height_pt > 0.0 {
        if shape.shape_properties.is_some() {
            let shape_style = build_shape_style(shape_props_ref, color_scheme)?;

            // Apply geometry-specific transform if needed (only for TextBox currently)
            if !geometry_transform_attrs.is_empty() {
                writeln!(svg_output, "  <g{}>", geometry_transform_attrs)?; // Geometry group start
            }

            match shape_type {
                crate::models::shape::ShapeType::Rectangle
                | crate::models::shape::ShapeType::TextBox => {
                    writeln!(
                        svg_output,
                        r#"    <rect x="0" y="0" width="{}" height="{}" style="{}" />"#, // Indent if inside geometry group
                        width_pt, height_pt, shape_style
                    )?;
                }
                crate::models::shape::ShapeType::RoundRectangle => {
                    let default_rx = (width_pt * 0.08).min(height_pt * 0.08).max(2.0);
                    writeln!(
                        svg_output,
                        r#"    <rect x="0" y="0" width="{}" height="{}" rx="{}" ry="{}" style="{}" />"#, // Indent
                        width_pt, height_pt, default_rx, default_rx, shape_style
                    )?;
                }
                crate::models::shape::ShapeType::Ellipse => {
                    writeln!(
                        svg_output,
                        r#"    <ellipse cx="{}" cy="{}" rx="{}" ry="{}" style="{}" />"#, // Indent
                        width_pt / 2.0,
                        height_pt / 2.0,
                        width_pt / 2.0,
                        height_pt / 2.0,
                        shape_style
                    )?;
                }
                _ => {
                    eprintln!("Warning: Unsupported or unspecified shape type '{:?}' for element ID: {}. Rendering placeholder.", shape_type, element_id);
                    writeln!(
                        svg_output,
                        r#"    <rect x="0" y="0" width="{}" height="{}" style="fill:#e0e0e0; stroke:gray; stroke-dasharray: 3 3; fill-opacity:0.7;" />"#, // Indent
                        width_pt, height_pt
                    )?;
                    writeln!(
                        svg_output,
                        r#"    <text x="2" y="10" style="font-family:sans-serif; font-size:8pt; fill:#555;">Unsupported Shape: {}</text>"#, // Indent
                        escape_svg_text(&format!("{:?}", shape_type))
                    )?;
                }
            }
            // Close geometry-specific transform group if opened
            if !geometry_transform_attrs.is_empty() {
                writeln!(svg_output, "  </g>")?; // Geometry group end
            }
        } else {
            eprintln!(
                "Debug: Shape (id: {}) lacks shapeProperties, skipping geometry rendering.",
                element_id
            );
        }
    } else if width_pt > 0.0 || height_pt > 0.0 {
        eprintln!(
            "Warning: Shape (id: {}) has zero width or height ({}x{}pt). Geometry skipped.",
            element_id, width_pt, height_pt
        );
    }

    // --- Resolve Inherited Text Styles ---
    // (This logic remains the same)
    let mut effective_text_style_base = TextStyle::default();
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
                // Get base text style (font, size, color etc.)
                if let Some(placeholder_base_style) =
                    get_placeholder_default_text_style(placeholder_element)
                {
                    effective_text_style_base = placeholder_base_style;
                }
                // Get base paragraph style (alignment etc.) from the first ParagraphMarker in placeholder
                if let Some(placeholder_shape) = placeholder_element.element_kind.as_shape() {
                    if let Some(text) = &placeholder_shape.text {
                        if let Some(elements) = &text.text_elements {
                            for element in elements {
                                if let Some(TextElementKind::ParagraphMarker(pm)) = &element.kind {
                                    if let Some(style) = &pm.style {
                                        effective_paragraph_style = Some(style.clone());
                                        break; // Found the first paragraph's style
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                eprintln!(
                    "Warning: Placeholder parent ID '{}' not found for shape ID: {}",
                    placeholder.parent_object_id.as_deref().unwrap_or("N/A"),
                    element_id
                );
            }
        } else {
            eprintln!("Warning: Shape ID '{}' has placeholder but slide_layout_id is missing for style lookup.", element_id);
        }
    }

    // --- Render Text Content using <foreignObject> ---
    // Positioned relative to the outer group's origin (0,0) plus padding.
    // For TextBoxes, the outer group only has translation, so text isn't scaled/sheared.
    // For other shapes, the outer group has the full transform, so text IS scaled/sheared.
    if let Some(text) = &shape.text {
        let text_padding_x = 3.0; // Left padding
        let text_padding_y = 2.0; // Top padding
        let right_padding = 3.0;
        let bottom_padding = 2.0;

        // Calculate the available area for the foreignObject based on the *unscaled* shape size.
        // For non-TextBox shapes, this area will be scaled by the outer group transform.
        // For TextBoxes, it will not be scaled.
        let text_box_width = (width_pt - text_padding_x - right_padding).max(0.0);
        let text_box_height = (height_pt - text_padding_y - bottom_padding).max(0.0);

        if text_box_width > 0.0 && text_box_height > 0.0 {
            // Resolve final paragraph style (shape overrides placeholder)
            let mut final_para_style = effective_paragraph_style.clone();
            if let Some(elements) = &text.text_elements {
                for element in elements {
                    if let Some(TextElementKind::ParagraphMarker(pm)) = &element.kind {
                        if let Some(style) = &pm.style {
                            final_para_style = Some(style.clone());
                            break;
                        }
                    }
                }
            }

            // Create <foreignObject> inside the *outer* group
            writeln!(
                svg_output,
                r#"  <foreignObject x="{}" y="{}" width="{}" height="{}">"#,
                text_padding_x, text_padding_y, text_box_width, text_box_height
            )?;
            writeln!(
                svg_output,
                r#"    <div xmlns="http://www.w3.org/1999/xhtml" style="width:100%; height:100%; overflow:hidden; box-sizing: border-box;">"#
            )?;

            // Convert TextContent to HTML
            convert_text_content_to_html(
                text,
                final_para_style.as_ref(),
                &effective_text_style_base,
                color_scheme,
                svg_output,
            )?;

            writeln!(svg_output)?;
            writeln!(svg_output, "    </div>")?;
            writeln!(svg_output, "  </foreignObject>")?;
        } else if !text.text_elements.as_ref().map_or(true, |v| v.is_empty()) {
            eprintln!("Debug: Skipping text rendering for shape ID {} due to zero-area text box ({}x{}) after padding.", element_id, text_box_width, text_box_height);
        }
    }

    // Close the main outer group for the shape
    writeln!(svg_output, "</g>")?;

    Ok(())
}

/// Converts a Table element to SVG using `<foreignObject>` to embed styled HTML content.
/// Handles transform, size, basic cell styling (border, background), and cell text content.
///
/// # Arguments
/// * `element_id` - The object ID of the PageElement containing this table.
/// * `table` - The `Table` data.
/// * `transform`, `size` - Element's transform and size.
/// * `color_scheme` - Active `ColorScheme`.
/// * `svg_output` - Mutable string buffer.
///
/// # Returns
/// `Result<()>`
#[allow(clippy::too_many_arguments)]
fn convert_table_to_svg(
    element_id: &str, // Added element_id
    table: &Table,
    transform: Option<&AffineTransform>,
    size: Option<&Size>,
    color_scheme: Option<&ColorScheme>,
    svg_output: &mut String,
) -> Result<()> {
    // The table's transform only affects the position/size of the <foreignObject> container.
    // The HTML table inside will fill this container.
    let mut foreign_object_attrs = String::new();
    let (tx, ty) = apply_transform(transform, &mut foreign_object_attrs)?;
    let width = dimension_to_pt(size.and_then(|s| s.width.as_ref()));
    let height = dimension_to_pt(size.and_then(|s| s.height.as_ref()));

    // Avoid creating empty or invalid foreignObjects
    if width <= 0.0 || height <= 0.0 {
        eprintln!(
            "Warning: Skipping table element {} with zero or negative dimensions ({}x{}pt).",
            element_id, width, height
        );
        return Ok(());
    }

    // --- <foreignObject> Setup ---
    write!(
        svg_output,
        // Apply transform attributes to foreignObject
        r#"<foreignObject x="{}" y="{}" width="{}" height="{}" data-object-id="{}"{}>"#,
        tx, ty, width, height, element_id, foreign_object_attrs
    )?;
    writeln!(svg_output)?; // Newline after opening tag

    // --- HTML Content within <foreignObject> ---
    // XHTML namespace is crucial for proper rendering within SVG
    write!(
        svg_output,
        r#"  <div xmlns="http://www.w3.org/1999/xhtml" style="width:100%; height:100%; overflow:hidden; box-sizing: border-box;">"# // Added box-sizing
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
                        // --- Resolve Text Styles for Cell (Simplified: Use default base) ---
                        // Tables don't typically use placeholder inheritance in the same way shapes do.
                        // We'll use a default TextStyle as the base and the first paragraph's style for alignment.
                        let cell_text_style_base = TextStyle::default();
                        let mut cell_para_style: Option<ParagraphStyle> = None;
                        if let Some(elements) = &text.text_elements {
                            for element in elements {
                                if let Some(TextElementKind::ParagraphMarker(pm)) = &element.kind {
                                    if let Some(style) = &pm.style {
                                        cell_para_style = Some(style.clone());
                                        break;
                                    }
                                }
                            }
                        }

                        convert_text_content_to_html(
                            text,
                            cell_para_style.as_ref(), // Pass initial paragraph style (if any)
                            &cell_text_style_base,    // Pass default text style as base
                            color_scheme,
                            svg_output,
                        )?;
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
            r#"<image x="{}" y="{}" width="{}" height="{}" xlink:href="{}"{} preserveAspectRatio="xMidYMid meet" data-object-id="{}"/>"#, // Use xlink:href for broader compatibility, add ID
            tx, ty, width, height, safe_url, img_attrs, element_id
        )?;
    } else {
        // Fallback if no URL is provided - render a placeholder rectangle with text.
        eprintln!(
            "Warning: Image element {} is missing contentUrl.",
            element_id
        );
        // Apply transform to the placeholder group
        write!(
            svg_output,
            "<g data-object-id=\"{}\"{}>",
            element_id, img_attrs
        )?; // Add ID to group
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
        let a = tf.scale_x.unwrap_or(0.0); // Default to 0.0 for scale if missing
        let b = tf.shear_y.unwrap_or(0.0); // b = shearY
        let c = tf.shear_x.unwrap_or(0.0); // c = shearX
        let d = tf.scale_y.unwrap_or(0.0); // Default to 0.0 for scale if missing
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
        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" style="{}" data-object-id="{}"/>"#, // Add ID
        x1,
        y1,
        x2,
        y2,
        line_style.trim_end(), // Trim trailing space
        element_id
    )?;

    Ok(())
}

/// Renders a placeholder for unsupported element types.
fn render_placeholder(
    element_type: &str,
    element_id: &str, // Added element_id
    transform: Option<&AffineTransform>,
    size: Option<&Size>,
    svg_output: &mut String,
) -> Result<()> {
    let mut ph_attrs = String::new();
    let (_tx, _ty) = apply_transform(transform, &mut ph_attrs)?;
    let width = dimension_to_pt(size.and_then(|s| s.width.as_ref())).max(20.0); // Min width
    let height = dimension_to_pt(size.and_then(|s| s.height.as_ref())).max(10.0); // Min height

    // Apply transform to a group containing the placeholder visuals
    write!(
        svg_output,
        "<g data-object-id=\"{}\"{}>",
        element_id, ph_attrs
    )?; // Add ID
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

    // Most elements handle their own transform internally (Shape, Table, Image, Line)
    // or calculate transformed coordinates (Line). Groups apply transform to their own <g>.

    match &element.element_kind {
        PageElementKind::Shape(shape) => {
            // Shape conversion now handles its own transform strategy based on shape type
            convert_shape_to_svg(
                &element.object_id,
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
            // Table conversion uses foreignObject which handles its own transform attribute
            convert_table_to_svg(
                &element.object_id,
                table,
                element.transform.as_ref(),
                element.size.as_ref(),
                color_scheme,
                svg_output,
            )?;
        }
        PageElementKind::Image(image_data) => {
            // Image conversion applies transform directly to the <image> tag or its wrapper group
            convert_image_to_svg(
                image_data,
                &element.object_id,
                element.transform.as_ref(),
                element.size.as_ref(),
                svg_output,
            )?;
        }
        PageElementKind::Line(line_data) => {
            // Line conversion calculates transformed coordinates, doesn't need a separate group transform
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
                "<g data-object-id=\"{}_group\" {}>", // Add data-id suffix for clarity
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
            &element.object_id,
            element.transform.as_ref(),
            element.size.as_ref(),
            svg_output,
        )?,
        PageElementKind::WordArt(_) => render_placeholder(
            "WordArt",
            &element.object_id,
            element.transform.as_ref(),
            element.size.as_ref(),
            svg_output,
        )?,
        PageElementKind::SheetsChart(_) => render_placeholder(
            "SheetsChart",
            &element.object_id,
            element.transform.as_ref(),
            element.size.as_ref(),
            svg_output,
        )?,
        PageElementKind::SpeakerSpotlight(_) => render_placeholder(
            "SpeakerSpotlight",
            &element.object_id,
            element.transform.as_ref(),
            element.size.as_ref(),
            svg_output,
        )?,
    }

    Ok(())
}
