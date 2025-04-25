use crate::{
    diff::structured::{Change, ChangeType, ValueRepr},
    Presentation,
};
use similar::{ChangeTag, TextDiff};
use std::{collections::BTreeMap, fmt::Write};

use super::error::DiffError; // Import Write trait for write! macro

/// Generates a Git-style diff string from the structured changes.
///
/// This version diffs the *entire* serialized presentations for simplicity.
/// A more advanced version would retrieve and diff context snippets based on paths.
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

    // If no changes, write only the header and exit early
    if structured_changes.is_empty() && old_str == new_str {
        // Check structured_changes AND actual content diff
        output.push_str("\nNo changes detected.\n");
        return Ok(output);
    }

    let mut header_written = false;
    // Iterate through changes and format them in unified diff format
    for group in diff.grouped_ops(3) {
        // 3 lines of context
        // Calculate hunk header info (basic version)
        // A real implementation needs careful tracking of line numbers
        let mut old_line = 0;
        let mut new_line = 0;
        if let Some(first_op) = group.first() {
            old_line = first_op.old_range().start + 1; // 1-based line numbers
            new_line = first_op.new_range().start + 1;
        }
        // Explicitly type the sum result as usize
        let old_len: usize = group.iter().map(|op| op.old_range().len()).sum();
        let new_len: usize = group.iter().map(|op| op.new_range().len()).sum();

        writeln!(
            output,
            "@@ -{},{} +{},{} @@",
            old_line, old_len, new_line, new_len
        )?; // Basic hunk header
        header_written = true; // Mark that we have written at least one hunk header

        for op in group {
            // Pass op by reference to iter_inline_changes
            for change in diff.iter_inline_changes(&op) {
                let sign = match change.tag() {
                    ChangeTag::Delete => '-',
                    ChangeTag::Insert => '+',
                    ChangeTag::Equal => ' ',
                };
                for &(_emphasized, value) in change.values() {
                    // Simple emphasis: just write the value
                    // More complex emphasis could involve ANSI codes etc.
                    write!(output, "{}{}", sign, value)?;
                }
                // Ensure lines end with a newline if the original did NOT have one explicitly removed/added
                // similar already handles line endings within `value` based on `from_lines`
                if change.missing_newline() {
                    // This indicates the line itself didn't end with \n in the original input
                    // For unified diff, we don't typically add an extra newline here unless it's
                    // specifically the "\ No newline at end of file" marker, which similar doesn't directly provide easily.
                    // Let's omit adding an extra one here for now.
                }
            }
        }
    }

    // If headers were written but structured_changes is empty (e.g. whitespace change only detected by similar)
    // Or if structured changes exist but similar didn't produce ops (less likely but possible)
    // We rely on the initial check for perfect equality. If not equal, but no hunks, it's weird.
    if !header_written && !structured_changes.is_empty() {
        output.push_str("\nNote: Structured changes detected, but text diff generated no output (potentially whitespace differences only).\n");
    } else if !header_written && old_str != new_str {
        // Fallback if similar somehow produced no ops despite strings differing
        output.push_str("\nNote: Presentations differ, but text diff generated no output.\n");
    }

    Ok(output)
}

/// Attempts to parse the slide index and the remaining path from a full path string.
/// e.g., "slides[1].pageElements[0].text..." -> Some((1, "pageElements[0].text..."))
/// Returns None if the path doesn't start with "slides[index]".
fn parse_slide_path(path: &str) -> Option<(usize, String)> {
    if path.starts_with("slides[") {
        if let Some(end_bracket_pos) = path.find(']') {
            let index_str = &path[7..end_bracket_pos];
            if let Ok(index) = index_str.parse::<usize>() {
                // Check if there's content after the bracket
                let rest_start_pos = end_bracket_pos + 1;
                let remaining_path = if rest_start_pos < path.len() {
                    // Check if the next character is a dot and skip it
                    if path.chars().nth(rest_start_pos) == Some('.') {
                        path[rest_start_pos + 1..].to_string()
                    } else {
                        path[rest_start_pos..].to_string() // Should include brackets like pageElements[0]
                    }
                } else {
                    String::new() // Path ended exactly at slides[index]
                };
                return Some((index, remaining_path));
            }
        }
    }
    None
}

