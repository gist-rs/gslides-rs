use crate::models::{
    colors::{
        ColorScheme, OpaqueColor, OpaqueColorContent, OptionalColor, ThemeColorPair, ThemeColorType,
    },
    common::{AffineTransform, Dimension, Size, Unit},
    elements::{PageElement, PageElementKind},
    page::Page,
    page_properties::PageBackgroundFill,
    placeholder::Placeholder,
    presentation::Presentation,
    properties::{Alignment, ParagraphStyle, TextStyle},
    shape::Shape,
    table::Table,
    text::TextContent,
    text_element::TextElementKind,
};
use std::{collections::HashMap, fmt::Write};
use thiserror::Error;

// --- Error Type ---

#[derive(Error, Debug)]
pub enum SvgConversionError {
    #[error("Formatting error: {0}")]
    FormatError(#[from] std::fmt::Error),
    #[error("Missing expected data: {0}")]
    MissingData(String),
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

type Result<T> = std::result::Result<T, SvgConversionError>;

// --- Constants & Defaults ---

// Conversion factors (assuming 96 DPI for px equivalence, but mainly using pt)
const PT_PER_INCH: f64 = 72.0;
const EMU_PER_INCH: f64 = 914400.0;
const EMU_PER_PT: f64 = EMU_PER_INCH / PT_PER_INCH; // Approx 12700

const DEFAULT_FONT_SIZE_PT: f64 = 11.0; // Default fallback font size
const DEFAULT_FONT_FAMILY: &str = "Arial"; // Default fallback font
const DEFAULT_TEXT_COLOR: &str = "#000000"; // Black

// --- Helper Functions ---

/// Escapes special XML characters for SVG text content.
fn escape_svg_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escapes special XML characters for HTML text content (within foreignObject).
fn escape_html_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
    // HTML also needs quotes escaped in attributes, but less critical for content here.
    // .replace('"', "&quot;")
    // .replace('\'', "&apos;")
}

/// Converts a Dimension to points (pt), returning 0.0 if None or invalid.
fn dimension_to_pt(dim: Option<&Dimension>) -> f64 {
    match dim {
        Some(d) => {
            let magnitude = d.magnitude.unwrap_or(0.0);
            match d.unit.as_ref() {
                Some(Unit::Pt) => magnitude,
                Some(Unit::Emu) => magnitude / EMU_PER_PT,
                _ => 0.0, // Treat unspecified or unknown as 0
            }
        }
        None => 0.0,
    }
}

/// Converts an OpaqueColor to an SVG color string (e.g., #RRGGBB).
/// TODO: Implement full ThemeColor resolution. Currently only supports RGB + hardcoded ACCENT1.
fn format_color(
    color_opt: Option<&OpaqueColor>,
    color_scheme: Option<&ColorScheme>, // <-- Added color_scheme argument
) -> String {
    match color_opt {
        Some(opaque_color) => match &opaque_color.color_kind {
            OpaqueColorContent::RgbColor(rgb) => {
                let r = (rgb.red.unwrap_or(0.0) * 255.0).round() as u8;
                let g = (rgb.green.unwrap_or(0.0) * 255.0).round() as u8;
                let b = (rgb.blue.unwrap_or(0.0) * 255.0).round() as u8;
                format!("#{:02x}{:02x}{:02x}", r, g, b)
            }
            OpaqueColorContent::ThemeColor(theme_color_type) => {
                // Attempt to resolve theme color using the provided scheme
                if let Some(scheme) = color_scheme {
                    // scheme is &ColorScheme
                    // scheme.colors is Vec<ThemeColorPair>, iterate directly
                    if let Some(theme_pair) = scheme
                        .colors // Directly access the vector
                        .iter() // Iterate over &ThemeColorPair
                        .find(|pair| pair.theme_color_type == *theme_color_type)
                    // Find the pair matching the type
                    {
                        // theme_pair.color is RgbColor. Need to construct OpaqueColor for recursive call.
                        let resolved_opaque_color = OpaqueColor {
                            color_kind: OpaqueColorContent::RgbColor(theme_pair.color.clone()),
                        };
                        // Format the resolved color. Pass None for scheme to prevent infinite recursion.
                        return format_color(Some(&resolved_opaque_color), None);
                    }
                }
                // Fallback if scheme is missing or color type not found
                DEFAULT_TEXT_COLOR.to_string()
            }
        },
        None => DEFAULT_TEXT_COLOR.to_string(), // Fallback if OpaqueColor itself is missing
    }
}

/// Converts an OptionalColor (often used for background/foreground) to SVG fill/opacity attributes.
/// Returns a tuple: (fill_color, fill_opacity).
/// Uses DEFAULT_TEXT_COLOR if color is None, returns "none" if color is transparent (opaque_color is None).
/// Looks up theme colors in the provided ColorScheme.
fn format_optional_color(
    optional_color: Option<&OptionalColor>,
    color_scheme: Option<&ColorScheme>, // Expects &ColorScheme
) -> (String, String) {
    match optional_color {
        Some(opt_color) => {
            // Check if the optional color contains an opaque color
            match &opt_color.opaque_color {
                Some(opaque_color) => {
                    // Opaque color exists, format it (handles both RGB and ThemeColor lookup via format_color)
                    let color_hex = format_color(Some(opaque_color), color_scheme);
                    // TODO: Alpha handling might need refinement if OpaqueColor provides it later
                    (color_hex, "1".to_string())
                }
                // opaque_color field was None in the JSON, meaning transparent
                None => ("none".to_string(), "0".to_string()),
            }
        }
        // OptionalColor struct itself was None
        None => (DEFAULT_TEXT_COLOR.to_string(), "1".to_string()),
    }
}

/// Applies TextStyle properties to an SVG `<tspan>` or `<text>` element's style attribute.
/// Applies TextStyle properties to an SVG `<tspan>` or `<text>` element's style attribute.
fn apply_text_style(
    style: Option<&TextStyle>,
    svg_style: &mut String,
    color_scheme: Option<&ColorScheme>, // <-- Added color_scheme argument
) -> Result<()> {
    if let Some(ts) = style {
        // Font Family
        write!(
            svg_style,
            "font-family:'{}'; ",
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
                DEFAULT_FONT_SIZE_PT
            }
        )?;

        // Foreground Color
        let (fg_color, fg_opacity) =
            format_optional_color(ts.foreground_color.as_ref(), color_scheme); // Pass scheme
        write!(
            svg_style,
            "fill:{}; fill-opacity:{}; ",
            fg_color, fg_opacity
        )?;

        // Background Color (Difficult to represent reliably for tspans, maybe ignore for now?)
        // let (bg_color, bg_opacity) = format_optional_color(ts.background_color.as_ref(), color_scheme); // Pass scheme
        // if bg_color != "none" { ... }

        // Bold
        if ts.bold.unwrap_or(false) {
            write!(svg_style, "font-weight:bold; ")?;
        } else {
            write!(svg_style, "font-weight:normal; ")?;
        }

        // Italic
        if ts.italic.unwrap_or(false) {
            write!(svg_style, "font-style:italic; ")?;
        } else {
            write!(svg_style, "font-style:normal; ")?;
        }

        // Underline / Strikethrough
        let mut decorations = Vec::new();
        if ts.underline.unwrap_or(false) {
            decorations.push("underline");
        }
        if ts.strikethrough.unwrap_or(false) {
            decorations.push("line-through");
        }
        if !decorations.is_empty() {
            write!(svg_style, "text-decoration:{}; ", decorations.join(" "))?;
        } else {
            write!(svg_style, "text-decoration:none; ")?;
        }

        // Baseline Offset (Superscript/Subscript)
        match ts.baseline_offset {
            Some(crate::models::properties::BaselineOffset::Superscript) => {
                write!(svg_style, "baseline-shift:super; ")?;
            }
            Some(crate::models::properties::BaselineOffset::Subscript) => {
                write!(svg_style, "baseline-shift:sub; ")?;
            }
            _ => { /* Use default baseline */ }
        }

        // Small Caps (SVG: font-variant)
        if ts.small_caps.unwrap_or(false) {
            write!(svg_style, "font-variant:small-caps; ")?;
        } else {
            write!(svg_style, "font-variant:normal; ")?;
        }
    }
    Ok(())
}

