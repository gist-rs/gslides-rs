//! Handles the overall structure of the presentation conversion,
//! including building lookup maps, resolving inheritance (placeholders),
//! and converting slides.

use super::{
    constants::*,
    elements::convert_page_element_to_svg,
    error::{Result, SvgConversionError},
    utils::{dimension_to_pt, format_color, AsShape},
};
use crate::models::{
    colors::{ColorScheme, OpaqueColor, OpaqueColorContent, ThemeColorType},
    elements::{PageElement, PageElementKind},
    page::Page,
    placeholder::Placeholder,
    presentation::Presentation,
    properties::TextStyle,
    text_element::TextElementKind,
};
use std::{collections::HashMap, fmt::Write};

// Type aliases for lookup maps for clarity
pub(crate) type LayoutsMap<'a> = HashMap<String, &'a Page>;
pub(crate) type MastersMap<'a> = HashMap<String, &'a Page>;
pub(crate) type ElementsMap<'a> = HashMap<String, &'a PageElement>;

/// Recursively collects all page elements (including those inside groups) into a map.
fn collect_elements_recursive<'a>(elements: &'a [PageElement], map: &mut ElementsMap<'a>) {
    for element in elements {
        // Use entry API for potential efficiency and clarity, though clone might be necessary if IDs aren't unique across types?
        // Assuming object_ids are unique presentation-wide.
        map.insert(element.object_id.clone(), element);
        if let PageElementKind::ElementGroup(group) = &element.element_kind {
            collect_elements_recursive(&group.children, map);
        }
    }
}

/// Collects page elements from a list of pages (slides, layouts, or masters) into the elements map.
fn collect_page_elements<'a>(pages: Option<&'a Vec<Page>>, elements_map: &mut ElementsMap<'a>) {
    if let Some(page_list) = pages {
        for page in page_list {
            if let Some(elements) = &page.page_elements {
                // Use the recursive helper to handle nested groups correctly
                collect_elements_recursive(elements, elements_map);
            }
        }
    }
}

/// Builds lookup maps for layouts, masters, and all page elements within a presentation.
/// These maps allow for efficient access during inheritance resolution and rendering.
///
/// # Arguments
/// * `presentation` - A reference to the `Presentation` object.
///
/// # Returns
/// A tuple containing `(LayoutsMap, MastersMap, ElementsMap)`.
pub(crate) fn build_lookup_maps(
    presentation: &Presentation,
) -> (LayoutsMap, MastersMap, ElementsMap) {
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

    // Build a map of *all* page elements for quick lookup across slides, layouts, and masters.
    let mut elements_map: ElementsMap = HashMap::new();
    collect_page_elements(presentation.slides.as_ref(), &mut elements_map);
    collect_page_elements(presentation.layouts.as_ref(), &mut elements_map);
    collect_page_elements(presentation.masters.as_ref(), &mut elements_map);

    (layouts_map, masters_map, elements_map)
}

/// Finds the corresponding placeholder `PageElement` on a layout or master slide
/// that a shape on the current slide inherits from.
/// This is used to find default styles and properties.
///
/// # Arguments
/// * `shape_placeholder` - The `Placeholder` info from the shape on the current slide.
/// * `slide_layout_id` - The object ID of the layout slide used by the current slide.
/// * `layouts_map` - Pre-built map of layout pages.
/// * `masters_map` - Pre-built map of master pages.
/// * `elements_map` - Pre-built map of all page elements.
///
/// # Returns
/// An `Option<&'a PageElement>` pointing to the placeholder element on the layout/master,
/// or `None` if it cannot be found.
pub(crate) fn find_placeholder_element<'a>(
    shape_placeholder: &Placeholder,
    slide_layout_id: &str,
    layouts_map: &LayoutsMap<'a>,
    masters_map: &MastersMap<'a>,
    elements_map: &ElementsMap<'a>,
) -> Option<&'a PageElement> {
    // The placeholder on the slide shape points to its parent placeholder on the layout or master.
    let parent_placeholder_object_id = match &shape_placeholder.parent_object_id {
        Some(id) => id,
        None => return None, // Cannot find parent if ID is missing
    };

    // 1. Directly look up the parent placeholder element using the elements_map.
    // This is the most efficient way and covers elements on layouts and masters.
    if let Some(parent_element) = elements_map.get(parent_placeholder_object_id) {
        return Some(parent_element);
    }

    // --- Fallback Search (Less efficient, potentially redundant if elements_map is comprehensive) ---
    // This fallback might be necessary if the elements_map wasn't populated correctly
    // or if dealing with very complex inheritance structures not covered by direct lookup.

    // 2. Look for the parent placeholder specifically on the slide's layout.
    if let Some(layout) = layouts_map.get(slide_layout_id) {
        if let Some(layout_elements) = &layout.page_elements {
            let mut elements_to_search: Vec<&PageElement> = layout_elements.iter().collect();
            while let Some(element) = elements_to_search.pop() {
                if element.object_id == *parent_placeholder_object_id {
                    return Some(element);
                }
                if let PageElementKind::ElementGroup(group) = &element.element_kind {
                    elements_to_search.extend(&group.children);
                }
            }
        }

        // 3. If not found on the layout, find the layout's master and search there.
        if let Some(master_id) = layout
            .layout_properties
            .as_ref()
            .and_then(|p| p.master_object_id.as_ref())
        {
            if let Some(master) = masters_map.get(master_id) {
                if let Some(master_elements) = &master.page_elements {
                    let mut elements_to_search: Vec<&PageElement> =
                        master_elements.iter().collect();
                    while let Some(element) = elements_to_search.pop() {
                        if element.object_id == *parent_placeholder_object_id {
                            return Some(element);
                        }
                        if let PageElementKind::ElementGroup(group) = &element.element_kind {
                            elements_to_search.extend(&group.children);
                        }
                    }
                }
            }
        }
    }

    // Placeholder parent element could not be found anywhere.
    eprintln!(
        "Warning: Could not find placeholder parent element with ID: {}",
        parent_placeholder_object_id
    );
    None
}

