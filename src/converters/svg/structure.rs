//! Handles the overall structure of the presentation conversion,
//! including building lookup maps, resolving inheritance (placeholders),
//! and converting slides.

use log::{debug, warn};

use super::{
    constants::*,
    elements::convert_page_element_to_svg,
    error::{Result, SvgConversionError},
    utils::{dimension_to_pt, format_color, AsShape},
};
use crate::models::{
    bullet::Bullet,
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
/// Prioritizes the style defined in the associated list's nesting level 0 bulletStyle,
/// as this commonly holds the placeholder's default text attributes (font, size, color).
/// Falls back to the style of the first `TextRun` if the list style method fails.
///
/// # Arguments
/// * `placeholder_element` - The placeholder `PageElement` (likely a Shape) on the layout/master.
///
/// # Returns
/// An `Option<TextStyle>` containing the cloned default style, or `None` if no style could be found.
pub(crate) fn get_placeholder_default_text_style(
    placeholder_element: &PageElement,
) -> Option<TextStyle> {
    debug!(
        "[get_placeholder_default_text_style] Attempting for placeholder ID: {}",
        placeholder_element.object_id
    );

    if let Some(shape) = placeholder_element.element_kind.as_shape() {
        if let Some(text) = &shape.text {
            // --- Strategy 1 (Original Priority): Use List Style for Nesting Level 0 ---
            debug!(
                "[get_placeholder_default_text_style] Placeholder '{}': Trying List/Bullet style lookup.",
                 placeholder_element.object_id
             );
            let list_info: Option<(&String, i32)> =
                text.text_elements.as_ref().and_then(|elements| {
                    elements.iter().find_map(|element| {
                        if let Some(TextElementKind::ParagraphMarker(pm)) = &element.kind {
                            pm.bullet.as_ref().and_then(|b: &Bullet| {
                                b.list_id
                                    .as_ref()
                                    .map(|id| (id, b.nesting_level.unwrap_or(0)))
                            })
                        } else {
                            None
                        }
                    })
                });

            if let Some((list_id, _nesting_level)) = list_info {
                // nesting_level isn't directly used here, we hardcode lookup for 0
                debug!(
                    "[get_placeholder_default_text_style] Placeholder '{}': Found list_id '{}' from ParagraphMarker.",
                     placeholder_element.object_id, list_id
                 );
                if let Some(lists) = &text.lists {
                    if let Some(level_0_style) = lists
                        .get(list_id)
                        .and_then(|list_props| list_props.nesting_level.as_ref())
                        .and_then(|nesting_map| nesting_map.get(&0)) // Specifically level 0
                        .and_then(|level_0_props| level_0_props.bullet_style.as_ref())
                    {
                        debug!(
                            "[get_placeholder_default_text_style] Placeholder '{}': SUCCESS using list '{}', level 0: {:?}",
                             placeholder_element.object_id, list_id, level_0_style
                         );
                        return Some(level_0_style.clone()); // Return this style
                    } else {
                        debug!(
                            "[get_placeholder_default_text_style] Placeholder '{}': List '{}' found, but no style defined for level 0.",
                              placeholder_element.object_id, list_id
                         );
                    }
                }
            } else {
                debug!(
                    "[get_placeholder_default_text_style] Placeholder '{}': No ParagraphMarker with list info found.",
                      placeholder_element.object_id
                 );
            }

            // --- Strategy 2 (Fallback): Use the style of the first TextRun ---
            debug!(
                "[get_placeholder_default_text_style] Placeholder '{}': Falling back to first TextRun style lookup.",
                 placeholder_element.object_id
             );
            if let Some(text_elements) = &text.text_elements {
                if let Some(first_tr_style) =
                    text_elements
                        .iter()
                        .find_map(|element| match &element.kind {
                            Some(TextElementKind::TextRun(tr)) => tr.style.as_ref(),
                            _ => None,
                        })
                {
                    debug!(
                        "[get_placeholder_default_text_style] Placeholder '{}': SUCCESS using fallback TextRun style: {:?}",
                         placeholder_element.object_id, first_tr_style
                     );
                    return Some(first_tr_style.clone());
                } else {
                    debug!(
                        "[get_placeholder_default_text_style] Placeholder '{}': No styled TextRun found for fallback.",
                         placeholder_element.object_id
                     );
                }
            }
        }
    }

    warn!( // Keep as warn if no style is found at all
        "[get_placeholder_default_text_style] No default text style could be determined for placeholder '{}'.",
         placeholder_element.object_id
     );
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