/// Applies ParagraphStyle properties (mainly alignment) to an SVG `<text>` element.
fn apply_paragraph_style(
    style: Option<&ParagraphStyle>,
    svg_attrs: &mut String,
    x: f64,
    width: f64,
) -> Result<f64> {
    let mut adjusted_x = x;
    let mut text_anchor = "start"; // SVG default

    if let Some(ps) = style {
        match ps.alignment {
            Some(Alignment::Center) => {
                text_anchor = "middle";
                adjusted_x = x + width / 2.0;
            }
            Some(Alignment::End) => {
                text_anchor = "end";
                adjusted_x = x + width;
            }
            Some(Alignment::Justified) => {
                // Justification is complex in SVG. text-align="justify" exists but support varies.
                // text-anchor="start" is a fallback.
                // Maybe add 'text-align:justify;' to style?
                // style.push_str("text-align:justify; ");
                text_anchor = "start"; // Stick with start for broader compatibility
            }
            _ => {
                // Start or Unspecified
                text_anchor = "start";
                adjusted_x = x;
            }
        }
        // Indentation and Spacing (line_spacing, space_above/below) are hard to map directly to SVG text attributes.
        // Might need manual y/dy adjustments or just ignore for basic conversion.
        // Let's ignore these for now.
    }

    write!(svg_attrs, r#" text-anchor="{}""#, text_anchor)?;
    Ok(adjusted_x) // Return the potentially adjusted x based on anchor
}

// Helper for recursive group element collection
fn collect_elements_recursive<'a>(elements: &'a [PageElement], map: &mut ElementsMap<'a>) {
    for element in elements {
        map.insert(element.object_id.clone(), element);
        if let PageElementKind::ElementGroup(group) = &element.element_kind {
            collect_elements_recursive(&group.children, map);
        }
    }
}

// Directly iterate over pages instead of using a closure
fn collect_page_elements<'a>(pages: Option<&'a Vec<Page>>, elements_map: &mut ElementsMap<'a>) {
    if let Some(page_list) = pages {
        for page in page_list {
            if let Some(elements) = &page.page_elements {
                for element in elements {
                    elements_map.insert(element.object_id.clone(), element);
                    if let PageElementKind::ElementGroup(group) = &element.element_kind {
                        collect_elements_recursive(&group.children, elements_map);
                    }
                }
            }
        }
    }
}

/// Applies AffineTransform to an SVG element's `transform` attribute.
/// Converts EMU translation to points.
fn apply_transform(
    transform: Option<&AffineTransform>,
    svg_attrs: &mut String,
) -> Result<(f64, f64, f64)> {
    let mut tx_pt = 0.0;
    let mut ty_pt = 0.0;
    let width_pt = 0.0; // Placeholder width, actual size needed separately

    if let Some(tf) = transform {
        let scale_x = tf.scale_x.unwrap_or(1.0);
        let scale_y = tf.scale_y.unwrap_or(1.0);
        let shear_x = tf.shear_x.unwrap_or(0.0);
        let shear_y = tf.shear_y.unwrap_or(0.0);
        // Translations need unit conversion
        tx_pt = dimension_to_pt(Some(&Dimension {
            magnitude: tf.translate_x,
            unit: tf.unit.clone(), // Use the transform's unit
        }));
        ty_pt = dimension_to_pt(Some(&Dimension {
            magnitude: tf.translate_y,
            unit: tf.unit.clone(),
        }));

        // Construct the SVG transform matrix: matrix(a, b, c, d, e, f)
        // a = scaleX, b = shearY, c = shearX, d = scaleY, e = translateX, f = translateY
        write!(
            svg_attrs,
            r#" transform="matrix({} {} {} {} {} {})""#,
            scale_x, shear_y, shear_x, scale_y, tx_pt, ty_pt
        )?;
    } else {
        // No transform attribute if none provided
    }
    Ok((tx_pt, ty_pt, width_pt)) // Return position for potential use
}

