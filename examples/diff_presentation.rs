use std::fs;
use std::io; // For io::Error
use std::path::Path;

// Use the new SVG diffing module
use gslides_tools::diff::svg_diff::compare_svg_content;

/// Helper function to load SVG file content.
fn load_svg_content(file_path: &str) -> Result<String, io::Error> {
    let path = Path::new(file_path);
    fs::read_to_string(path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Google Slides SVG Diff Markdown Report Example ---");

    // --- Configuration ---
    // Paths are relative to the gslides-tools crate root if running with `cargo run --example <name>`
    // If running the compiled binary directly from `target/debug/examples`, adjust paths accordingly or use absolute paths.
    let base_svg_file_path = "base_slide_1.svg";
    let changed_svg_file_path = "diff_slide_1.svg";

    // --- Load SVG Content ---
    println!("Loading base SVG from: {}", base_svg_file_path);
    let base_svg_content = load_svg_content(base_svg_file_path)?;

    println!("Loading SVG to compare from: {}", changed_svg_file_path);
    let changed_svg_content = load_svg_content(changed_svg_file_path)?;

    // --- Perform Comparison ---
    println!("Comparing SVG files...");
    let result = compare_svg_content(
        &base_svg_content,
        &changed_svg_content,
        base_svg_file_path,    // Filename for diff header
        changed_svg_file_path, // Filename for diff header
    );

    // --- Output Results ---
    println!("\n-- SVG Diff Report --\n");
    println!("{}", result.markdown_report);

    Ok(())
}
