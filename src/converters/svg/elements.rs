//! Handles the conversion of specific `PageElement` types (Shape, Table, Group, Line, Image)
//! into their corresponding SVG representations.

use log::{debug, warn};

use super::{
    constants::*,
    error::Result, // Keep SvgConversionError if needed for specific errors here
    structure::{
        find_placeholder_element, get_placeholder_default_text_style, ElementsMap, LayoutsMap,
        MastersMap,
    },
    text::{convert_text_content_to_html, merge_paragraph_styles}, // Keep this import
    utils::{apply_transform, dimension_to_pt, escape_svg_text, format_color, AsShape},
};
use crate::models::{
    colors::ColorScheme,
    common::{AffineTransform, Dimension, Size, Unit}, // Keep Dimension and Unit
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
///   Resolves placeholder styles and applies them to the HTML text.
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
#[allow(unused_assignments)]
fn convert_shape_to_svg(
    element_id: &str,
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

    // --- Handle Transform ---
    let mut outer_group_transform_attr = String::new();
    let mut geometry_transform_attr = String::new();
    let mut tx_pt = 0.0;
    let mut ty_pt = 0.0;
    let mut scale_x = 1.0;
    let mut scale_y = 1.0;
    let mut shear_x = 0.0;
    let mut shear_y = 0.0;

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

        // Outer group only gets translation
        if tx_pt != 0.0 || ty_pt != 0.0 {
            write!(
                outer_group_transform_attr,
                r#" transform="translate({} {})""#,
                tx_pt, ty_pt
            )?;
        }

        // Geometry gets scale/shear matrix (relative to translated origin)
        if scale_x != 1.0 || scale_y != 1.0 || shear_x != 0.0 || shear_y != 0.0 {
            write!(
                geometry_transform_attr,
                r#" transform="matrix({} {} {} {} 0 0)""#,
                scale_x, shear_y, shear_x, scale_y
            )?;
        }
    }

    // --- Start Outer Group ---
    // This group only has translation applied (if any).
    writeln!(
        svg_output,
        "<g data-object-id=\"{}\"{}>",
        element_id,
        outer_group_transform_attr // Only translate transform here
    )?;

    // --- Render Shape Geometry ---
    // Geometry is rendered at (0,0) relative to the translated outer group.
    // Scale/shear is applied directly to the geometry element itself.
    let default_props = ShapeProperties::default();
    let shape_props_ref = shape.shape_properties.as_ref().unwrap_or(&default_props);
    let shape_type = shape
        .shape_type
        .as_ref()
        .unwrap_or(&crate::models::shape::ShapeType::TypeUnspecified);

    if width_pt > 0.0 && height_pt > 0.0 {
        if shape.shape_properties.is_some() {
            let shape_style = build_shape_style(shape_props_ref, color_scheme)?;

            match shape_type {
                crate::models::shape::ShapeType::Rectangle
                | crate::models::shape::ShapeType::TextBox => {
                    writeln!(
                        svg_output,
                        // Apply geometry transform here
                        r#"  <rect x="0" y="0" width="{}" height="{}" style="{}"{} />"#,
                        width_pt, height_pt, shape_style, geometry_transform_attr
                    )?;
                }
                crate::models::shape::ShapeType::RoundRectangle => {
                    let default_rx = (width_pt * 0.08).min(height_pt * 0.08).max(2.0);
                    // Apply geometry transform here
                    writeln!(
                        svg_output,
                        r#"  <rect x="0" y="0" width="{}" height="{}" rx="{}" ry="{}" style="{}"{} />"#,
                        width_pt,
                        height_pt,
                        default_rx,
                        default_rx,
                        shape_style,
                        geometry_transform_attr
                    )?;
                }
                crate::models::shape::ShapeType::Ellipse => {
                    // Apply geometry transform here
                    writeln!(
                        svg_output,
                        // Ellipse uses cx/cy, so transform needs careful application.
                        // Easiest is to wrap ellipse in a <g> with the transform.
                        "<g{}> <ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" style=\"{}\" /> </g>",
                        geometry_transform_attr, // Apply transform to wrapper group
                        width_pt / 2.0, // cx/cy relative to wrapper group's 0,0
                        height_pt / 2.0,
                        width_pt / 2.0, // rx/ry are base dimensions
                        height_pt / 2.0,
                        shape_style
                    )?;
                }
                _ => {
                    // Placeholder geometry - apply transform to it as well
                    warn!("Unsupported or unspecified shape type '{:?}' for element ID: {}. Rendering placeholder.", shape_type, element_id);
                    writeln!(
                        svg_output,
                        r#"  <rect x="0" y="0" width="{}" height="{}" style="fill:#e0e0e0; stroke:gray; stroke-dasharray: 3 3; fill-opacity:0.7;"{} />"#,
                        width_pt, height_pt, geometry_transform_attr
                    )?;
                    // Text inside placeholder doesn't need the geometry transform
                    writeln!(
                        svg_output,
                        r#"  <text x="2" y="10" style="font-family:sans-serif; font-size:8pt; fill:#555;">Unsupported Shape: {}</text>"#,
                        escape_svg_text(&format!("{:?}", shape_type))
                    )?;
                }
            }
        } else {
            debug!(
                "Shape (id: {}) lacks shapeProperties, skipping geometry rendering.",
                element_id
            );
        }
    } else if width_pt > 0.0 || height_pt > 0.0 {
        warn!(
            "Shape (id: {}) has zero width or height ({}x{}pt). Geometry skipped.",
            element_id, width_pt, height_pt
        );
    }

    // --- Resolve Inherited Text Styles ---
    let mut effective_text_style_base = TextStyle::default();
    // Style from placeholder
    let mut placeholder_paragraph_style: Option<ParagraphStyle> = None;

    if let Some(placeholder) = &shape.placeholder {
        if let Some(layout_id) = slide_layout_id {
            if let Some(placeholder_element) = find_placeholder_element(
                placeholder,
                layout_id,
                layouts_map,
                masters_map,
                elements_map,
            ) {
                if let Some(placeholder_base_style) =
                    get_placeholder_default_text_style(placeholder_element)
                {
                    effective_text_style_base = placeholder_base_style;
                }

                // Extract paragraph style from the placeholder element
                if let Some(placeholder_shape) = placeholder_element.element_kind.as_shape() {
                    if let Some(text) = &placeholder_shape.text {
                        if let Some(elements) = &text.text_elements {
                            for text_element in elements {
                                if let Some(TextElementKind::ParagraphMarker(pm)) =
                                    &text_element.kind
                                {
                                    if let Some(style) = &pm.style {
                                        placeholder_paragraph_style = Some(style.clone());
                                        // Found the first paragraph style in placeholder
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                warn!(
                    "Placeholder parent ID '{}' not found for shape ID: {}",
                    placeholder.parent_object_id.as_deref().unwrap_or("N/A"),
                    element_id
                );
            }
        } else {
            warn!(
                "Shape ID '{}' has placeholder but slide_layout_id is missing for style lookup.",
                element_id
            );
        }
    }

    // --- Render Text Content using <foreignObject> ---
    // Positioned at (0,0) relative to the outer (translate-only) group.
    // Uses base dimensions and is NOT scaled/sheared by SVG transforms.
    if let Some(text) = &shape.text {
        let text_padding_top = 2.0;
        let text_padding_right = 3.0;
        let text_padding_bottom = 2.0;
        let text_padding_left = 3.0;

        if width_pt > 0.0 && height_pt > 0.0 {
            // Find the shape's own primary paragraph style (if any)
            let mut shape_paragraph_style: Option<ParagraphStyle> = None;
            if let Some(elements) = &text.text_elements {
                for element in elements {
                    if let Some(TextElementKind::ParagraphMarker(pm)) = &element.kind {
                        if let Some(style) = &pm.style {
                            shape_paragraph_style = Some(style.clone());
                            // Use the first paragraph style found in the shape itself
                            break;
                        }
                    }
                }
            }

            // Merge the shape's style onto the placeholder's style
            // This merged style becomes the initial style for the text content rendering
            let final_initial_para_style = merge_paragraph_styles(
                shape_paragraph_style.as_ref(),
                placeholder_paragraph_style.as_ref(), // Pass Option<&ParagraphStyle>
            );

            // *** Extract font_scale from shape properties ***
            let font_scale = shape
                .shape_properties
                .as_ref()
                .map(|props| &props.autofit)
                .and_then(|autofit_ref| autofit_ref.font_scale);

            // Debug log the extracted font_scale
            if font_scale.is_some() {
                debug!(
                    "Shape ID {}: Applying font_scale: {:?}",
                    element_id, font_scale
                );
            }

            // Create <foreignObject> inside the outer group. Use base width/height.
            writeln!(
                svg_output,
                r#"  <foreignObject x="0" y="0" width="{}" height="{}" overflow="visible">"#,
                scale_x * width_pt, // Use BASE dimensions here
                scale_y * height_pt
            )?;

            let div_padding_style = format!(
                "padding: {}pt {}pt {}pt {}pt;",
                text_padding_top, text_padding_right, text_padding_bottom, text_padding_left
            );

            writeln!(
                svg_output,
                r#"    <div xmlns="http://www.w3.org/1999/xhtml" style="width:100%; height:100%; box-sizing: border-box; {}">"#,
                div_padding_style
            )?;

            // *** Pass the merged initial paragraph style to the HTML converter ***
            convert_text_content_to_html(
                text,
                Some(&final_initial_para_style), // Pass the merged initial style
                &effective_text_style_base,
                color_scheme,
                font_scale, // Pass the extracted font_scale here
                svg_output,
            )?;

            writeln!(svg_output)?;
            writeln!(svg_output, "    </div>")?;
            writeln!(svg_output, "  </foreignObject>")?;
        } else if !text.text_elements.as_ref().map_or(true, |v| v.is_empty()) {
            debug!(
                "Skipping text rendering for shape ID {} due to zero-area shape ({}x{}pt).",
                element_id, width_pt, height_pt
            );
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
    element_id: &str,
    table: &Table,
    transform: Option<&AffineTransform>,
    size: Option<&Size>,
    color_scheme: Option<&ColorScheme>,
    svg_output: &mut String,
) -> Result<()> {
    let mut foreign_object_attrs = String::new();
    // apply_transform gets the full matrix attribute string
    let (_, _) = apply_transform(transform, &mut foreign_object_attrs)?; // We only need the attribute string
    let width_pt = dimension_to_pt(size.and_then(|s| s.width.as_ref()));
    let height_pt = dimension_to_pt(size.and_then(|s| s.height.as_ref()));

    if width_pt <= 0.0 || height_pt <= 0.0 {
        warn!(
            "Skipping table element {} with zero or negative dimensions ({}x{}pt).",
            element_id, width_pt, height_pt
        );
        return Ok(());
    }

    // --- <foreignObject> Setup ---
    // Position at 0,0; the transform matrix handles the actual placement.
    write!(
        svg_output,
        r#"<foreignObject x="0" y="0" width="{}" height="{}" overflow="visible" data-object-id="{}"{}>"#,
        width_pt,
        height_pt,
        element_id,
        foreign_object_attrs // Apply full transform here
    )?;
    writeln!(svg_output)?;

    // --- HTML Content within <foreignObject> ---
    write!(
        svg_output,
        r#"  <div xmlns="http://www.w3.org/1999/xhtml" style="width:100%; height:100%; box-sizing: border-box;">"#
    )?;
    writeln!(svg_output)?;

    write!(
        svg_output,
        r#"    <table style="border-collapse: collapse; width:100%; height:100%; border: 1px solid #ccc; table-layout: fixed;">"#
    )?;

    // --- Table Rows and Cells ---
    if let Some(rows) = &table.table_rows {
        for row in rows {
            writeln!(svg_output)?;
            write!(svg_output, "      <tr>")?;

            if let Some(cells) = &row.table_cells {
                if !cells.is_empty() {
                    writeln!(svg_output)?;
                }

                for cell in cells {
                    let colspan = cell.column_span.unwrap_or(1);
                    let rowspan = cell.row_span.unwrap_or(1);
                    let mut td_attrs = String::new();
                    if colspan > 1 {
                        write!(td_attrs, r#" colspan="{}""#, colspan)?;
                    }
                    if rowspan > 1 {
                        write!(td_attrs, r#" rowspan="{}""#, rowspan)?;
                    }

                    let mut cell_style = "border: 1px solid #eee; padding: 3pt; vertical-align: top; overflow: hidden;".to_string();
                    if let Some(props) = &cell.table_cell_properties {
                        if let Some(bg_fill) = &props.table_cell_background_fill {
                            if let Some(solid) = &bg_fill.solid_fill {
                                let bg_color_hex = format_color(solid.color.as_ref(), color_scheme);
                                write!(cell_style, " background-color:{};", bg_color_hex)?;
                            }
                        }
                        // TODO: Handle contentAlignment (vertical-align: middle/bottom)
                        // match props.content_alignment { ... }
                    }

                    write!(
                        svg_output,
                        "        <td{} style=\"{}\">",
                        td_attrs, cell_style
                    )?;

                    if let Some(text) = &cell.text {
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
                            cell_para_style.as_ref(),
                            &cell_text_style_base,
                            color_scheme,
                            None,
                            svg_output,
                        )?;
                    } else {
                        write!(svg_output, "&nbsp;")?;
                    }
                    write!(svg_output, "</td>")?;
                    writeln!(svg_output)?;
                }
                if !cells.is_empty() {
                    write!(svg_output, "      ")?;
                }
            }
            write!(svg_output, "</tr>")?;
        }
        if !rows.is_empty() {
            writeln!(svg_output)?;
            write!(svg_output, "    ")?;
        }
    }

    // --- Closing Tags ---
    write!(svg_output, "</table>")?;
    writeln!(svg_output)?;
    write!(svg_output, "  </div>")?;
    writeln!(svg_output)?;
    write!(svg_output, "</foreignObject>")?;

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
    // apply_transform gets the full matrix attribute string
    let (_, _) = apply_transform(transform, &mut img_attrs)?; // We only need the attribute string
    let width_pt = dimension_to_pt(size.and_then(|s| s.width.as_ref()));
    let height_pt = dimension_to_pt(size.and_then(|s| s.height.as_ref()));

    if width_pt <= 0.0 || height_pt <= 0.0 {
        warn!(
            "Skipping image element {} with zero dimensions ({}x{}pt).",
            element_id, width_pt, height_pt
        );
        return Ok(());
    }

    if let Some(url) = &image_data.content_url {
        let safe_url = url; // Assuming URL is safe enough for XML attribute
                            // Apply transform directly to the <image> tag. Position at (0,0) relative to the transform.
        write!(
            svg_output,
            r#"<image x="0" y="0" width="{}" height="{}" xlink:href="{}"{} preserveAspectRatio="xMidYMid meet" data-object-id="{}"/>"#,
            width_pt,
            height_pt,
            safe_url,
            img_attrs,
            element_id // img_attrs contains the transform
        )?;
    } else {
        warn!("Image element {} is missing contentUrl.", element_id);
        // Apply transform to the placeholder group
        write!(
            svg_output,
            "<g data-object-id=\"{}\"{}>", // Group gets the transform
            element_id, img_attrs
        )?;
        // Rect and text are positioned at (0,0) within the transformed group
        write!(
            svg_output,
            r#"<rect width="{}" height="{}" style="fill:#e0e0e0; stroke:gray; fill-opacity:0.5;" />"#,
            width_pt, height_pt
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
        // DON'T CHANGE THIS!
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
        } else {
            // Handle case where line_fill itself is None
            stroke_color = "none".to_string();
            stroke_opacity = 0.0;
        }

        // Only apply stroke styles if color is not "none"
        if stroke_color != "none" {
            write!(line_style, "stroke:{}; ", stroke_color)?;
            write!(line_style, "stroke-opacity:{}; ", stroke_opacity)?;

            // Stroke Weight
            let stroke_width_pt = dimension_to_pt(props.weight.as_ref());
            let effective_stroke_width = if stroke_width_pt > 0.0 {
                stroke_width_pt
            } else {
                1.0 // Min 1pt width for visible lines if properties exist but width is 0
            };
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
            // If stroke color is explicitly "none", don't render the line or set stroke:none
            write!(line_style, "stroke:none; ")?;
        }
    } else {
        // Default style if no lineProperties are defined
        write!(
            line_style,
            "stroke:{}; stroke-width:1pt; stroke-opacity:1.0; ",
            DEFAULT_TEXT_COLOR
        )?;
    }

    // 3. Write the SVG <line> element only if style is not stroke:none
    if !line_style.contains("stroke:none;") {
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
    } else {
        // Line style resolved to stroke:none, skip rendering the line element entirely.
        eprintln!(
            "Debug: Skipping line element {} because its effective stroke is 'none'.",
            element_id
        );
    }

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
    let (tx_pt, ty_pt) = apply_transform(transform, &mut ph_attrs)?; // Get transform attributes and translation
    let width_pt = dimension_to_pt(size.and_then(|s| s.width.as_ref())).max(20.0); // Min width
    let height_pt = dimension_to_pt(size.and_then(|s| s.height.as_ref())).max(10.0); // Min height

    // Apply transform to a group containing the placeholder visuals
    // Use matrix from ph_attrs if present, otherwise use translate for position.
    let group_transform = if ph_attrs.is_empty() && (tx_pt != 0.0 || ty_pt != 0.0) {
        format!(r#" transform="translate({} {})""#, tx_pt, ty_pt)
    } else {
        ph_attrs // Contains the full matrix or is empty
    };

    write!(
        svg_output,
        "<g data-object-id=\"{}\"{}>",
        element_id,
        group_transform // Use consolidated transform
    )?;
    // Dashed rectangle at 0,0 within group
    write!(
        svg_output,
        r#"<rect width="{}" height="{}" style="fill:#f0f0f0; stroke:lightgray; stroke-dasharray:3 3; fill-opacity:0.5;" />"#,
        width_pt, height_pt
    )?;
    // Text label within group
    write!(
        svg_output,
        r#"<text x="2" y="2" dy="0.8em" style="font-family:sans-serif; font-size:8pt; fill:gray;">{}</text>"#,
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

    match &element.element_kind {
        PageElementKind::Shape(shape) => {
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
            // apply_transform returns translation separately, but we just need the attribute string here.
            let _ = apply_transform(element.transform.as_ref(), &mut group_attrs)?;
            writeln!(
                svg_output,
                "<g data-object-id=\"{}_group\" {}>", // Add data-id suffix for clarity
                element.object_id,
                group_attrs // Add group transform attributes
            )?;

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
