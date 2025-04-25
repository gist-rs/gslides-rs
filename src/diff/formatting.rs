use crate::{
    diff::structured::{Change, ChangeType, ValueRepr},
    Presentation,
};
use similar::{ChangeTag, TextDiff};
use std::fmt::Write;

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

    // Iterate through changes and format them in unified diff format
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        // 3 lines of context
        // Write hunk header (simplified, could be improved to show line numbers)
        if idx > 0 {
            writeln!(output, "@@ ... @@")?; // Placeholder hunk header
        } else {
            writeln!(output, "@@ @@")?; // Initial hunk header
        }

        for op in group {
            for change in diff.iter_inline_changes(op) {
                // Use iter_inline_changes
                let sign = match change.tag() {
                    ChangeTag::Delete => '-',
                    ChangeTag::Insert => '+',
                    ChangeTag::Equal => ' ',
                };
                // Write each line with its sign (+, -, or space for context)
                // Handle newline characters correctly within the changed text
                for &(emphasized, value) in change.values() {
                    if emphasized {
                        // Highlight changes within the line if needed (optional)
                        write!(output, "{}{}", sign, value)?;
                    } else {
                        write!(output, "{}{}", sign, value)?;
                    }
                }
                // Ensure lines end with a newline if the original did
                if change.missing_newline() {
                    writeln!(output)?; // Add newline if missing
                }
            }
        }
    }

    // If no changes were found, indicate that
    if structured_changes.is_empty() {
        output.push_str("\nNo changes detected.\n");
    }

    Ok(output)
}

/// Generates a human-readable summary string from the structured changes.
/// Based on Section 7 of the design document.
pub(crate) fn generate_readable_summary(changes: &[Change]) -> Result<String, DiffError> {
    let mut summary_lines: Vec<String> = Vec::new();
    let mut counts = (0, 0, 0); // (added, removed, modified)

    for change in changes {
        match change.change_type {
            ChangeType::Added => counts.0 += 1,
            ChangeType::Removed => counts.1 += 1,
            ChangeType::Modified => counts.2 += 1,
        }

        // Step (a): Map path to a more friendly name (basic implementation)
        let mapped_path = map_path_to_friendly_name(&change.path);

        let line = match change.change_type {
            ChangeType::Added => {
                // Step (c): Summarize list additions (basic)
                let type_info = get_type_from_value_repr(&change.new_value);
                format!("- Added {} at `{}`", type_info, mapped_path)
            }
            ChangeType::Removed => {
                // Step (c): Summarize list removals (basic)
                let type_info = get_type_from_value_repr(&change.old_value);
                format!("- Removed {} from `{}`", type_info, mapped_path)
            }
            ChangeType::Modified => {
                // Step (b) & (d): Format simple changes or generic modification
                if let (Some(old), Some(new)) = (&change.old_value, &change.new_value) {
                    // Check for simple value changes
                    match (old, new) {
                        (ValueRepr::String(_), ValueRepr::String(_))
                        | (ValueRepr::Number(_), ValueRepr::Number(_))
                        | (ValueRepr::Boolean(_), ValueRepr::Boolean(_))
                        | (ValueRepr::Null, _)
                        | (_, ValueRepr::Null) => {
                            format!(
                                "- Changed `{}` from {} to {}",
                                mapped_path,
                                old.format_for_display(),
                                new.format_for_display()
                            )
                        }
                        // Fallback for complex modifications (Object/Array summaries)
                        _ => format!(
                            "- Modified `{}` ({} -> {})",
                            mapped_path,
                            old.format_for_display(),
                            new.format_for_display()
                        ),
                    }
                } else {
                    // Should not happen for Modified type, but handle defensively
                    format!("- Modified `{}` (incomplete data)", mapped_path)
                }
            }
        };
        summary_lines.push(line);
    }

    // Step (e): Aggregation
    let total = counts.0 + counts.1 + counts.2;
    let header = format!(
        "## Summary:\nDetected {} changes: {} additions, {} removals, {} modifications.",
        total, counts.0, counts.1, counts.2
    );

    let mut final_summary = header;
    if !summary_lines.is_empty() {
        final_summary.push_str("\n\n## Details:\n");
        final_summary.push_str(&summary_lines.join("\n"));
    }

    Ok(final_summary)
}

/// Basic helper to make paths slightly more readable.
/// A production version would need a much more sophisticated mapping.
fn map_path_to_friendly_name(path: &str) -> String {
    path.replace(".pageElements", ".Element")
        .replace(".textElements", ".Text")
        .replace(".shape.text", ".Shape Text")
        .replace(".shape.shapeProperties", ".Shape Properties")
        .replace(".textRun.content", ".Content")
        .replace(".textRun.style", ".Style")
        .replace(".style.foregroundColor", ".Color")
        .replace(".style.fontSize", ".Font Size")
        .replace(".style.bold", ".Bold")
        // Add more replacements as needed
        .trim_start_matches('.') // Remove leading dot if present
        .to_string()
}

/// Basic helper to infer type information from ValueRepr for summaries.
fn get_type_from_value_repr(value: &Option<ValueRepr>) -> String {
    match value {
        Some(ValueRepr::Object(_)) => "Object".to_string(),
        Some(ValueRepr::Array(_)) => "Item in Array".to_string(),
        Some(ValueRepr::String(_)) => "Text".to_string(),
        Some(ValueRepr::Number(_)) => "Number".to_string(),
        Some(ValueRepr::Boolean(_)) => "Boolean".to_string(),
        Some(ValueRepr::Null) => "Null value".to_string(),
        None => "Item".to_string(), // Fallback if value is None
    }
}
