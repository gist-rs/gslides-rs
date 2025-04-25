use super::error::DiffError;
use crate::{
    diff::structured::{Change, ChangeType, ValueRepr},
    Presentation,
};
use similar::{ChangeTag, TextDiff};
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Write,
};

/// Generates a Git-style diff string from the structured changes.
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

/// Extracts the parent path of a color component (up to .rgbColor)
fn extract_color_component_parent_path(path: &str) -> Option<String> {
    if let Some(last_dot) = path.rfind('.') {
        let component = &path[last_dot + 1..];
        if component == "red" || component == "green" || component == "blue" {
            // Ensure the part before the component ends with ".rgbColor"
            let parent_path = &path[..last_dot];
            if parent_path.ends_with(".rgbColor") {
                // Find the start of ".rgbColor"
                if let Some(rgb_color_start) = parent_path.rfind(".rgbColor") {
                    // Return path up to and including ".rgbColor"
                    return Some(parent_path[..rgb_color_start + 9].to_string());
                }
            }
        }
    }
    None
}

/// Generates a human-readable summary string from the structured changes, grouped by slide.
/// Handles color component Add/Remove pairs. Filters based on allowlist.
pub(crate) fn generate_readable_summary(
    changes: &[Change],
    is_simplify: bool,
) -> Result<String, DiffError> {
    // --- Allowlist: MUST include "Modified Color" and specific properties like "Font Family" ---
    const ALLOWED_DESCRIPTIONS: &[&str] = &[
        "PresentationId",
        "RevisionId",
        "Title",
        "Text Content",
        "Modified Color", // This is the key for consolidated color changes
        "Font Family",    // Allow specific property
                          // Add other specific descriptions returned by describe_change_target
    ];

    println!("--- DEBUG: Starting generate_readable_summary ---");
    println!("--- DEBUG: Allowlist: {:?}", ALLOWED_DESCRIPTIONS);
    println!("--- DEBUG: Raw changes received ({}):", changes.len());
    for (idx, change) in changes.iter().enumerate() {
        println!(
            "  [{}] Path: '{}', Type: {:?}, Old: {:?}, New: {:?}",
            idx, change.path, change.change_type, change.old_value, change.new_value
        );
    }
    println!("--- DEBUG: End Raw changes ---");

    let mut changes_by_slide: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    let mut general_changes: Vec<String> = Vec::new();
    let mut summarized_counts = (0, 0, 0); // (add, remove, modify) *after filtering*

    let mut processed_indices: HashSet<usize> = HashSet::new();
    // Track parent *color* paths for which "Modified Color" has been generated per group
    let mut processed_color_parents_general: HashSet<String> = HashSet::new();
    let mut processed_color_parents_slide: BTreeMap<usize, HashSet<String>> = BTreeMap::new();

    for i in 0..changes.len() {
        if processed_indices.contains(&i) {
            // println!("--- DEBUG: Skipping index {} (already processed)", i); // Can be noisy
            continue;
        }

        let change = &changes[i];
        let (slide_index_opt, remaining_path) = parse_slide_path(&change.path)
            .map_or((None, change.path.clone()), |(idx, rp)| (Some(idx), rp));

        let mut handled_by_consolidation = false;
        let mut generated_line: Option<String> = None;
        let mut generated_desc: Option<String> = None; // The description *before* potential consolidation

        // --- Get description early for checks ---
        let change_target_desc = describe_change_target(&remaining_path);
        println!(
            "--- DEBUG: Index {}: Path '{}', Desc: '{}'",
            i, change.path, change_target_desc
        );

        // --- 1. Attempt Color Consolidation (Add/Remove component pairs -> Modified Color) ---
        // Use the specific description check first
        if change_target_desc == "Color Component" {
            if let Some(color_parent_path) = extract_color_component_parent_path(&change.path) {
                println!(
                    "--- DEBUG: Index {}: Is Color Component, Parent: '{}'",
                    i, color_parent_path
                );
                let current_change_type = change.change_type.clone(); // Clone needed for borrow checker later? Maybe not.
                if current_change_type == ChangeType::Added
                    || current_change_type == ChangeType::Removed
                {
                    // Look ahead for the opposite change type for the same parent
                    for j in (i + 1)..changes.len() {
                        if processed_indices.contains(&j) {
                            continue;
                        }

                        let next_change = &changes[j];
                        // Get description for next change to ensure it's also a color component
                        let (_next_slide_opt, next_remaining_path) =
                            parse_slide_path(&next_change.path)
                                .map_or((None, next_change.path.clone()), |(idx, rp)| {
                                    (Some(idx), rp)
                                });
                        let next_change_target_desc = describe_change_target(&next_remaining_path);

                        println!(
                            "--- DEBUG: Index {}: Lookahead index {} Desc: '{}'",
                            i, j, next_change_target_desc
                        );

                        if next_change_target_desc == "Color Component" {
                            if let Some(next_color_parent_path) =
                                extract_color_component_parent_path(&next_change.path)
                            {
                                if next_color_parent_path == color_parent_path {
                                    println!("--- DEBUG: Index {}: Lookahead index {} is Color Component for same parent '{}'", i, j, color_parent_path);
                                    // Check for Add/Remove pair
                                    if (current_change_type == ChangeType::Added
                                        && next_change.change_type == ChangeType::Removed)
                                        || (current_change_type == ChangeType::Removed
                                            && next_change.change_type == ChangeType::Added)
                                    {
                                        // --- Pair Found ---
                                        println!("--- DEBUG: Index {}: Confirmed Add/Remove pair at index {}", i, j);
                                        let processed_parents_ref = match slide_index_opt {
                                            Some(idx) => processed_color_parents_slide
                                                .entry(idx)
                                                .or_default(),
                                            None => &mut processed_color_parents_general,
                                        };
                                        if !processed_parents_ref.contains(&color_parent_path) {
                                            let friendly_location =
                                                map_path_to_friendly_name(&remaining_path);
                                            let consolidated_desc = "Modified Color".to_string(); // Use the consolidated description
                                                                                                  // Retrieve old/new values from the original Add/Remove changes to show detail
                                            let (old_val_repr, new_val_repr) =
                                                if current_change_type == ChangeType::Removed {
                                                    (&change.old_value, &next_change.new_value)
                                                // Removed old, Added new
                                                } else {
                                                    (&next_change.old_value, &change.new_value)
                                                    // Added new, Removed old
                                                };
                                            let old_str = old_val_repr
                                                .as_ref()
                                                .map_or("?".to_string(), |v| {
                                                    v.format_for_display()
                                                });
                                            let new_str = new_val_repr
                                                .as_ref()
                                                .map_or("?".to_string(), |v| {
                                                    v.format_for_display()
                                                });

                                            // *** FORMAT WITH DETAIL *** (We don't have full color, just components)
                                            // Let's stick to the simpler "Modified Color" for now, detail requires more state.
                                            let line = format!(
                                                "- {} {}",
                                                consolidated_desc,
                                                format_location(&friendly_location, is_simplify)
                                            );

                                            generated_line = Some(line);
                                            generated_desc = Some(consolidated_desc); // Store the *consolidated* desc
                                            processed_parents_ref.insert(color_parent_path.clone());
                                            println!("--- DEBUG: Index {}: Stored consolidated 'Modified Color' line for parent '{}'", i, color_parent_path);
                                        } else {
                                            println!("--- DEBUG: Index {}: Parent '{}' already had 'Modified Color' reported. Skipping generation.", i, color_parent_path);
                                        }
                                        processed_indices.insert(i);
                                        processed_indices.insert(j);
                                        handled_by_consolidation = true;
                                        break; // Stop lookahead
                                    }
                                } else {
                                    println!("--- DEBUG: Index {}: Lookahead index {} has different color parent. Stopping.", i, j);
                                    break;
                                } // Different parent
                            } else {
                                println!("--- DEBUG: Index {}: Lookahead index {} failed color parent extraction. Stopping.", i, j);
                                break;
                            } // Not color component path (shouldn't happen if desc matched)
                        } else {
                            println!("--- DEBUG: Index {}: Lookahead index {} is not Color Component (Desc: '{}'). Stopping.", i, j, next_change_target_desc);
                            break;
                        } // Not color component description
                    } // End lookahead loop
                } else {
                    println!("--- DEBUG: Index {}: Is Color Component but type is {:?}, not Add/Remove. No pair search.", i, current_change_type);
                }
            } else {
                println!("--- DEBUG: Index {}: Desc is Color Component, but failed parent extraction for path '{}'.", i, change.path);
            }
        } // End Color Consolidation Check

        // --- 2. Handle Non-Consolidated Changes (or failed consolidation) ---
        if !handled_by_consolidation {
            // Check if index was somehow processed by lookahead but didn't set handled_by_consolidation flag (shouldn't happen)
            if processed_indices.contains(&i) {
                println!(
                    "--- WARNING: Index {} was processed but not handled by consolidation?",
                    i
                );
                continue;
            }
            processed_indices.insert(i); // Mark as processed here

            let friendly_path = map_path_to_friendly_name(&remaining_path);
            // Use the description we got earlier
            let desc = change_target_desc.clone(); // Use the specific description

            // Generate line based on original change type
            let line = match change.change_type {
                ChangeType::Added => format!(
                    "- Added {} {}",
                    desc,
                    format_location(&friendly_path, is_simplify)
                ),
                ChangeType::Removed => format!(
                    "- Removed {} {}",
                    desc,
                    format_location(&friendly_path, is_simplify)
                ),
                ChangeType::Modified => {
                    if let (Some(old), Some(new)) = (&change.old_value, &change.new_value) {
                        // Use specific desc like "Font Family" here
                        format!(
                            "- Changed {} from {} to {} {}",
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
            generated_desc = Some(desc); // Store the *specific* desc for filtering
        }

        // --- 3. Filter and Add Line ---
        if let (Some(line), Some(desc)) = (generated_line, generated_desc) {
            println!(
                "--- DEBUG: Index {}: Checking filter for Desc: '{}'",
                i, desc
            );
            if ALLOWED_DESCRIPTIONS.contains(&desc.as_str()) {
                // Increment count based on *effective* change type (consolidation -> Modify)
                if desc == "Modified Color" {
                    summarized_counts.2 += 1; // Count consolidation as Modify
                } else {
                    match change.change_type {
                        // Base count on original type for non-consolidated
                        ChangeType::Added => summarized_counts.0 += 1,
                        ChangeType::Removed => summarized_counts.1 += 1,
                        ChangeType::Modified => summarized_counts.2 += 1,
                    }
                }
                // Add to the appropriate group
                match slide_index_opt {
                    Some(idx) => changes_by_slide.entry(idx).or_default().push(line.clone()),
                    None => general_changes.push(line.clone()),
                };
                println!(
                    "--- DEBUG: Adding ALLOWED line (Desc: '{}'): '{}'",
                    desc, line
                );
            } else {
                println!(
                    "--- DEBUG: Filtering out line (Desc: '{}'): '{}'",
                    desc, line
                );
            }
        } else if handled_by_consolidation {
            // This means consolidation happened, but we decided not to generate a line (e.g., duplicate parent)
            println!("--- DEBUG: Index {}: Handled by consolidation but no line generated (likely duplicate parent processing).", i);
        } else {
            // This means no line was generated and it wasn't explicitly handled by consolidation. Might be an issue.
            println!("--- WARNING: Index {}: No line generated and not handled by consolidation. Desc was '{}'", i, change_target_desc);
        }
    } // End main loop

    // --- Final Assembly ---
    println!("--- DEBUG: Finished processing loop ---");
    println!(
        "--- DEBUG: Summarized Counts (after filter): Add={}, Remove={}, Modify={}",
        summarized_counts.0, summarized_counts.1, summarized_counts.2
    );
    println!(
        "--- DEBUG: General Changes Collected: {}",
        general_changes.len()
    );
    for (idx, lines) in &changes_by_slide {
        println!(
            "--- DEBUG: Slide {} Changes Collected: {}",
            idx + 1,
            lines.len()
        );
    }

    let final_total = summarized_counts.0 + summarized_counts.1 + summarized_counts.2;
    let mut final_summary = format!(
        "## Summary:\nDetected {} relevant changes: {} additions, {} removals, {} modifications.", // Changed wording
        final_total, summarized_counts.0, summarized_counts.1, summarized_counts.2
    );

    if !changes_by_slide.is_empty() || !general_changes.is_empty() {
        final_summary.push_str("\n\n## Details:");
    }
    // ... (rest of summary string assembly remains the same) ...
    if !general_changes.is_empty() {
        final_summary.push_str("\n\n### General Changes:\n");
        final_summary.push_str(&general_changes.join("\n"));
    }
    for (slide_index, slide_lines) in &changes_by_slide {
        if slide_lines.is_empty() {
            continue;
        }
        write!(final_summary, "\n\n### Slide {}:\n", slide_index + 1)?;
        final_summary.push_str(&slide_lines.join("\n"));
    }
    if final_total > 0
        && changes_by_slide.values().all(|v| v.is_empty())
        && general_changes.is_empty()
    {
        final_summary
            .push_str("\n\nNote: Changes were detected but filtered out from the summary.");
    } else if final_total == 0 {
        final_summary.push_str("\n\nNo relevant changes detected.") // Changed message
    }

    println!("--- DEBUG: Final Summary Generated ---");
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
    // Match most specific paths first
    if remaining_path.ends_with(".textRun.content") {
        "Text Content".to_string()

    // Style properties
    } else if remaining_path.ends_with(".style.bold") {
        "Bold Style".to_string()
    } else if remaining_path.ends_with(".style.italic") {
        "Italic Style".to_string()
    } else if remaining_path.ends_with(".style.underline") {
        "Underline Style".to_string()
    } else if remaining_path.ends_with(".style.strikethrough") {
        "Strikethrough Style".to_string()
    } else if remaining_path.ends_with(".style.smallCaps") {
        "Small Caps Style".to_string()
    } else if remaining_path.ends_with(".style.backgroundColor") {
        "Text Background Color".to_string()
    } else if remaining_path.ends_with(".style.foregroundColor") {
        "Text Foreground Color".to_string()
    } else if remaining_path.ends_with(".style.link") {
        "Text Link".to_string()
    } else if remaining_path.ends_with(".style.baselineOffset") {
        "Baseline Offset".to_string()
    } else if remaining_path.ends_with(".style.fontFamily") {
        "Font Family".to_string() // Specific description
    } else if remaining_path.ends_with(".style.fontSize.magnitude") {
        "Font Size".to_string() // Specific description
    } else if remaining_path.ends_with(".style.weightedFontFamily.fontFamily") {
        "Rendered Font Family".to_string()
    } else if remaining_path.ends_with(".style.weightedFontFamily.weight") {
        "Rendered Font Weight".to_string()

    // Color Components
    } else if remaining_path.ends_with(".opaqueColor.rgbColor.red")
        || remaining_path.ends_with(".opaqueColor.rgbColor.green")
        || remaining_path.ends_with(".opaqueColor.rgbColor.blue")
    {
        "Color Component".to_string() // Describe the component specifically
    } else if remaining_path.ends_with(".solidFill.color.opaqueColor.rgbColor.red")
        || remaining_path.ends_with(".solidFill.color.opaqueColor.rgbColor.green")
        || remaining_path.ends_with(".solidFill.color.opaqueColor.rgbColor.blue")
    {
        "Color Component".to_string() // Describe the component specifically

    // Shape Properties
    } else if remaining_path.ends_with(".shapeProperties.shapeBackgroundFill") {
        // Complex obj
        "Shape Fill".to_string()
    } else if remaining_path.ends_with(".shapeProperties.outline") {
        // Complex obj
        "Shape Outline".to_string()
    } else if remaining_path.ends_with(".shapeProperties.shadow") {
        // Complex obj
        "Shape Shadow".to_string()
    } else if remaining_path.ends_with(".shapeProperties.link") {
        // Complex obj
        "Shape Link".to_string()
    } else if remaining_path.ends_with(".shapeProperties.contentAlignment") {
        "Shape Content Alignment".to_string()
    } else if remaining_path.ends_with(".shapeProperties.autofit") {
        // Complex obj
        "Shape Autofit".to_string()

    // Image Properties
    } else if remaining_path.ends_with(".imageProperties.brightness") {
        "Image Brightness".to_string()
    } else if remaining_path.ends_with(".imageProperties.contrast") {
        "Image Contrast".to_string()
    } else if remaining_path.ends_with(".imageProperties.transparency") {
        "Image Transparency".to_string()
    } else if remaining_path.ends_with(".imageProperties.cropProperties") {
        // Complex obj
        "Image Crop".to_string()
    } else if remaining_path.ends_with(".imageProperties.outline") {
        // Complex obj
        "Image Outline".to_string()
    } else if remaining_path.ends_with(".imageProperties.shadow") {
        // Complex obj
        "Image Shadow".to_string()
    } else if remaining_path.ends_with(".imageProperties.link") {
        // Complex obj
        "Image Link".to_string()
    } else if remaining_path.ends_with(".imageProperties.recolor") {
        // Complex obj
        "Image Recolor".to_string()

    // Generic fallbacks
    } else if remaining_path.contains(".shapeProperties.") {
        "Shape Property".to_string()
    } else if remaining_path.contains(".imageProperties.") {
        "Image Property".to_string()
    } else if remaining_path.contains(".textRun.style.") {
        "Text Style Property".to_string()
    } else if remaining_path.contains(".paragraphMarker.style.") {
        "Paragraph Style Property".to_string()
    } else if remaining_path.contains(".element.") {
        "Element Property".to_string()
    } else if remaining_path.contains(".text.") {
        // Generic text property fallback
        "Text Property".to_string()
    } else if remaining_path.is_empty() {
        "Item".to_string() // Generic term for change to element/slide itself
    } else {
        // Fallback: Use the last part of the path if possible
        remaining_path
            .split(|c| c == '.' || c == '[') // Split by '.' or '['
            .filter(|s| !s.is_empty() && *s != "]") // Filter out empty parts and ']'
            .last()
            .map(|s| {
                // Basic capitalization for readability
                let mut chars = s.chars();
                match chars.next() {
                    None => "Property".to_string(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .unwrap_or_else(|| "Property".to_string())
    }
}

/// Basic helper to make paths slightly more readable.
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