/// Extracts the *default* text style from a placeholder element (typically a Shape).
/// This is used as the base style for text within shapes that inherit from this placeholder.
/// It approximates the default style by looking first at the list style for nesting level 0,
/// and then falling back to the style of the first `TextRun` within the placeholder's text.
///
/// # Arguments
/// * `placeholder_element` - The placeholder `PageElement` (likely a Shape) on the layout/master.
///
/// # Returns
/// An `Option<TextStyle>` containing the cloned default style, or `None` if no style could be found.
pub(crate) fn get_placeholder_default_text_style(
    placeholder_element: &PageElement,
) -> Option<TextStyle> {
    // Ensure the placeholder is a Shape and has text content
    if let Some(shape) = placeholder_element.element_kind.as_shape() {
        if let Some(text) = &shape.text {
            // --- Strategy 1: Look for explicit list style for nesting level 0 ---
            // This is often the most reliable source for placeholder default styles.
            if let Some(lists) = &text.lists {
                // Find the listId associated with the first paragraph (nesting level 0)
                let first_para_list_id = text
                    .text_elements
                    .as_ref()
                    .and_then(|elements| elements.first())
                    .and_then(|first_element| first_element.kind.as_ref())
                    .and_then(|kind| match kind {
                        TextElementKind::ParagraphMarker(pm) => {
                            pm.bullet.as_ref().map(|b| &b.list_id)
                        }
                        _ => None,
                    });

                if let Some(list_id) = first_para_list_id {
                    // Use and_then for safer chaining, avoiding expects
                    if let Some(style) = lists
                        .get(list_id.clone().expect("Invalid id").as_str()) // Get ListProperties by &String list_id
                        .and_then(|list_props| list_props.nesting_level.as_ref()) // Get Option<&IndexMap<i32, NestingLevel>>
                        .and_then(|nesting_map| nesting_map.get(&0)) // Get Option<&NestingLevel> using i32 key 0
                        .and_then(|level_0_props| level_0_props.bullet_style.as_ref())
                    // Get Option<&TextStyle>
                    {
                        return Some(style.clone()); // Found the style
                    }
                }
            }

            // --- Strategy 2: Fallback to the style of the first TextRun ---
            // If list styles didn't provide the answer, find the first actual text run.
            if let Some(text_elements) = &text.text_elements {
                for element in text_elements {
                    if let Some(TextElementKind::TextRun(tr)) = &element.kind {
                        if let Some(style) = &tr.style {
                            // Found a styled TextRun, return its style as the default.
                            return Some(style.clone());
                        } // If the first TextRun has no style, we continue (unlikely for placeholders)
                    }
                }
            }
        }
    }
    // No shape, no text, or no styled elements/list styles found within the placeholder.
    None
}

