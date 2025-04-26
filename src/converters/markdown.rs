use crate::models::{
    common::AffineTransform,
    elements::{PageElement, PageElementKind},
    page::Page,
    presentation::Presentation,
    shape::{Shape, ShapeType},
    table::Table, // Added for Table support
    text::TextContent,
    text_element::{TextElement as ModelTextElement, TextElementKind as ModelTextElementKind},
};
use std::cmp::Ordering;
use std::fmt::Write; // Import Write trait for formatting

// --- Helper Function for Sorting ---
// (get_translate_y and compare_elements_by_y remain the same)
/// Extracts the translateY value from a PageElement's transform.
/// Returns f64::MAX if transform or translateY is None, placing such elements last.
fn get_translate_y(element: &PageElement) -> f64 {
    element
        .transform
        .as_ref()
        .and_then(|t: &AffineTransform| t.translate_y)
        .unwrap_or(f64::MAX) // Default to max value to sort elements without Y coord last
}

/// Compares two PageElements based on their vertical position (translateY).
fn compare_elements_by_y(a: &PageElement, b: &PageElement) -> Ordering {
    get_translate_y(a)
        .partial_cmp(&get_translate_y(b))
        .unwrap_or(Ordering::Equal) // Fallback if comparison fails (e.g., NaN)
}

// --- Text Extraction Logic ---
// (extract_text_from_text_run, extract_text_from_text_content,
//  extract_text_from_shape, extract_text_from_table,
//  extract_text_from_page_element remain the same)
/// Extracts text content from a single TextElement (specifically TextRun).
fn extract_text_from_text_run(text_element: &ModelTextElement) -> Option<String> {
    if let Some(ModelTextElementKind::TextRun(text_run)) = &text_element.kind {
        text_run.content.clone() // Clone the content string if it exists
    } else {
        None // Not a TextRun or no content
    }
}

/// Extracts text content from a TextContent block (iterates through TextElements).
fn extract_text_from_text_content(text_content: &TextContent) -> String {
    let mut combined_text = String::new();
    if let Some(elements) = &text_content.text_elements {
        for element in elements {
            if let Some(text) = extract_text_from_text_run(element) {
                combined_text.push_str(&text);
            }
        }
    }
    // Ensure consistent newline handling at the end of a content block
    // If the original text ended with \n, keep it. If not, don't add one here.
    combined_text
}

/// Extracts text from a Shape element, specifically if it's a TEXT_BOX.
fn extract_text_from_shape(shape: &Shape) -> Option<String> {
    // Only extract from shapes explicitly marked as TEXT_BOX
    if shape.shape_type == Some(ShapeType::TextBox) {
        if let Some(text_content) = &shape.text {
            let text = extract_text_from_text_content(text_content);
            // Return text even if it's just whitespace initially, trimming happens later
            if !text.is_empty() {
                // Trim surrounding whitespace from the shape's text but preserve internal newlines
                // Important: Trim before checking if empty again
                let trimmed_shape_text = text.trim();
                if !trimmed_shape_text.is_empty() {
                    return Some(trimmed_shape_text.to_string());
                }
            }
        }
    }
    None // Not a TextBox or no text content
}

/// Extracts text from a Table element.
/// Formats as a simple concatenation of cell text, row by row, left to right.
fn extract_text_from_table(table: &Table) -> Option<String> {
    let mut table_text = String::new();
    if let Some(rows) = &table.table_rows {
        for row in rows {
            let mut row_text = String::new();
            if let Some(cells) = &row.table_cells {
                for cell in cells {
                    if let Some(text_content) = &cell.text {
                        // Extract and trim text within each cell
                        let cell_content = extract_text_from_text_content(text_content);
                        let trimmed_cell = cell_content.trim();
                        if !trimmed_cell.is_empty() {
                            row_text.push_str(trimmed_cell);
                            row_text.push(' '); // Add space *after* non-empty cell contents
                        }
                    }
                }
            }
            // Trim trailing space from the row and add newline if row wasn't empty
            let trimmed_row = row_text.trim_end();
            if !trimmed_row.is_empty() {
                table_text.push_str(trimmed_row);
                table_text.push('\n');
            }
        }
    }

    // Trim final newline from the table block
    let trimmed_table = table_text.trim_end();
    if !trimmed_table.is_empty() {
        Some(trimmed_table.to_string())
    } else {
        None
    }
}

/// Extracts text from a single PageElement by dispatching to specific element type handlers.
fn extract_text_from_page_element(element: &PageElement) -> Option<String> {
    match &element.element_kind {
        PageElementKind::Shape(shape) => extract_text_from_shape(shape),
        PageElementKind::Table(table) => extract_text_from_table(table),
        // Add other element kinds here if they can contain extractable text
        // e.g., PageElementKind::ElementGroup(group) => extract_text_from_group(group),
        _ => None, // Ignore other element types for text extraction
    }
}

