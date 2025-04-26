use similar::{ChangeTag, TextDiff};
use std::fmt::Write; // Use Write for building the output string

/// Generates a diff string in the unified format between two text inputs.
/// Includes a simple summary of lines added/removed.
///
/// # Arguments
/// * `base_text` - The "original" text content (e.g., from base_presentation.md).
/// * `changed_text` - The "new" text content (e.g., from changed_presentation.md).
/// * `base_filename` - The name to use for the original file in the diff header (e.g., "a/presentation.md").
/// * `changed_filename` - The name to use for the new file in the diff header (e.g., "b/presentation.md").
///
/// # Returns
/// A `String` containing the summary and the unified diff.
pub fn generate_markdown_diff(
    base_text: &str,
    changed_text: &str,
    base_filename: &str,
    changed_filename: &str,
) -> String {
    let diff = TextDiff::from_lines(base_text, changed_text);
    let mut output = String::new();
    let mut added_lines = 0;
    let mut removed_lines = 0;

    // --- Generate Unified Diff ---
    // Use unified_diff() for standard formatting
    write!(
        output,
        "{}",
        diff.unified_diff().header(base_filename, changed_filename) // Sets --- a/... and +++ b/...
                                                                    // .context_radius(3) // Optional: Number of context lines around changes
    )
    .expect("Failed to write unified diff to string"); // Writing to String shouldn't fail

    // --- Calculate Summary (Iterate over changes) ---
    // We iterate separately to count lines accurately for the summary,
    // as unified_diff doesn't return counts directly.
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => added_lines += 1,
            ChangeTag::Delete => removed_lines += 1,
            ChangeTag::Equal => (), // Do nothing for equal lines
        }
    }

    // --- Prepend Summary ---
    let summary = format!(
        "## Summary of Changes (Text Content)\n\n- Lines Added: {}\n- Lines Removed: {}\n\n---\n\n",
        added_lines, removed_lines
    );

    summary + &output // Prepend summary to the diff output
}

// --- Test Runner Function (can be moved to main.rs or tests later) ---
#[allow(clippy::expect_fun_call)]
#[cfg(test)]
mod tests {
    use super::*;

    use crate::{markdown::extract_text_from_presentation, models::presentation::Presentation};
    use std::{fs, io::Write as IoWrite};

    #[test]
    fn test_markdown_diff_generation() {
        // 1. Load Presentations
        let base_json_path = "base_presentation.json";
        let changed_json_path = "changed_presentation.json";

        let base_json_string = fs::read_to_string(base_json_path)
            .expect(&format!("Failed to read {}", base_json_path));
        let changed_json_string = fs::read_to_string(changed_json_path)
            .expect(&format!("Failed to read {}", changed_json_path));

        let base_presentation: Presentation = serde_json::from_str(&base_json_string)
            .expect(&format!("Failed to deserialize {}", base_json_path));
        let changed_presentation: Presentation = serde_json::from_str(&changed_json_string)
            .expect(&format!("Failed to deserialize {}", changed_json_path));

        // 2. Extract Markdown Text
        println!("Extracting Markdown from base presentation...");
        std::io::stdout().flush().unwrap();
        let base_md = extract_text_from_presentation(&base_presentation);
        fs::write("base_extracted.md", &base_md).expect("Failed to write base_extracted.md");
        println!("Base Markdown extracted and saved to base_extracted.md");

        println!("Extracting Markdown from changed presentation...");
        std::io::stdout().flush().unwrap();
        let changed_md = extract_text_from_presentation(&changed_presentation);
        fs::write("changed_extracted.md", &changed_md)
            .expect("Failed to write changed_extracted.md");
        println!("Changed Markdown extracted and saved to changed_extracted.md");

        // 3. Generate Diff
        println!("Generating Markdown diff...");
        std::io::stdout().flush().unwrap();
        let diff_output = generate_markdown_diff(
            &base_md,
            &changed_md,
            "a/presentation.md", // Standard naming for git diffs
            "b/presentation.md",
        );

        // 4. Output Diff
        println!("\n--- Generated Diff ---");
        println!("{}", diff_output);
        println!("--- End Generated Diff ---");

        // 5. Save Diff to File
        let diff_file_path = "presentation_diff.diff";
        let error_message = format!("Failed to write diff to {}", diff_file_path);
        fs::write(diff_file_path, &diff_output).expect(&error_message);
        println!("Diff saved to {}", diff_file_path);

        // 6. Assertions (Basic)
        assert!(diff_output.contains("## Summary of Changes"));
        assert!(diff_output.contains("--- a/presentation.md"));
        assert!(diff_output.contains("+++ b/presentation.md"));
        assert!(diff_output.contains("@@")); // Check for hunk headers

        // Add more specific assertions based on expected changes between your files
        // e.g., assert!(diff_output.contains("+New Text Added"));
        // e.g., assert!(diff_output.contains("-Old Text Removed"));
    }
}