/// Converts a single slide (`Page`) object into an SVG string representation.
/// Sets up the SVG canvas, background, and iterates through page elements for conversion.
/// Resolves the active `ColorScheme` based on slide/layout/master hierarchy.
///
/// # Arguments
/// * `slide` - Reference to the `Page` object representing the slide.
/// * `presentation_page_size` - Optional `Size` of the presentation canvas.
/// * `layouts_map` - Pre-built map of layout pages.
/// * `masters_map` - Pre-built map of master pages.
/// * `elements_map` - Pre-built map of all page elements.
///
/// # Returns
/// A `Result<String>` containing the SVG markup for the slide, or an error.
pub(crate) fn convert_slide_to_svg(
    slide: &Page,
    presentation_page_size: Option<&crate::models::common::Size>, // Use fully qualified path
    layouts_map: &LayoutsMap,
    masters_map: &MastersMap,
    elements_map: &ElementsMap,
) -> Result<String> {
    let mut svg_string = String::new();

    // Determine page dimensions in points
    let page_width_pt = dimension_to_pt(presentation_page_size.and_then(|s| s.width.as_ref()));
    let page_height_pt = dimension_to_pt(presentation_page_size.and_then(|s| s.height.as_ref()));

    if page_width_pt <= 0.0 || page_height_pt <= 0.0 {
        return Err(SvgConversionError::MissingData(
            "Invalid or missing presentation page size".to_string(),
        ));
    }

    // --- SVG Header ---
    writeln!(
        svg_string,
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{0}pt" height="{1}pt" viewBox="0 0 {0} {1}">"#,
        page_width_pt, page_height_pt
    )?;
    // Optional: Add a <defs> section if needed later (e.g., for markers, patterns, filters)
    // writeln!(svg_string, "  <defs></defs>")?;

    // --- Determine the Active Color Scheme ---
    // Hierarchy: Slide -> Layout -> Master -> Default (if none found)
    let mut active_color_scheme: Option<&ColorScheme> = None;

    // 1. Check Slide properties (most specific)
    if let Some(props) = &slide.page_properties {
        if props.color_scheme.is_some() {
            active_color_scheme = props.color_scheme.as_ref();
        }
    }

    // 2. If not on slide, check Layout properties
    let slide_layout_id = slide
        .slide_properties
        .as_ref()
        .and_then(|p| p.layout_object_id.as_ref());
    let mut master_id_from_layout: Option<&String> = None; // To store master ID if found via layout

    if active_color_scheme.is_none() {
        if let Some(layout_id) = slide_layout_id {
            if let Some(layout) = layouts_map.get(layout_id) {
                if let Some(props) = &layout.page_properties {
                    if props.color_scheme.is_some() {
                        active_color_scheme = props.color_scheme.as_ref();
                    }
                }
                // Store the master ID referenced by this layout for step 3
                master_id_from_layout = layout
                    .layout_properties
                    .as_ref()
                    .and_then(|lp| lp.master_object_id.as_ref());
            }
        }
    }

    // 3. If not on slide or layout, check Master properties
    if active_color_scheme.is_none() {
        // Use master linked directly from slide OR master linked via layout
        let master_id = slide
            .slide_properties
            .as_ref()
            .and_then(|p| p.master_object_id.as_ref())
            .or(master_id_from_layout); // Fallback to master from layout

        if let Some(id) = master_id {
            if let Some(master) = masters_map.get(id) {
                if let Some(props) = &master.page_properties {
                    if props.color_scheme.is_some() {
                        active_color_scheme = props.color_scheme.as_ref();
                    }
                }
            }
        }
    }

    // If still no scheme found after checking hierarchy, active_color_scheme remains None.
    // Functions using it should handle this (e.g., by using default colors).

    // --- Render Slide Background ---
    // Use the ColorScheme to find the background color, defaulting to white.
    // TODO: Handle complex backgrounds (gradients, images) defined in PageBackgroundFill.
    let background_fill_color = active_color_scheme
        .and_then(|cs| cs.resolve_theme_color(ThemeColorType::Background1)) // Use helper
        .unwrap_or_else(|| DEFAULT_BACKGROUND_COLOR.to_string());

    writeln!(
        svg_string,
        r#"  <rect width="100%" height="100%" fill="{}" />"#,
        background_fill_color
    )?;

    // --- Render Page Elements ---
    // Retrieve layout ID again, safely handling Option
    let layout_id_str = slide_layout_id.map(|id| id.as_str());

    if let Some(elements) = &slide.page_elements {
        // Create a mutable copy or list of references if sorting/filtering is needed.
        // Sorting by Y can approximate rendering order, but Z-order is complex.
        // Let's render in the order provided by the API for now.
        // let mut sorted_elements: Vec<&PageElement> = elements.iter().collect();
        // sorted_elements.sort_by(|a, b| /* Some Z-order comparison or Y-comparison */ );

        for element in elements {
            // Pass context (maps, layout ID) and the resolved color scheme to element conversion.
            convert_page_element_to_svg(
                element,
                layout_id_str, // Pass as Option<&str>
                layouts_map,
                masters_map,
                elements_map,
                active_color_scheme, // Pass the resolved scheme or None
                &mut svg_string,
            )?;
            writeln!(svg_string)?; // Add newline between elements for readability
        }
    }

    // --- SVG Footer ---
    writeln!(svg_string, "</svg>")?;
    Ok(svg_string)
}

// Helper function added to ColorScheme (consider moving to models/colors.rs if appropriate)
impl ColorScheme {
    /// Resolves a `ThemeColorType` to its corresponding RGB hex color string within this scheme.
    /// Returns `None` if the color type is not found in the scheme.
    fn resolve_theme_color(&self, theme_color_type: ThemeColorType) -> Option<String> {
        self.colors
            .iter()
            .find(|pair| pair.theme_color_type == theme_color_type)
            .map(|found_pair| {
                // Construct a temporary OpaqueColor to reuse the formatting logic
                let opaque_color = OpaqueColor {
                    color_kind: OpaqueColorContent::RgbColor(found_pair.color.clone()),
                };
                // Format this resolved color (pass None for scheme to avoid recursion)
                format_color(Some(&opaque_color), None)
            })
    }
}
