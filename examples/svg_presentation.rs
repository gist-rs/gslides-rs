use std::fs;

use gslides_rs::{converters::svg::convert_presentation_to_svg, Presentation};

fn main() {
    // Load a sample presentation JSON
    let json_path = "changed_presentation.json";
    // let json_path = "converted_presentation.json";
    let json_string =
        fs::read_to_string(json_path).expect("Should have been able to read the file");
    let presentation: Presentation =
        serde_json::from_str(&json_string).expect("Failed to deserialize presentation JSON");

    // Convert to SVG
    let svg_results = convert_presentation_to_svg(&presentation);

    match svg_results {
        Ok(svg_vec) => {
            assert!(
                !svg_vec.is_empty(),
                "SVG conversion should produce output for slides."
            );

            // Optionally save each SVG to a file for inspection
            for (i, svg_content) in svg_vec.iter().enumerate() {
                let output_path = format!("test_slide_{}.svg", i + 1);
                let err_msg = format!("Unable to write SVG file: {}", output_path);
                fs::write(&output_path, svg_content).expect(&err_msg);
                println!("SVG for slide {} saved to {}", i + 1, output_path);

                // Basic checks on SVG content
                assert!(svg_content.starts_with("<svg"));
                assert!(svg_content.ends_with("</svg>\n")); // Check for closing tag and newline
                assert!(svg_content.contains("xmlns=\"http://www.w3.org/2000/svg\""));
            }
        }
        Err(e) => {
            panic!("SVG Conversion failed: {}", e);
        }
    }
}