/// Extracts and concatenates text from all relevant elements on a single slide, sorted vertically.
fn extract_text_from_slide(slide: &Page) -> Option<String> {
    let mut slide_parts: Vec<String> = Vec::new();

    if let Some(elements) = &slide.page_elements {
        let mut sorted_elements: Vec<&PageElement> = elements.iter().collect();
        sorted_elements.sort_by(|a, b| compare_elements_by_y(a, b));

        for element in sorted_elements {
            if let Some(text) = extract_text_from_page_element(element) {
                // The text from extractors should already be reasonably trimmed
                // Add the non-empty text block directly
                slide_parts.push(text);
            }
        }
    }

    if !slide_parts.is_empty() {
        // Join the parts with a single newline
        Some(slide_parts.join("\n"))
    } else {
        None // No text found on this slide
    }
}

// --- Public API Function ---

/// Extracts text from all slides in a presentation, formats it as Markdown.
/// Includes presentation title and slide headers, sorted vertically within slides.
///
/// # Arguments
///
/// * `presentation` - A reference to the `Presentation` object.
///
/// # Returns
///
/// A `String` containing the extracted text formatted in a Markdown structure.
pub fn extract_text_from_presentation(presentation: &Presentation) -> String {
    let mut full_text = String::new();

    // Add Presentation Header
    writeln!(full_text, "# Presentation").expect("Writing to String failed");
    if let Some(title) = &presentation.title {
        writeln!(full_text, "{}\n", title).expect("Writing to String failed");
    } else {
        full_text.push('\n'); // Add newline even if no title
    }

    let mut first_slide = true;
    if let Some(slides) = &presentation.slides {
        for (index, slide) in slides.iter().enumerate() {
            if let Some(slide_content) = extract_text_from_slide(slide) {
                // Add separator before the second slide onwards
                if !first_slide {
                    writeln!(full_text, "\n---\n").expect("Writing to String failed");
                } else {
                    first_slide = false;
                }

                // Add Slide Header (1-based index)
                writeln!(full_text, "## Slide {}\n", index + 1).expect("Writing to String failed");
                // Add Slide Content
                writeln!(full_text, "{}", slide_content).expect("Writing to String failed");
            }
            // If extract_text_from_slide returns None, we simply skip adding that slide's section
        }
    }

    full_text.to_string()
}

// --- Optional: Example Usage (Requires enabling test feature or separate binary) ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::presentation::Presentation; // Adjust path as needed
    use std::fs; // Need std::io::Write for flush

    #[test]
    fn test_extraction_from_json() {
        // Load a sample presentation JSON (replace with your actual path)
        let json_path = "changed_presentation.json";
        // let json_path = "base_presentation.json";

        let json_string =
            fs::read_to_string(json_path).expect("Should have been able to read the file");

        // Add print statements to debug deserialization if needed
        // println!("Attempting to deserialize JSON from: {}", json_path);
        // std::io::stdout().flush().unwrap(); // Ensure print happens before potential panic

        let presentation: Presentation = match serde_json::from_str(&json_string) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Deserialization failed: {}", e);
                // Optionally print a snippet of the JSON
                let snippet_len = json_string.len().min(500);
                eprintln!("JSON Snippet:\n{}", &json_string[..snippet_len]);
                panic!("Failed to deserialize presentation JSON");
            }
        };

        // println!("Deserialization successful."); // Confirm success
        // std::io::stdout().flush().unwrap();

        let extracted_text = extract_text_from_presentation(&presentation);

        // Basic assertion: Check if the output is non-empty if slides exist
        if presentation
            .slides
            .as_ref()
            .map_or(false, |s| !s.is_empty())
        {
            // Only assert non-empty if there were slides to potentially extract from
            let has_content = presentation.slides.as_ref().unwrap().iter().any(|slide| {
                slide
                    .page_elements
                    .as_ref()
                    .map_or(false, |elements| !elements.is_empty())
            });
            if has_content {
                // More specific check: only assert if slides *have* elements
                assert!(
                    !extracted_text.is_empty(),
                    "Extracted text should not be empty if presentation has slides with elements"
                );
            }
        }

        println!("--- Extracted Text ---");
        println!("{}", extracted_text);
        println!("--- End Extracted Text ---");

        // Optional: Write to a file
        let output_path = "extracted_text.md";
        let erorr_msg = format!("Unable to write file: {}", output_path);
        fs::write(output_path, extracted_text.clone()).expect(&erorr_msg);
        println!("Extracted text written to {}", output_path);

        // Add more specific assertions based on the *expected* content
        // of your `changed_presentation.json` file.
        // Example:
        // assert!(extracted_text.contains("# Presentation"));
        // if presentation.title.is_some() {
        //     assert!(extracted_text.contains(presentation.title.as_ref().unwrap()));
        // }
        // assert!(extracted_text.contains("## Slide 1"));
        // assert!(extracted_text.contains("Expected text from slide 1")); // Replace with actual expected text
        // assert!(extracted_text.contains("\n---\n")); // Check for separator if more than one slide has text
        // assert!(extracted_text.contains("## Slide 2"));
        // assert!(extracted_text.contains("Expected text from slide 2"));
    }
}
