use std::fs;

use gslides_tools::{converters::svg::convert_presentation_to_svg, Presentation};
// No need to explicitly 'use log;' here unless this file uses log macros itself

fn main() {
    // Initialize the logger
    // This reads the RUST_LOG environment variable to configure logging levels
    // It's good practice to set a default level (e.g., Info) in case RUST_LOG isn't set
    env_logger::builder()
        .filter_level(log::LevelFilter::Info) // Set default level
        .parse_default_env() // Allow RUST_LOG override
        .init();

    // Load a sample presentation JSON
    log::info!("Loading presentation from JSON..."); // Use log::info instead of println!
    let json_path = "changed_presentation.json";
    // let json_path = "converted_presentation.json";
    let json_string =
        fs::read_to_string(json_path).expect("Should have been able to read the file");
    let presentation: Presentation =
        serde_json::from_str(&json_string).expect("Failed to deserialize presentation JSON");
    log::info!("Presentation loaded successfully.");

    // Convert to SVG
    log::info!("Starting SVG conversion...");
    let svg_results = convert_presentation_to_svg(&presentation);
    log::info!("SVG conversion finished.");

    match svg_results {
        Ok(svg_vec) => {
            log::info!("SVG conversion successful, got {} slides.", svg_vec.len());
            assert!(
                !svg_vec.is_empty(),
                "SVG conversion should produce output for slides."
            );

            // Optionally save each SVG to a file for inspection
            for (i, svg_content) in svg_vec.iter().enumerate() {
                let output_path = format!("test_slide_{}.svg", i + 1);
                log::debug!(
                    "Attempting to write SVG for slide {} to {}",
                    i + 1,
                    output_path
                ); // Use log::debug for verbose actions
                let err_msg = format!("Unable to write SVG file: {}", output_path);
                fs::write(&output_path, svg_content).expect(&err_msg);
                log::info!("SVG for slide {} saved to {}", i + 1, output_path); // Use log::info for success messages

                // Basic checks on SVG content
                assert!(svg_content.starts_with("<svg"));
                // Adjust check if your SVG doesn't end with newline
                assert!(svg_content.ends_with("</svg>") || svg_content.ends_with("</svg>\n"));
                assert!(svg_content.contains("xmlns=\"http://www.w3.org/2000/svg\""));
            }
            log::info!("All SVG files processed.");
        }
        Err(e) => {
            log::error!("SVG Conversion failed: {}", e); // Use log::error for failures
            panic!("SVG Conversion failed: {}", e); // Keep panic if you want execution to stop
        }
    }
}