// Step 2: Define lookup maps type alias for clarity
type LayoutsMap<'a> = HashMap<String, &'a Page>;
type MastersMap<'a> = HashMap<String, &'a Page>;
type ElementsMap<'a> = HashMap<String, &'a PageElement>;

// Step 3: Create a function to build lookup maps for efficient access
// Step 3: Create a function to build lookup maps for efficient access
fn build_lookup_maps<'a>(
    presentation: &'a Presentation,
) -> (LayoutsMap<'a>, MastersMap<'a>, ElementsMap<'a>) {
    let mut layouts_map: LayoutsMap = HashMap::new();
    if let Some(layouts) = &presentation.layouts {
        for layout in layouts {
            layouts_map.insert(layout.object_id.clone(), layout);
        }
    }

    let mut masters_map: MastersMap = HashMap::new();
    if let Some(masters) = &presentation.masters {
        for master in masters {
            masters_map.insert(master.object_id.clone(), master);
        }
    }

    // Build a map of all page elements for quick placeholder lookup
    let mut elements_map: ElementsMap = HashMap::new();

    // Use the helper functions (defined outside this function) to populate the map
    collect_page_elements(presentation.slides.as_ref(), &mut elements_map);
    collect_page_elements(presentation.layouts.as_ref(), &mut elements_map);
    collect_page_elements(presentation.masters.as_ref(), &mut elements_map);

    (layouts_map, masters_map, elements_map)
}

// Step 4: Create a helper to find the corresponding placeholder element on the layout/master
fn find_placeholder_element<'a>(
    shape_placeholder: &Placeholder,
    slide_layout_id: &str,
    layouts_map: &LayoutsMap<'a>,
    masters_map: &MastersMap<'a>,
    elements_map: &ElementsMap<'a>,
) -> Option<&'a PageElement> {
    let shape_placeholder_parent_object_id = shape_placeholder
        .parent_object_id
        .as_ref()
        .expect("Invalid id")
        .clone();
    // 1. Find the direct parent placeholder on the layout
    if let Some(layout_placeholder_element) = elements_map.get(&shape_placeholder_parent_object_id)
    {
        // Check if this layout placeholder itself inherits from a master placeholder
        if let Some(_placeholder_info) = layout_placeholder_element
            .element_kind
            .as_shape()
            .and_then(|s| s.placeholder.as_ref())
        {
            if let Some(master_placeholder_element) =
                elements_map.get(&shape_placeholder_parent_object_id)
            {
                // Found master placeholder element - this is often where default styles live
                return Some(master_placeholder_element);
            }
        }
        // Return the layout placeholder if no master link or master not found
        return Some(layout_placeholder_element);
    }

    // Fallback: Look through the specified layout directly if not found in elements_map quickly (should be rare)
    if let Some(layout) = layouts_map.get(slide_layout_id) {
        if let Some(elements) = &layout.page_elements {
            // TODO: Need recursive search here too for groups on layouts
            for element in elements {
                if element.object_id == shape_placeholder_parent_object_id {
                    return Some(element);
                }
            }
        }
        // If not found on layout, check the master referenced by the layout
        if let Some(master_id) = layout
            .layout_properties
            .as_ref()
            .map(|p| &p.master_object_id)
        {
            let master_id_2 = master_id.as_ref().expect("Invalid id");
            if let Some(master) = masters_map.get(master_id_2) {
                if let Some(elements) = &master.page_elements {
                    // TODO: Need recursive search here too
                    for element in elements {
                        if element.object_id == shape_placeholder_parent_object_id {
                            return Some(element);
                        }
                    }
                }
            }
        }
    }

    None // Placeholder parent not found
}

// Step 5: Create a helper to merge TextStyles (slide style overrides inherited)
fn merge_text_styles(
    specific_style: Option<&TextStyle>,
    inherited_style: Option<&TextStyle>,
) -> TextStyle {
    let mut merged = inherited_style.cloned().unwrap_or_default(); // Start with inherited or default

    if let Some(specific) = specific_style {
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
        // Language code might not make sense to merge this way, usually set explicitly.
        // if specific.language_code.is_some() {
        //     merged.language_code = specific.language_code.clone();
        // }
    }

    merged
}

// Step 6: Create a function to get the *effective* style for a specific text run,
// considering inheritance. This is complex due to matching text runs.
// Let's simplify: find the *first* relevant style in the placeholder for now.
fn get_placeholder_default_text_style(placeholder_element: &PageElement) -> Option<TextStyle> {
    match &placeholder_element.element_kind {
        PageElementKind::Shape(shape) => {
            if let Some(text) = &shape.text {
                if let Some(text_elements) = &text.text_elements {
                    // Find the first TextRun with a style defined.
                    // This is an approximation, as different nesting levels might have different defaults.
                    for element in text_elements {
                        if let Some(TextElementKind::TextRun(tr)) = &element.kind {
                            if let Some(style) = &tr.style {
                                return Some(style.clone());
                            }
                        }
                        // Maybe also check ParagraphMarker -> Bullet -> TextStyle? Less common for font size.
                    }
                }
            }
        }
        _ => { /* Placeholder element is not a shape? Or shape has no text? */ }
    }
    None
}

// --- Conversion Functions ---