/// Generates a human-readable summary string from the structured changes, grouped by slide.
pub(crate) fn generate_readable_summary(
    changes: &[Change],
    is_simplify: bool,
) -> Result<String, DiffError> {
    // Group changes by slide index. Use BTreeMap for sorted keys (slide order).
    let mut changes_by_slide: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut general_changes: Vec<String> = Vec::new();
    let mut counts = (0, 0, 0); // (added, removed, modified)

    for change in changes {
        counts = match change.change_type {
            ChangeType::Added => (counts.0 + 1, counts.1, counts.2),
            ChangeType::Removed => (counts.0, counts.1 + 1, counts.2),
            ChangeType::Modified => (counts.0, counts.1, counts.2 + 1),
        };

        let line: String;
        if let Some((slide_index, remaining_path)) = parse_slide_path(&change.path) {
            // This change belongs to a specific slide
            let friendly_element_path = map_path_to_friendly_name(&remaining_path);
            let change_target_desc = describe_change_target(&remaining_path);

            line = match change.change_type {
                ChangeType::Added => format!(
                    "- Added {} {}",
                    change_target_desc,
                    format_location(&friendly_element_path, is_simplify)
                ),
                ChangeType::Removed => format!(
                    "- Removed {} {}",
                    change_target_desc,
                    format_location(&friendly_element_path, is_simplify)
                ),
                ChangeType::Modified => {
                    if let (Some(old), Some(new)) = (&change.old_value, &change.new_value) {
                        format!(
                            "- Changed {} from {} to {} {}",
                            change_target_desc,
                            old.format_for_display(),
                            new.format_for_display(),
                            format_location(&friendly_element_path, is_simplify)
                        )
                    } else {
                        format!(
                            "- Modified {} {} (incomplete data)",
                            change_target_desc,
                            format_location(&friendly_element_path, is_simplify)
                        )
                    }
                }
            };
            changes_by_slide.entry(slide_index).or_default().push(line);
        } else {
            // This is a general change
            let mapped_path = map_path_to_friendly_name(&change.path);
            line = match change.change_type {
                ChangeType::Added => format!(
                    "- Added {} at `{}`",
                    get_type_from_value_repr(&change.new_value),
                    mapped_path
                ),
                ChangeType::Removed => format!(
                    "- Removed {} from `{}`",
                    get_type_from_value_repr(&change.old_value),
                    mapped_path
                ),
                ChangeType::Modified => {
                    if let (Some(old), Some(new)) = (&change.old_value, &change.new_value) {
                        format!(
                            "- Changed `{}` from {} to {}",
                            mapped_path,
                            old.format_for_display(),
                            new.format_for_display()
                        )
                    } else {
                        format!("- Modified `{}` (incomplete data)", mapped_path)
                    }
                }
            };
            general_changes.push(line);
        }
    }

    // --- Assemble the final summary string ---

    let total = counts.0 + counts.1 + counts.2;
    let mut final_summary = format!(
        "## Summary:\nDetected {} changes: {} additions, {} removals, {} modifications.",
        total, counts.0, counts.1, counts.2
    );

    if !changes_by_slide.is_empty() || !general_changes.is_empty() {
        final_summary.push_str("\n\n## Details:");
    }

    // Append general changes first
    if !general_changes.is_empty() {
        final_summary.push_str("\n### General Changes:\n");
        final_summary.push_str(&general_changes.join("\n"));
    }

    // Append changes grouped by slide
    // ***** CHANGE HERE *****
    // Iterate over references to avoid moving the map
    for (slide_index, slide_lines) in &changes_by_slide {
        // Dereference slide_index since it's now &usize
        write!(final_summary, "\n### Slide {}:\n", *slide_index + 1)?;
        // slide_lines is &Vec<String>, join works correctly on it
        final_summary.push_str(&slide_lines.join("\n"));
    }

    // Handle case where there were changes but neither general nor slide-specific groups were populated
    // ***** THIS CHECK IS NOW VALID *****
    if total > 0 && changes_by_slide.is_empty() && general_changes.is_empty() {
        final_summary.push_str("\n\nNote: Changes were detected but could not be categorized by slide or general properties.");
    } else if total == 0 {
        final_summary.push_str("\n\nNo changes detected.") // Add confirmation if summary shows 0
    }

    Ok(final_summary)
}

