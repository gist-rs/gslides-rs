use super::error::DiffError;
use crate::{
    diff::structured::{Change, ChangeType, ValueRepr},
    models::colors::RgbColor,
    // models::text_element::TextElementKind, // No longer needed for lookup
    Presentation,
};
use serde_json::Value as JsonValue;
use similar::{ChangeTag, TextDiff};
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Write,
};

//=============================================================================
// Git-Style Diff Generation
//=============================================================================

/// Generates a Git-style diff string from the structured changes.
/// Diffs the entire serialized presentations.
pub(crate) fn generate_git_diff(
    old_presentation: &Presentation,
    new_presentation: &Presentation,
    structured_changes: &[Change], // Keep changes for potential future context use
) -> Result<String, DiffError> {
    // Serialize both presentations to pretty JSON strings
    let old_str = serde_json::to_string_pretty(old_presentation)?;
    let new_str = serde_json::to_string_pretty(new_presentation)?;

    // Use `similar` to generate the diff
    let diff = TextDiff::from_lines(&old_str, &new_str);

    let mut output = String::new();
    // Write the diff header (simplified)
    writeln!(output, "--- a/presentation.json")?;
    writeln!(output, "+++ b/presentation.json")?;

    // If no changes detected by treediff AND strings are identical, exit early
    if structured_changes.is_empty() && old_str == new_str {
        output.push_str("\nNo changes detected.\n");
        return Ok(output);
    }

    let mut header_written = false;
    // Iterate through changes and format them in unified diff format
    for group in diff.grouped_ops(3) {
        // 3 lines of context
        // Calculate hunk header info
        let mut old_line = 0;
        let mut new_line = 0;
        if let Some(first_op) = group.first() {
            old_line = first_op.old_range().start + 1; // 1-based line numbers
            new_line = first_op.new_range().start + 1;
        }
        let old_len: usize = group.iter().map(|op| op.old_range().len()).sum();
        let new_len: usize = group.iter().map(|op| op.new_range().len()).sum();

        writeln!(
            output,
            "@@ -{},{} +{},{} @@",
            old_line, old_len, new_line, new_len
        )?;
        header_written = true; // Mark that we have written at least one hunk header

        // Format lines within the hunk
        for op in group {
            for change in diff.iter_inline_changes(&op) {
                let sign = match change.tag() {
                    ChangeTag::Delete => '-',
                    ChangeTag::Insert => '+',
                    ChangeTag::Equal => ' ',
                };
                for &(_emphasized, value) in change.values() {
                    write!(output, "{}{}", sign, value)?;
                }
                if change.missing_newline() {
                    // Handle missing newline markers if necessary
                }
            }
        }
    }

    // Handle edge cases where diff generation might seem inconsistent
    if !header_written && !structured_changes.is_empty() {
        output.push_str("\nNote: Structured changes detected, but text diff generated no output (potentially minor structural or whitespace differences only).\n");
        output.push_str(&format!(
            "\nStructured changes found: {}\n",
            structured_changes.len()
        ));
    } else if !header_written && old_str != new_str {
        output.push_str(
            "\nNote: Presentations differ textually, but diff algorithm generated no output.\n",
        );
    }

    Ok(output)
}

//=============================================================================
// Human-Readable Summary Generation - Helpers
//=============================================================================

/// Attempts to parse the slide index and the remaining path from a full path string.
fn parse_slide_path(path: &str) -> Option<(usize, String)> {
    if path.starts_with("slides[") {
        if let Some(end_bracket_pos) = path.find(']') {
            let index_str = &path[7..end_bracket_pos];
            if let Ok(index) = index_str.parse::<usize>() {
                let rest_start_pos = end_bracket_pos + 1;
                let remaining_path = if rest_start_pos < path.len() {
                    if path.chars().nth(rest_start_pos) == Some('.') {
                        path[rest_start_pos + 1..].to_string()
                    } else {
                        path[rest_start_pos..].to_string()
                    }
                } else {
                    String::new()
                };
                return Some((index, remaining_path));
            }
        }
    }
    None
}

/// Basic helper to make paths slightly more readable for context.
fn map_path_to_friendly_name(path: &str) -> String {
    path.replace("pageElements", "Element")
        .replace("textElements", "TextElement")
        .replace(".elementKind.shape.text", ".ShapeText")
        .replace(".shape.shapeProperties", ".ShapeProps")
        .replace(".textRun.content", ".Content")
        .replace(".textRun.style", ".Style")
        .replace(".foregroundColor.opaqueColor.rgbColor", ".Color(FG)")
        .replace(".backgroundColor.opaqueColor.rgbColor", ".Color(BG)")
        .replace(".solidFill.color.opaqueColor.rgbColor", ".Color(Fill)")
        .replace(".style.fontFamily", ".Font")
        .replace(".style.fontSize.magnitude", ".Size")
        .replace(".style.bold", ".Bold")
        .trim_start_matches('.')
        .to_string()
}