/// Converts the text content of a shape or cell into SVG `<text>` and `<tspan>` elements.
fn convert_text_content_to_svg(
    text_content: &TextContent,
    effective_paragraph_style: Option<&ParagraphStyle>,
    effective_text_style_base: &TextStyle,
    transform_x: f64,
    transform_y: f64,
    element_width: f64,
    _element_height: f64,
    color_scheme: Option<&ColorScheme>, // <-- Added color_scheme argument
    svg_output: &mut String,
) -> Result<()> {
    let text_elements = match &text_content.text_elements {
        Some(elements) => elements,
        None => return Ok(()),
    };

    // Store paragraph-level info (bullets, potentially specific paragraph styles if not pre-merged)
    let mut para_bullets: HashMap<u32, String> = HashMap::new();
    // Note: We now assume paragraph style (like alignment) is pre-resolved in effective_paragraph_style

    for element in text_elements {
        if let Some(TextElementKind::ParagraphMarker(pm)) = &element.kind {
            if let Some(start_index) = element.start_index {
                // TODO: Handle bullet glyph lookup/rendering - this needs list context from TextContent.lists
                if let Some(bullet) = &pm.bullet {
                    // Lookup list properties based on bullet.list_id and nesting level
                    // For now, use placeholder glyph
                    let bullet_text = bullet.glyph.as_deref().unwrap_or("* ").to_string(); // Default bullet
                    para_bullets.insert(start_index, bullet_text);
                }
            }
        }
    }

    let mut current_y = transform_y;
    // Estimate line height based on the base font size.
    let base_font_size_pt = dimension_to_pt(effective_text_style_base.font_size.as_ref());
    let line_height_pt = if base_font_size_pt > 0.0 {
        base_font_size_pt * 1.2
    } else {
        DEFAULT_FONT_SIZE_PT * 1.2
    };
    let mut first_line_in_paragraph = true;
    // Use the pre-resolved paragraph style for alignment
    let current_para_style = effective_paragraph_style;

    for element in text_elements {
        let start_index = element.start_index.unwrap_or(0); // Use 0 if missing

        match &element.kind {
            Some(TextElementKind::ParagraphMarker(_)) => {
                if !first_line_in_paragraph {
                    current_y += line_height_pt;
                }
                first_line_in_paragraph = true;
                if let Some(_bullet_text) = para_bullets.get(&start_index) {
                    // Omit bullets for now
                }
            }
            Some(TextElementKind::TextRun(tr)) => {
                let content = tr.content.as_deref().unwrap_or("");
                if content.is_empty() || content == "\n" {
                    continue;
                }

                let run_specific_style = tr.style.as_ref();
                let final_run_style =
                    merge_text_styles(run_specific_style, Some(effective_text_style_base));

                let mut text_style_attr = String::new();
                apply_text_style(Some(&final_run_style), &mut text_style_attr, color_scheme)?; // Pass scheme

                if first_line_in_paragraph {
                    let mut para_attrs = String::new();
                    let adjusted_x = apply_paragraph_style(
                        current_para_style,
                        &mut para_attrs,
                        transform_x,
                        element_width,
                    )?;

                    let run_font_size_pt = dimension_to_pt(final_run_style.font_size.as_ref());
                    let y_pos = current_y
                        + if run_font_size_pt > 0.0 {
                            run_font_size_pt
                        } else {
                            line_height_pt / 1.2
                        };

                    write!(
                        svg_output,
                        r#"<text x="{}" y="{}"{}"#,
                        adjusted_x, y_pos, para_attrs
                    )?;
                    write!(svg_output, r#" style="{}">"#, text_style_attr)?;
                    write!(svg_output, "{}", escape_svg_text(content))?;
                    write!(svg_output, "</text>")?;

                    first_line_in_paragraph = false;
                } else {
                    eprintln!("Warning: Subsequent TextRuns in the same paragraph are currently skipped in SVG conversion.");
                }

                if content.contains('\n') {
                    current_y += line_height_pt;
                    first_line_in_paragraph = true;
                    eprintln!("Warning: Newlines within TextRuns reset to new line, potentially losing style continuity.");
                }
            }
            Some(TextElementKind::AutoText(at)) => {
                let content = at.content.as_deref().unwrap_or("");
                if content.is_empty() || content == "\n" {
                    continue;
                }

                let autotext_specific_style = at.style.as_ref();
                let final_autotext_style =
                    merge_text_styles(autotext_specific_style, Some(effective_text_style_base));

                let mut text_style_attr = String::new();
                apply_text_style(
                    Some(&final_autotext_style),
                    &mut text_style_attr,
                    color_scheme,
                )?; // Pass scheme

                if first_line_in_paragraph {
                    let mut para_attrs = String::new();
                    let adjusted_x = apply_paragraph_style(
                        current_para_style,
                        &mut para_attrs,
                        transform_x,
                        element_width,
                    )?;
                    let run_font_size_pt = dimension_to_pt(final_autotext_style.font_size.as_ref());
                    let y_pos = current_y
                        + if run_font_size_pt > 0.0 {
                            run_font_size_pt
                        } else {
                            line_height_pt / 1.2
                        };

                    write!(
                        svg_output,
                        r#"<text x="{}" y="{}"{}"#,
                        adjusted_x, y_pos, para_attrs
                    )?;
                    write!(svg_output, r#" style="{}">"#, text_style_attr)?;
                    write!(svg_output, "{}", escape_svg_text(content))?;
                    write!(svg_output, "</text>")?;
                    first_line_in_paragraph = false;
                } else {
                    eprintln!(
                        "Warning: Subsequent AutoText in the same paragraph are currently skipped."
                    );
                }
                if content.contains('\n') {
                    current_y += line_height_pt;
                    first_line_in_paragraph = true;
                    eprintln!("Warning: Newlines within AutoText reset to new line.");
                }
            }
            None => { /* Element kind is None, skip */ }
        }
    }

    Ok(())
}

/// Converts the text content of a table cell into basic HTML for `<foreignObject>`.
fn convert_text_content_to_html(
    text_content: &TextContent,
    color_scheme: Option<&ColorScheme>, // <-- Added color_scheme argument
    html_output: &mut String,
) -> Result<()> {
    let text_elements = match &text_content.text_elements {
        Some(elements) => elements,
        None => return Ok(()),
    };
    let mut paragraph_open = false; // Track if <p> is open

    for element in text_elements {
        match &element.kind {
            Some(TextElementKind::ParagraphMarker(_)) => {
                if paragraph_open {
                    write!(html_output, "</p>")?;
                    paragraph_open = false;
                }
                write!(html_output, "<p style=\"margin:0; padding:0;\">")?;
                paragraph_open = true;
            }
            Some(TextElementKind::TextRun(tr)) => {
                let content = tr.content.as_deref().unwrap_or("");
                if content.is_empty() {
                    continue;
                }

                if !paragraph_open {
                    write!(html_output, "<p style=\"margin:0; padding:0;\">")?;
                    paragraph_open = true;
                }

                let mut span_style = String::new();
                // Convert TextStyle to inline CSS
                if let Some(ts) = &tr.style {
                    write!(
                        span_style,
                        "font-family:'{}'; ",
                        ts.font_family.as_deref().unwrap_or(DEFAULT_FONT_FAMILY)
                    )?;
                    let font_size_pt = dimension_to_pt(ts.font_size.as_ref());
                    write!(
                        span_style,
                        "font-size:{}pt; ",
                        if font_size_pt > 0.0 {
                            font_size_pt
                        } else {
                            DEFAULT_FONT_SIZE_PT
                        }
                    )?;
                    // Use format_optional_color with the scheme
                    let (fg_color, _) =
                        format_optional_color(ts.foreground_color.as_ref(), color_scheme);
                    write!(span_style, "color:{}; ", fg_color)?;
                    if ts.bold.unwrap_or(false) {
                        write!(span_style, "font-weight:bold; ")?;
                    }
                    if ts.italic.unwrap_or(false) {
                        write!(span_style, "font-style:italic; ")?;
                    }
                    let mut decorations = Vec::new();
                    if ts.underline.unwrap_or(false) {
                        decorations.push("underline");
                    }
                    if ts.strikethrough.unwrap_or(false) {
                        decorations.push("line-through");
                    }
                    if !decorations.is_empty() {
                        write!(span_style, "text-decoration:{}; ", decorations.join(" "))?;
                    }
                    match ts.baseline_offset {
                        Some(crate::models::properties::BaselineOffset::Superscript) => {
                            write!(span_style, "vertical-align:super; ")?
                        }
                        Some(crate::models::properties::BaselineOffset::Subscript) => {
                            write!(span_style, "vertical-align:sub; ")?
                        }
                        _ => {}
                    }
                    if ts.small_caps.unwrap_or(false) {
                        write!(span_style, "font-variant:small-caps; ")?;
                    }
                }

                let html_content = escape_html_text(content).replace('\n', "<br/>");

                write!(
                    html_output,
                    r#"<span style="{}">{}</span>"#,
                    span_style, html_content
                )?;
            }
            Some(TextElementKind::AutoText(at)) => {
                let content = at.content.as_deref().unwrap_or("");
                if content.is_empty() {
                    continue;
                }
                if !paragraph_open {
                    write!(html_output, "<p style=\"margin:0; padding:0;\">")?;
                    paragraph_open = true;
                }
                // TODO: Apply styles similar to TextRun, passing color_scheme
                let html_content = escape_html_text(content).replace('\n', "<br/>");
                write!(html_output, "<span>{}</span>", html_content)?;
            }
            None => {}
        }
    }

    if paragraph_open {
        write!(html_output, "</p>")?;
    }

    Ok(())
}

/// Converts a Shape element (especially TextBox) to SVG.
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
    let (tx, ty, _) = apply_transform(transform, &mut shape_attrs)?;
    let width = dimension_to_pt(size.and_then(|s| s.width.as_ref()));
    let height = dimension_to_pt(size.and_then(|s| s.height.as_ref()));

    // Resolve effective styles
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
                if let Some(placeholder_base_style) =
                    get_placeholder_default_text_style(placeholder_element)
                {
                    effective_text_style_base = placeholder_base_style;
                }

                if let Some(placeholder_shape) = placeholder_element.element_kind.as_shape() {
                    if let Some(text) = &placeholder_shape.text {
                        if let Some(elements) = &text.text_elements {
                            for element in elements {
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

    // Render text content if present, passing the resolved styles AND color scheme
    if let Some(text) = &shape.text {
        if let Some(elements) = &text.text_elements {
            for element in elements {
                if let Some(TextElementKind::ParagraphMarker(pm)) = &element.kind {
                    if let Some(style) = &pm.style {
                        effective_paragraph_style = Some(style.clone());
                        break;
                    }
                }
            }
        }

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

/// Converts a Table element to SVG using `<foreignObject>` and HTML.
/// Converts a Table element to SVG using `<foreignObject>` and HTML.
fn convert_table_to_svg(
    table: &Table,
    transform: Option<&AffineTransform>,
    size: Option<&Size>,
    color_scheme: Option<&ColorScheme>, // <-- Added color_scheme argument
    svg_output: &mut String,
) -> Result<()> {
    let mut foreign_object_attrs = String::new();
    let (tx, ty, _) = apply_transform(transform, &mut foreign_object_attrs)?;
    let width = dimension_to_pt(size.and_then(|s| s.width.as_ref()));
    let height = dimension_to_pt(size.and_then(|s| s.height.as_ref()));

    if width <= 0.0 || height <= 0.0 {
        eprintln!("Warning: Skipping table with zero or negative dimensions.");
        return Ok(());
    }

    write!(
        svg_output,
        r#"<foreignObject x="{}" y="{}" width="{}" height="{}"{}>"#,
        tx, ty, width, height, foreign_object_attrs
    )?;
    write!(svg_output, r#"<div xmlns="http://www.w3.org/1999/xhtml">"#)?;
    write!(
        svg_output,
        r#"<table style="border-collapse: collapse; width:100%; height:100%; border: 1px solid #ccc;">"#
    )?;

    if let Some(rows) = &table.table_rows {
        for row in rows {
            write!(svg_output, "<tr>")?;
            if let Some(cells) = &row.table_cells {
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

                    let mut cell_style =
                        "border: 1px solid #eee; padding: 2pt; vertical-align: top;".to_string();
                    if let Some(props) = &cell.table_cell_properties {
                        if let Some(bg_fill) = &props.table_cell_background_fill {
                            if let Some(solid) = &bg_fill.solid_fill {
                                // Pass scheme for background color formatting
                                let bg_color = format_color(solid.color.as_ref(), color_scheme);
                                write!(cell_style, " background-color:{};", bg_color)?;
                            }
                        }
                    }

                    write!(svg_output, r#"<td{} style="{}">"#, td_attrs, cell_style)?;

                    if let Some(text) = &cell.text {
                        // Pass scheme for text content formatting
                        convert_text_content_to_html(text, color_scheme, svg_output)?;
                    }

                    write!(svg_output, "</td>")?;
                }
            }
            write!(svg_output, "</tr>")?;
        }
    }

    write!(svg_output, "</table></div></foreignObject>")?;

    Ok(())
}

// Step 7: Modify `convert_page_element_to_svg` to pass context
/// Converts a single PageElement to an SVG fragment.
/// Converts a single PageElement to an SVG fragment.
fn convert_page_element_to_svg(
    element: &PageElement,
    slide_layout_id: Option<&str>,
    layouts_map: &LayoutsMap,
    masters_map: &MastersMap,
    elements_map: &ElementsMap,
    color_scheme: Option<&ColorScheme>, // <-- Added color_scheme argument
    svg_output: &mut String,
) -> Result<()> {
    write!(svg_output, r#"<g data-object-id="{}">"#, element.object_id)?;

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
                color_scheme, // Pass scheme
                svg_output,
            )?;
        }
        PageElementKind::Table(table) => {
            convert_table_to_svg(
                table,
                element.transform.as_ref(),
                element.size.as_ref(),
                color_scheme, // Pass scheme
                svg_output,
            )?;
        }
        PageElementKind::ElementGroup(group) => {
            let mut group_attrs = String::new();
            apply_transform(element.transform.as_ref(), &mut group_attrs)?;
            writeln!(svg_output, "<g{}> <!-- Start Group -->", group_attrs)?; // Apply transform to group <g>

            for child_element in &group.children {
                convert_page_element_to_svg(
                    child_element,
                    slide_layout_id,
                    layouts_map,
                    masters_map,
                    elements_map,
                    color_scheme, // Pass scheme recursively
                    svg_output,
                )?;
            }
            write!(svg_output, "</g> <!-- End Group -->")?; // Close group <g>
        }
        PageElementKind::Image(_) => {
            // Placeholder for Image (unchanged, no colors to resolve)
            let mut img_attrs = String::new();
            let (tx, ty, _) = apply_transform(element.transform.as_ref(), &mut img_attrs)?;
            let width = dimension_to_pt(element.size.as_ref().and_then(|s| s.width.as_ref()));
            let height = dimension_to_pt(element.size.as_ref().and_then(|s| s.height.as_ref()));
            write!(
                svg_output,
                r#"<rect x="{}" y="{}" width="{}" height="{}" {} style="fill:#e0e0e0; stroke:gray; fill-opacity:0.5;" />"#,
                tx, ty, width, height, img_attrs
            )?;
            write!(
                svg_output,
                r#"<text x="{}" y="{}" dy="1em" style="font-size:8pt; fill:gray;">Image Placeholder</text>"#,
                tx + 2.0,
                ty + 2.0
            )?;
        }
        PageElementKind::Line(_) => {
            // Placeholder for Line (unchanged, stroke handled differently if needed)
            let mut line_attrs = String::new();
            let (tx, ty, _) = apply_transform(element.transform.as_ref(), &mut line_attrs)?;
            let width = dimension_to_pt(element.size.as_ref().and_then(|s| s.width.as_ref()));
            let height = dimension_to_pt(element.size.as_ref().and_then(|s| s.height.as_ref()));
            // TODO: Handle line color / stroke properties properly using color_scheme
            let line_color = DEFAULT_TEXT_COLOR; // Placeholder
            write!(
                svg_output,
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" {} style="stroke:{}; stroke-width:1;" />"#,
                tx,
                ty,
                tx + width,
                ty + height,
                line_attrs,
                line_color
            )?;
            write!(
                svg_output,
                r#"<text x="{}" y="{}" dy="1em" style="font-size:8pt; fill:gray;">Line Placeholder</text>"#,
                tx + 2.0,
                ty + 2.0
            )?;
        }
        _ => {
            // Default placeholder (unchanged)
            let mut ph_attrs = String::new();
            let (tx, ty, _) = apply_transform(element.transform.as_ref(), &mut ph_attrs)?;
            let width = dimension_to_pt(element.size.as_ref().and_then(|s| s.width.as_ref()));
            let height = dimension_to_pt(element.size.as_ref().and_then(|s| s.height.as_ref()));
            let type_name = match element.element_kind {
                PageElementKind::Video(_) => "Video",
                PageElementKind::WordArt(_) => "WordArt",
                PageElementKind::SheetsChart(_) => "SheetsChart",
                PageElementKind::SpeakerSpotlight(_) => "SpeakerSpotlight",
                _ => "Unknown",
            };

            write!(
                svg_output,
                r#"<rect x="{}" y="{}" width="{}" height="{}" {} style="fill:#f0f0f0; stroke:lightgray; stroke-dasharray: 3 3; fill-opacity:0.5;" />"#,
                tx, ty, width, height, ph_attrs
            )?;
            write!(
                svg_output,
                r#"<text x="{}" y="{}" dy="1em" style="font-size:8pt; fill:gray;">{} Placeholder</text>"#,
                tx + 2.0,
                ty + 2.0,
                type_name
            )?;
        }
    }

    // This closing </g> was outside the match, but it closes the data-object-id group added at the start.
    // Ensure the ElementGroup closing tag is handled correctly inside its match arm.
    if !matches!(element.element_kind, PageElementKind::ElementGroup(_)) {
        write!(svg_output, "</g>")?;
    } // Closing tag for non-group elements

    Ok(())
}

// Step 8: Modify `convert_slide_to_svg` to use the context
/// Converts a single slide (Page) to an SVG string.
fn convert_slide_to_svg(
    slide: &Page,
    presentation_page_size: Option<&Size>,
    layouts_map: &LayoutsMap,
    masters_map: &MastersMap,
    elements_map: &ElementsMap,
) -> Result<String> {
    let mut svg_string = String::new();

    let page_width_pt = dimension_to_pt(presentation_page_size.and_then(|s| s.width.as_ref()));
    let page_height_pt = dimension_to_pt(presentation_page_size.and_then(|s| s.height.as_ref()));

    if page_width_pt <= 0.0 || page_height_pt <= 0.0 {
        return Err(SvgConversionError::MissingData(
            "Invalid or missing presentation page size".to_string(),
        ));
    }

    writeln!(
        svg_string,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{0}pt" height="{1}pt" viewBox="0 0 {0} {1}">"#,
        page_width_pt, page_height_pt
    )?;

    // --- Determine the active ColorScheme ---
    // Priority: Slide -> Layout -> Master
    let mut active_color_scheme: Option<&ColorScheme> = None;

    // 1. Check Slide properties
    if let Some(props) = &slide.page_properties {
        if let Some(scheme) = &props
            .page_background_fill
            .as_ref()
            .and_then(|f| f.get_color_scheme())
        {
            // Assuming get_color_scheme() is a helper or direct access
            // Need modification if scheme is elsewhere in PageProperties
            active_color_scheme = Some(scheme);
            // This part needs adjustment based on where ColorScheme actually lives in PageProperties
            // For now, let's assume it's directly accessible or None.
            // Let's use the Master's scheme as the primary source for simplicity first.
            active_color_scheme = None; // Temporarily disable slide override check
        }
    }

    // 2. Find Master and check its properties (most common source)
    let master_id = slide
        .slide_properties
        .as_ref()
        .and_then(|p| p.master_object_id.as_ref())
        .or_else(|| {
            // If slide doesn't link master, try layout
            slide
                .slide_properties
                .as_ref()
                .and_then(|p| p.layout_object_id.as_ref())
                .and_then(|layout_id| layouts_map.get(layout_id))
                .and_then(|layout| layout.layout_properties.as_ref())
                .and_then(|lp| lp.master_object_id.as_ref())
        });

    if active_color_scheme.is_none() {
        if let Some(id) = master_id {
            if let Some(master) = masters_map.get(id) {
                if let Some(props) = &master.page_properties {
                    active_color_scheme = props.color_scheme.as_ref();
                }
            }
        }
    }

    // --- Render Slide Background (using resolved scheme if possible) ---
    // TODO: Properly resolve background fill color (solid, gradient, etc.) using the active_color_scheme
    let background_fill = active_color_scheme
        .and_then(|s| s.get_background_fill_color()) // Hypothetical helper
        .unwrap_or_else(|| "#FFFFFF".to_string()); // Default white

    writeln!(
        svg_string,
        r##"  <rect width="100%" height="100%" fill="{}"/>"##,
        background_fill // Use resolved or default background
    )?;

    // Get the layout ID for this slide (needed for placeholder resolution)
    let slide_layout_id = slide.slide_properties.as_ref().map(|props| {
        props
            .layout_object_id
            .as_ref()
            .expect("Invalid id")
            .as_str()
    });

    if let Some(elements) = &slide.page_elements {
        let mut sorted_elements: Vec<&PageElement> = elements.iter().collect();
        sorted_elements.sort_by(|a, b| crate::converters::markdown::compare_elements_by_y(a, b));

        for element in sorted_elements {
            writeln!(svg_string, "  <!-- Element ID: {} -->", element.object_id)?;
            // Pass context AND the resolved color scheme to element conversion
            convert_page_element_to_svg(
                element,
                slide_layout_id,
                layouts_map,
                masters_map,
                elements_map,
                active_color_scheme, // Pass the resolved scheme
                &mut svg_string,
            )?;
            writeln!(svg_string)?;
        }
    }

    writeln!(svg_string, "</svg>")?;
    Ok(svg_string)
}

// Step 9: Modify `convert_presentation_to_svg` to build and pass maps
/// Converts a Google Slides presentation into a vector of SVG strings, one for each slide.
/// Focuses on converting text elements and tables (using HTML within foreignObject).
/// Other elements are rendered as placeholders.
/// Groups are handled recursively.
/// Handles basic text style inheritance from placeholders.
///
/// # Arguments
/// * `presentation` - A reference to the `Presentation` object.
///
/// # Returns
/// A `Result` containing a `Vec<String>` where each string is the SVG representation of a slide,
/// or an `SvgConversionError` on failure.
pub fn convert_presentation_to_svg(presentation: &Presentation) -> Result<Vec<String>> {
    let mut svg_slides = Vec::new();

    // Build lookup maps once
    let (layouts_map, masters_map, elements_map) = build_lookup_maps(presentation);

    if let Some(slides) = &presentation.slides {
        for (index, slide) in slides.iter().enumerate() {
            match convert_slide_to_svg(
                slide,
                presentation.page_size.as_ref(),
                // Pass maps
                &layouts_map,
                &masters_map,
                &elements_map,
            ) {
                Ok(svg_content) => svg_slides.push(svg_content),
                Err(e) => {
                    eprintln!(
                        "Error converting slide {} (ID: {}): {}",
                        index + 1,
                        slide.object_id,
                        e
                    );
                    return Err(e);
                }
            }
        }
    }

    Ok(svg_slides)
}

impl ColorScheme {
    fn get_background_fill_color(&self) -> Option<String> {
        // self.colors is Vec<ThemeColorPair>
        self.colors
            .iter()
            // Find the pair where the theme_color_type matches Background1
            .find(|pair: &&ThemeColorPair| pair.theme_color_type == ThemeColorType::Background1)
            // If found...
            .map(|found_pair: &ThemeColorPair| {
                // Construct an OpaqueColor wrapping the RgbColor from the theme pair
                let opaque_color = OpaqueColor {
                    color_kind: OpaqueColorContent::RgbColor(found_pair.color.clone()),
                };
                // Format this constructed OpaqueColor
                format_color(Some(&opaque_color), None)
            })
    }
}

// Add missing trait AsShape (if not already present, seems it was there before)
trait AsShape {
    fn as_shape(&self) -> Option<&Shape>;
}

impl AsShape for PageElementKind {
    fn as_shape(&self) -> Option<&Shape> {
        match self {
            PageElementKind::Shape(s) => Some(s),
            _ => None,
        }
    }
}

// Add missing helper for PageBackgroundFill (if needed for slide background)
trait GetColorScheme {
    fn get_color_scheme(&self) -> Option<&ColorScheme>;
}

impl GetColorScheme for PageBackgroundFill {
    fn get_color_scheme(&self) -> Option<&ColorScheme> {
        // Implementation depends on where ColorScheme might be stored within PageBackgroundFill variants
        // e.g., if it's part of StretchedPictureFill, SolidFill etc.
        // For now, return None as it's typically in PageProperties.
        None
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::presentation::Presentation;
    use std::fs;

    #[test]
    fn test_svg_conversion_from_json() {
        // Load a sample presentation JSON
        let json_path = "changed_presentation.json"; // Use your test file
        let json_string =
            fs::read_to_string(json_path).expect("Should have been able to read the file");
        let presentation: Presentation =
            serde_json::from_str(&json_string).expect("Failed to deserialize presentation JSON");

        // Convert to SVG
        let svg_results = convert_presentation_to_svg(&presentation);

        match svg_results {
            Ok(svg_vec) => {
                assert!(
                    !svg_vec.is_empty(),
                    "SVG conversion should produce output for slides."
                );

                // Optionally save each SVG to a file for inspection
                for (i, svg_content) in svg_vec.iter().enumerate() {
                    let output_path = format!("test_slide_{}.svg", i + 1);
                    let err_msg = format!("Unable to write SVG file: {}", output_path);
                    fs::write(&output_path, svg_content).expect(&err_msg);
                    println!("SVG for slide {} saved to {}", i + 1, output_path);

                    // Basic checks on SVG content
                    assert!(svg_content.starts_with("<svg"));
                    assert!(svg_content.ends_with("</svg>\n")); // Check for closing tag and newline
                    assert!(svg_content.contains("xmlns=\"http://www.w3.org/2000/svg\""));

                    // Check if the specific text "Hello" has the inherited font size applied
                    if i == 0 {
                        // Assuming the first slide has the "Hello" text box
                        assert!(
                            svg_content.contains(r#"font-size:52pt;"#), // Inherited from p2_i0 placeholder
                            "Expected font-size:52pt for 'Hello' text was not found in slide 1 SVG."
                        );
                        assert!(
                            svg_content.contains(r#"fill:#ff0000;"#), // Red (r=1) specified in textRun
                            "Expected fill:#ff0000 for 'こんにちは' text was not found in slide 1 SVG."
                        );
                        assert!(
                             svg_content.contains(r#"font-family:'Oswald';"#), // Specified in textRun
                             "Expected font-family:'Oswald' for 'こんにちは' text was not found in slide 1 SVG."
                         );
                    }
                    // Check if the specific text "world" has its own styles applied correctly
                    if i == 0 {
                        // Assuming the first slide has the "world" text box
                        assert!(
                            svg_content.contains(r#"font-size:28pt;"#), // Inherited from its placeholder? Check p2_i1
                            "Expected font-size:28pt for '世界' text was not found in slide 1 SVG."
                        );
                        assert!(
                            svg_content.contains(r#"fill:#00ff00;"#), // Green (g=1) specified in textRun
                            "Expected fill:#00ff00 for '世界' text was not found in slide 1 SVG."
                        );
                        assert!(
                               svg_content.contains(r#"font-family:'Roboto Mono';"#), // Specified in textRun
                              "Expected font-family:'Roboto Mono' for '世界' text was not found in slide 1 SVG."
                          );
                        // Check that bold is NOT applied (explicitly false in JSON)
                        assert!(
                               svg_content.contains(r#"font-weight:normal;"#) && !svg_content.contains(r#"font-weight:bold;"#),
                               "Expected font-weight:normal for '世界' text was not found in slide 1 SVG."
                           );
                    }

                    // Check if tables were processed (if expected)
                    // assert!(svg_content.contains("<foreignObject"));
                    // assert!(svg_content.contains("<table"));
                    // Check if groups were processed (if expected) - look for nested <g> or comments
                    // assert!(svg_content.contains("<!-- Start Group -->"));
                }
            }
            Err(e) => {
                panic!("SVG Conversion failed: {}", e);
            }
        }
    }
}