/// Formats the element path for display within the summary line.
fn format_location(friendly_element_path: &str, is_simplify: bool) -> String {
    if friendly_element_path.is_empty() || is_simplify {
        // Change affects the slide object itself directly (less common for pageElements)
        "".to_string()
    } else {
        format!("(at `{}`)", friendly_element_path)
    }
}

/// Tries to describe *what* changed based on the suffix of the remaining path.
fn describe_change_target(remaining_path: &str) -> String {
    // Match more specific paths first
    if remaining_path.ends_with(".textRun.content") {
        "Text Content".to_string()
    } else if remaining_path.ends_with(".style.bold") {
        "Bold Style".to_string()
    } else if remaining_path.ends_with(".style.italic") {
        "Italic Style".to_string()
    } else if remaining_path.ends_with(".style.underline") {
        "Underline Style".to_string()
    } else if remaining_path.ends_with(".style.strikethrough") {
        "Strikethrough Style".to_string()
    } else if remaining_path.ends_with(".style.fontSize.magnitude") {
        "Font Size".to_string()
    } else if remaining_path.ends_with(".style.foregroundColor.opaqueColor.rgbColor") {
        // This is quite specific, maybe just ".foregroundColor"?
        "Text Color".to_string()
    } else if remaining_path.ends_with(
        ".shape.shapeProperties.shapeBackgroundFill.solidFill.color.opaqueColor.rgbColor",
    ) {
        "Shape Fill Color".to_string()
    } else if remaining_path
        .ends_with(".shape.shapeProperties.outline.solidFill.color.opaqueColor.rgbColor")
    {
        "Shape Outline Color".to_string()
    } else if remaining_path.ends_with(".shape.shapeProperties.outline.weight.magnitude") {
        "Shape Outline Weight".to_string()
    } else if remaining_path.contains(".shape.") {
        // General shape property
        "Shape Property".to_string()
    } else if remaining_path.contains(".imageProperties.") {
        "Image Property".to_string()
    } else if remaining_path.contains(".element.") {
        // General element property
        "Element Property".to_string()
    } else if remaining_path.contains(".text.") {
        // General text property
        "Text Property".to_string()
    } else if remaining_path.is_empty() {
        "Slide Properties".to_string() // e.g. added/removed slide itself if path was just slides[N]
    } else {
        // Fallback: Use the last part of the path if possible
        remaining_path
            .split('.')
            .last()
            .unwrap_or("Property")
            .to_string()
    }
}

/// Basic helper to make paths slightly more readable.
/// Now operates on the remaining path *within* a slide or a general path.
fn map_path_to_friendly_name(path: &str) -> String {
    // Keep replacements simple for element paths
    path.replace("pageElements", "Element") // Note: Index [N] will remain
        .replace("textElements", "Text") // Note: Index [N] will remain
        .replace(".shape.text", ".ShapeText") // Make it one word? Or keep space? Let's try no space.
        .replace(".shape.shapeProperties", ".ShapeProps")
        .replace(".textRun.content", ".Content")
        .replace(".textRun.style", ".Style")
        .replace(".style.foregroundColor", ".Color")
        .replace(".style.fontSize", ".FontSize")
        .replace(".style.bold", ".Bold")
        // Remove redundant structure if possible - might be too aggressive
        // .replace(".opaqueColor.rgbColor", "") // Example: Might simplify color paths too much
        .trim_start_matches('.') // Remove leading dot if present
        .to_string()
}

/// Basic helper to infer type information from ValueRepr for summaries.
/// (Used primarily for general changes where specific target isn't parsed)
fn get_type_from_value_repr(value: &Option<ValueRepr>) -> String {
    match value {
        Some(ValueRepr::Object(_)) => "Object data".to_string(), // Slightly more descriptive
        Some(ValueRepr::Array(_)) => "Item in list".to_string(), // Slightly more descriptive
        Some(ValueRepr::String(_)) => "Text value".to_string(),
        Some(ValueRepr::Number(_)) => "Number value".to_string(),
        Some(ValueRepr::Boolean(_)) => "Boolean value".to_string(),
        Some(ValueRepr::Null) => "Null value".to_string(),
        None => "Item".to_string(), // Fallback if value is None (e.g., for Removed)
    }
}