/// Formats the location part of a summary line.
fn format_location(friendly_element_path: &str, is_simplify: bool) -> String {
    if friendly_element_path.is_empty() || is_simplify {
        "".to_string()
    } else {
        format!("(at `{}`)", friendly_element_path)
    }
}

/// Tries to determine the type of change based on the path suffix.
fn describe_change_target(remaining_path: &str) -> String {
    // --- HIGHEST PRIORITY: Direct changes to text content ---
    if remaining_path.ends_with(".textRun.content") {
        return "Text Content".to_string();
    }

    // --- Consolidation Hooks ---
    // Check for the exact path where color component Add/Remove events might occur
    if remaining_path.ends_with(".foregroundColor.opaqueColor.rgbColor")
        || remaining_path.ends_with(".backgroundColor.opaqueColor.rgbColor")
        || remaining_path.ends_with(".solidFill.color.opaqueColor.rgbColor")
    {
        return "Color Object Components Changed".to_string();
    }
    // Removed style consolidation hook

    // --- Other Specific Properties ---
    else if remaining_path.ends_with(".style.fontFamily") {
        return "Font Family".to_string();
    } else if remaining_path.ends_with(".style.fontSize.magnitude") {
        return "Font Size".to_string();
    } else if remaining_path.ends_with(".style.bold") {
        return "Bold Style".to_string();
    } else if remaining_path.ends_with(".style.italic") {
        return "Italic Style".to_string();
    } else if remaining_path.ends_with(".style.underline") {
        return "Underline Style".to_string();
    } else if remaining_path.ends_with(".style.strikethrough") {
        return "Strikethrough Style".to_string();
    } else if remaining_path.ends_with(".style.smallCaps") {
        return "Small Caps Style".to_string();
    } else if remaining_path.ends_with(".style.backgroundColor") {
        return "Text Background Color".to_string();
    } else if remaining_path.ends_with(".style.foregroundColor") {
        return "Text Foreground Color".to_string();
    } else if remaining_path.ends_with(".style.link") {
        return "Text Link".to_string();
    } else if remaining_path.ends_with(".style.baselineOffset") {
        return "Baseline Offset".to_string();
    } else if remaining_path.ends_with(".shapeProperties.autofit") {
        return "Shape Autofit".to_string();
    } else if remaining_path.ends_with(".style.weightedFontFamily.fontFamily") {
        return "Rendered Font Family".to_string();
    } else if remaining_path.ends_with(".style.weightedFontFamily.weight") {
        return "Rendered Font Weight".to_string();
    }
    // --- Fallbacks ---
    else if remaining_path.contains(".shapeProperties.") {
        return "Shape Property".to_string();
    } else if remaining_path.contains(".imageProperties.") {
        return "Image Property".to_string();
    } else if remaining_path.contains(".textRun.style.") {
        return "Text Style Property".to_string();
    } else if remaining_path.contains(".paragraphMarker.style.") {
        return "Paragraph Style Property".to_string();
    }
    // Generic check for changes within textElements if not content/style - will likely be filtered
    else if remaining_path.contains(".textElements") {
        return "Text Property".to_string();
    } else if remaining_path.contains(".element.") {
        return "Element Property".to_string();
    } else if remaining_path.is_empty() {
        return "Item".to_string();
    } else {
        // Final fallback based on last path segment
        remaining_path
            .rsplit(|c| c == '.' || c == '[')
            .next()
            .map_or_else(
                || "Property".to_string(),
                |segment| {
                    if segment.is_empty() || segment == "]" {
                        "Property".to_string()
                    } else {
                        let mut chars = segment.chars();
                        match chars.next() {
                            None => "Property".to_string(),
                            Some(first_char) => {
                                first_char.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    }
                },
            )
    }
}

/// Helper to traverse serde_json::Value using a simplified path string.
fn get_value_at_path<'a>(root: &'a JsonValue, path_str: &str) -> Option<&'a JsonValue> {
    let mut current = root;
    let mut remaining_path = path_str;

    // Handle initial segment (if path doesn't start with . or [)
    let initial_split = remaining_path
        .find(|c| c == '.' || c == '[')
        .unwrap_or(remaining_path.len());
    if initial_split > 0 {
        let (segment, rest) = remaining_path.split_at(initial_split);
        current = current.get(segment)?;
        remaining_path = rest;
    }

    // Handle subsequent segments
    while !remaining_path.is_empty() {
        if remaining_path.starts_with('.') {
            remaining_path = &remaining_path[1..]; // Skip '.'
            let next_split = remaining_path
                .find(|c| c == '.' || c == '[')
                .unwrap_or(remaining_path.len());
            let (key, rest) = remaining_path.split_at(next_split);
            if key.is_empty() {
                return None;
            }
            current = current.get(key)?;
            remaining_path = rest;
        } else if remaining_path.starts_with('[') {
            if let Some(end_bracket_pos) = remaining_path.find(']') {
                let index_str = &remaining_path[1..end_bracket_pos];
                if let Ok(index) = index_str.parse::<usize>() {
                    current = current.get(index)?;
                    remaining_path = &remaining_path[end_bracket_pos + 1..];
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    Some(current)
}

/// Helper to format RgbColor to Hex string.
fn format_rgb_to_hex(rgb: &RgbColor) -> String {
    let r = (rgb.red.unwrap_or(0.0) * 255.0).round() as u8;
    let g = (rgb.green.unwrap_or(0.0) * 255.0).round() as u8;
    let b = (rgb.blue.unwrap_or(0.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

//=============================================================================
// Human-Readable Summary Generation - Main Function
//=============================================================================

/// Generates a human-readable summary string from the structured changes, grouped by slide.
/// Retrieves color details from full presentation objects. Filters based on allowlist.
pub(crate) fn generate_readable_summary(
    old_presentation: &Presentation,
    new_presentation: &Presentation,
    changes: &[Change],
    is_simplify: bool,
) -> Result<String, DiffError> {
    // Convert presentations to serde_json::Value for traversal (once)
    let old_val = serde_json::to_value(old_presentation)?;
    let new_val = serde_json::to_value(new_presentation)?;

    // Define which change descriptions should appear in the final summary
    const ALLOWED_DESCRIPTIONS: &[&str] = &[
        "PresentationId",
        "RevisionId",
        "Title",          // General
        "Text Content",   // Content (Covers Add/Remove/Modify)
        "Modified Color", // Consolidated Color result
        "Font Family",    // Specific Style property
                          // Add other specific descriptions like "Font Size", "Bold Style" if desired
    ];

    // --- Debug Setup ---
    // println!("--- DEBUG: Starting generate_readable_summary ---");
    // println!("--- DEBUG: Allowlist: {:?}", ALLOWED_DESCRIPTIONS);
    // println!("--- DEBUG: Raw changes received ({}):", changes.len());
    // for (idx, change) in changes.iter().enumerate() {
    //      println!("  [{}] Path: '{}', Type: {:?}, Old: {:?}, New: {:?}",
    //               idx, change.path, change.change_type, change.old_value, change.new_value);
    // }
    // println!("--- DEBUG: End Raw changes ---");

    let mut changes_by_slide: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut general_changes: Vec<String> = Vec::new();
    let mut summarized_counts = (0, 0, 0); // (add, remove, modify) *after filtering*

    let mut processed_indices: HashSet<usize> = HashSet::new();
    // Track paths where color Add/Remove pairs have been consolidated per group
    let mut consolidated_color_paths_general: HashSet<String> = HashSet::new();
    let mut consolidated_color_paths_slide: BTreeMap<usize, HashSet<String>> = BTreeMap::new();

    // No longer need added_object_tracker

    for i in 0..changes.len() {
        if processed_indices.contains(&i) {
            continue;
        }

        let change = &changes[i];
        let (slide_index_opt, remaining_path) = parse_slide_path(&change.path)
            .map_or((None, change.path.clone()), |(idx, rp)| (Some(idx), rp));

        let mut handled_by_consolidation = false;
        let mut generated_line: Option<String> = None;
        let mut generated_desc: Option<String> = None; // Final description used for filtering

        // Determine description based on path
        let change_target_desc = describe_change_target(&remaining_path);
        // println!("--- DEBUG: Index {}: Path '{}', Initial Desc: '{}'", i, change.path, change_target_desc);

        // --- 1. Consolidate Color Add/Remove pairs -> "Modified Color" ---
        if change_target_desc == "Color Object Components Changed" {
            let current_change_type = change.change_type.clone();
            if current_change_type == ChangeType::Added
                || current_change_type == ChangeType::Removed
            {
                // Look ahead for the opposite change type *at the exact same path*
                for j in (i + 1)..changes.len() {
                    if processed_indices.contains(&j) {
                        continue;
                    }
                    let next_change = &changes[j];

                    if next_change.path == change.path {
                        let (_next_slide_opt, next_remaining_path) =
                            parse_slide_path(&next_change.path)
                                .map_or((None, next_change.path.clone()), |(_, rp)| (Some(0), rp));
                        let next_change_target_desc = describe_change_target(&next_remaining_path);
                        // Ensure the lookahead item is also identified as the color trigger
                        if next_change_target_desc == "Color Object Components Changed" {
                            // Check for Add vs Remove pairing
                            if (current_change_type == ChangeType::Added
                                && next_change.change_type == ChangeType::Removed)
                                || (current_change_type == ChangeType::Removed
                                    && next_change.change_type == ChangeType::Added)
                            {
                                // --- Pair Found ---
                                let processed_paths_ref = match slide_index_opt {
                                    Some(idx) => {
                                        consolidated_color_paths_slide.entry(idx).or_default()
                                    }
                                    None => &mut consolidated_color_paths_general,
                                };
                                // Use the exact change path for tracking consolidation uniqueness
                                if !processed_paths_ref.contains(&change.path) {
                                    let friendly_location =
                                        map_path_to_friendly_name(&remaining_path);
                                    let consolidated_desc = "Modified Color".to_string(); // This is the key for filtering

                                    // Retrieve color details by looking up the full object at the path
                                    let (old_hex, new_hex) = {
                                        let default_hex = "?".to_string();
                                        if let (Some(old_color_val), Some(new_color_val)) = (
                                            get_value_at_path(&old_val, &change.path),
                                            get_value_at_path(&new_val, &change.path),
                                        ) {
                                            match (
                                                serde_json::from_value::<RgbColor>(
                                                    old_color_val.clone(),
                                                ),
                                                serde_json::from_value::<RgbColor>(
                                                    new_color_val.clone(),
                                                ),
                                            ) {
                                                (Ok(old_c), Ok(new_c)) => (
                                                    format_rgb_to_hex(&old_c),
                                                    format_rgb_to_hex(&new_c),
                                                ),
                                                _ => (default_hex.clone(), default_hex.clone()),
                                            }
                                        } else {
                                            (default_hex.clone(), default_hex.clone())
                                        }
                                    };

                                    // Format the line, including details if available
                                    let line = if old_hex != "?" && new_hex != "?" {
                                        format!(
                                            "- {} from `{}` to `{}` {}",
                                            consolidated_desc,
                                            old_hex,
                                            new_hex,
                                            format_location(&friendly_location, is_simplify)
                                        )
                                    } else {
                                        format!(
                                            "- {} {}",
                                            consolidated_desc,
                                            format_location(&friendly_location, is_simplify)
                                        ) // Fallback
                                    };

                                    generated_line = Some(line);
                                    generated_desc = Some(consolidated_desc); // Store the *consolidated* description
                                    processed_paths_ref.insert(change.path.clone());
                                // Mark this path as consolidated for this group
                                // println!("--- DEBUG: Index {}: Stored consolidated 'Modified Color' line for path '{}'", i, change.path);
                                } else {
                                    // Path already consolidated for this group, skip generating another line
                                    // println!("--- DEBUG: Index {}: Path '{}' already consolidated. Skipping generation.", i, change.path);
                                }
                                // Mark both original Add/Remove events as processed
                                processed_indices.insert(i);
                                processed_indices.insert(j);
                                handled_by_consolidation = true;
                                break; // Found pair, stop lookahead for change 'i'
                            }
                        } else {
                            /* Desc mismatch */
                            break;
                        }
                    } else {
                        /* Path mismatch */
                        break;
                    }
                } // End lookahead loop
            }
        } // End Color Consolidation

        // --- 2. Handle Non-Consolidated Changes ---
        if !handled_by_consolidation {
            // Ensure index wasn't processed during a failed lookahead or other logic
            if processed_indices.contains(&i) {
                continue;
            }
            processed_indices.insert(i); // Mark as processed now

            let friendly_path = map_path_to_friendly_name(&remaining_path);
            // Use the description determined by describe_change_target
            let desc = change_target_desc;

            // Format the line based on change type
            let line = match change.change_type {
                ChangeType::Added => {
                    let val_str = change
                        .new_value
                        .as_ref()
                        .map_or_else(|| "?".to_string(), |v| v.format_for_display());
                    // Special format for Text Content additions
                    if desc == "Text Content" {
                        format!(
                            "- Added Text Content `{}` {}",
                            val_str,
                            format_location(&friendly_path, is_simplify)
                        )
                    } else {
                        // Generic Add format for other allowed types
                        format!(
                            "- Added {} `{}` {}",
                            desc,
                            val_str,
                            format_location(&friendly_path, is_simplify)
                        )
                    }
                }
                ChangeType::Removed => {
                    let val_str = change
                        .old_value
                        .as_ref()
                        .map_or_else(|| "?".to_string(), |v| v.format_for_display());
                    // Special format for Text Content removals
                    if desc == "Text Content" {
                        format!(
                            "- Removed Text Content `{}` {}",
                            val_str,
                            format_location(&friendly_path, is_simplify)
                        )
                    } else {
                        // Generic Remove format
                        format!(
                            "- Removed {} `{}` {}",
                            desc,
                            val_str,
                            format_location(&friendly_path, is_simplify)
                        )
                    }
                }
                ChangeType::Modified => {
                    if let (Some(old), Some(new)) = (&change.old_value, &change.new_value) {
                        format!(
                            "- Changed {} from `{}` to `{}` {}",
                            desc,
                            old.format_for_display(),
                            new.format_for_display(),
                            format_location(&friendly_path, is_simplify)
                        )
                    } else {
                        format!(
                            "- Modified {} {} (incomplete data)",
                            desc,
                            format_location(&friendly_path, is_simplify)
                        )
                    }
                }
            };
            generated_line = Some(line);
            generated_desc = Some(desc); // Store the description used for this line
        }

        // --- 3. Filter and Add Line ---
        if let (Some(line), Some(desc)) = (generated_line, generated_desc) {
            // Check the final description against the allowlist
            // println!("--- DEBUG: Index {}: Checking filter for Desc: '{}'", i, desc);
            if ALLOWED_DESCRIPTIONS.contains(&desc.as_str()) {
                // Increment the correct count based on the *effective* change type
                if desc == "Modified Color" {
                    // Consolidated changes count as Modify
                    summarized_counts.2 += 1;
                } else {
                    // Use the original change type for non-consolidated items
                    match change.change_type {
                        ChangeType::Added => summarized_counts.0 += 1,
                        ChangeType::Removed => summarized_counts.1 += 1,
                        ChangeType::Modified => summarized_counts.2 += 1,
                    }
                }
                // Add the formatted line to the appropriate group (slide or general)
                match slide_index_opt {
                    Some(idx) => changes_by_slide.entry(idx).or_default().push(line.clone()),
                    None => general_changes.push(line.clone()),
                };
                // println!("--- DEBUG: Adding ALLOWED line (Desc: '{}'): '{}'", desc, line);
            } else {
                // Description was not in the allowlist, filter it out
                // println!("--- DEBUG: Filtering out line (Desc: '{}'): '{}'", desc, line);
            }
        } else if !handled_by_consolidation {
            // This case indicates no line was generated for a change that wasn't consolidated.
            // Could happen if Added Text Content lookup failed before.
            // println!("--- WARNING: Index {}: No line generated and not consolidated. Desc was '{}'", i, change_target_desc);
        }
    } // End main loop processing changes

    // --- Final Summary Assembly ---
    // println!("--- DEBUG: Finished processing loop ---");
    // println!("--- DEBUG: Summarized Counts (after filter): Add={}, Remove={}, Modify={}", summarized_counts.0, summarized_counts.1, summarized_counts.2);
    // ... other final debug prints ...

    let final_total = summarized_counts.0 + summarized_counts.1 + summarized_counts.2;
    let mut final_summary = format!(
        "## Summary:\nDetected {} relevant changes: {} additions, {} removals, {} modifications.",
        final_total, summarized_counts.0, summarized_counts.1, summarized_counts.2
    );

    if !changes_by_slide.is_empty() || !general_changes.is_empty() {
        final_summary.push_str("\n\n## Details:");
    }

    // Append General Changes
    if !general_changes.is_empty() {
        final_summary.push_str("\n\n### General Changes:\n");
        final_summary.push_str(&general_changes.join("\n"));
    }

    // Append Slide Changes
    for (slide_index, slide_lines) in &changes_by_slide {
        if slide_lines.is_empty() {
            continue;
        } // Skip slides with no relevant changes
          // Use write! macro which returns a Result, handle potential error
        write!(final_summary, "\n\n### Slide {}:\n", slide_index + 1)?;
        final_summary.push_str(&slide_lines.join("\n"));
    }

    // Footer notes
    if final_total > 0
        && changes_by_slide.values().all(|v| v.is_empty())
        && general_changes.is_empty()
    {
        final_summary
            .push_str("\n\nNote: Changes were detected but filtered out by relevance settings.");
    } else if final_total == 0 {
        final_summary.push_str("\n\nNo relevant changes detected.")
    }

    // println!("--- DEBUG: Final Summary Generated ---");
    Ok(final_summary)
}
