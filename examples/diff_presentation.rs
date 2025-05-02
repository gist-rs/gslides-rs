use std::fs;
use std::path::Path;

use gslides_tools::diff::error::DiffError;
use gslides_tools::{ComparerBuilder, Presentation};

/// Helper function to load and parse a Presentation JSON file.
fn load_presentation(file_path: &str) -> Result<Presentation, DiffError> {
    let path = Path::new(file_path);
    let file_content = fs::read_to_string(path)?;
    // Use DiffError::Serialization for JSON parsing errors
    let presentation: Presentation = serde_json::from_str(&file_content)?;
    Ok(presentation)
}

fn main() -> Result<(), DiffError> {
    println!("--- Google Slides Diff Example ---");

    // --- Configuration ---
    let base_file_path = "base_presentation.json"; // Path to your basebase JSON
                                                   // let compare_file_path = "output.json"; // Path to the JSON to compare
    let compare_file_path = "changed_presentation.json"; // Path to the JSON to compare

    // --- Load Presentations ---
    println!("Loading base presentation from: {}", base_file_path);
    let base_presentation = load_presentation(base_file_path)?;

    println!(
        "Loading presentation to compare from: {}",
        compare_file_path
    );
    let compare_presentation = load_presentation(compare_file_path)?;

    // --- Setup Comparer ---
    println!("Setting up comparer with base presentation...");
    let comparer = ComparerBuilder::new()
        .set_base(base_presentation)
        .set_simplify(true)
        .build()?;

    // --- Perform Comparison ---
    println!("Comparing presentations...");
    let result = comparer.compare(&compare_presentation)?;

    // --- Output Results ---

    // 1. Structured Diff (Print paths of changes)
    println!("\n--- Structured Diff (Paths) ---");
    let structured_diff = result.get_structured_diff();
    if structured_diff.is_empty() {
        println!("No differences found.");
    } else {
        println!("structured_diff {:#?}:", structured_diff);
        println!("Found {} differences:", structured_diff.len());
        for change in structured_diff {
            println!(" - Path: {}, Type: {:?}", change.path, change.change_type);
            // Optionally print old/new values (can be verbose)
            // if let Some(old) = &change.old_value { println!("   Old: {}", old.format_for_display()); }
            // if let Some(new) = &change.new_value { println!("   New: {}", new.format_for_display()); }
        }
    }

    // 2. Git-Style Diff
    println!("\n--- Git-Style Diff ---");
    match result.get_git_diff() {
        Ok(git_diff) => println!("{}", git_diff),
        Err(e) => eprintln!("Error generating Git diff: {}", e),
    }

    // 3. Readable Summary
    println!("\n--- Readable Summary ---");
    match result.get_readable_diff() {
        Ok(summary) => println!("{}", summary),
        Err(e) => eprintln!("Error generating readable summary: {}", e),
    }

    println!("\n--- Comparison Finished ---");

    Ok(())
}
