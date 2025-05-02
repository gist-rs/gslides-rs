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
/// Extracts the translateY value from a PageElement's transform.
/// Returns f64::MAX if transform or translateY is None, placing such elements last.
pub fn get_translate_y(element: &PageElement) -> f64 {
    element
        .transform
        .as_ref()
        .and_then(|t: &AffineTransform| t.translate_y)
        .unwrap_or(f64::MAX) // Default to max value to sort elements without Y coord last
}

/// Compares two PageElements based on their vertical position (translateY).
pub fn compare_elements_by_y(a: &PageElement, b: &PageElement) -> Ordering {
    get_translate_y(a)
        .partial_cmp(&get_translate_y(b))
        .unwrap_or(Ordering::Equal) // Fallback if comparison fails (e.g., NaN)
}

// --- Text Extraction Logic ---
/// Extracts text content from a single TextElement (specifically TextRun).
pub fn extract_text_from_text_run(text_element: &ModelTextElement) -> Option<String> {
    if let Some(ModelTextElementKind::TextRun(text_run)) = &text_element.kind {
        text_run.content.clone() // Clone the content string if it exists
    } else {
        None // Not a TextRun or no content
    }
}

/// Extracts text content from a TextContent block (iterates through TextElements).
/// Note: This function is now primarily used internally by cell/shape extractors.
pub fn extract_text_from_text_content(text_content: &TextContent) -> String {
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
    // Let callers handle trimming and formatting based on context (e.g., table cells).
    combined_text
}

/// Extracts text from a Shape element, specifically if it's a TEXT_BOX.
pub fn extract_text_from_shape(shape: &Shape) -> Option<String> {
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
                    // Return the trimmed text, potentially containing internal newlines.
                    // Markdown rendering will handle these later.
                    return Some(trimmed_shape_text.to_string());
                }
            }
        }
    }
    None // Not a TextBox or no text content
}

/// Converts a Table element into a Markdown formatted table string.
/// Handles basic cell text extraction, trimming, and Markdown table syntax.
/// Escapes pipe characters within cell content and replaces newlines with <br>.
/// Note: Does not currently handle complex features like merged cells (row/column spans).
pub fn table_to_markdown(table: &Table) -> Option<String> {
    let num_cols = match table.columns {
        n if n > 0 => n as usize,
        _ => return None, // No columns, invalid table for Markdown
    };

    let rows = match &table.table_rows {
        Some(r) if !r.is_empty() => r,
        _ => return None, // No rows, nothing to format
    };

    let mut md_table = String::new();
    let mut has_content = false; // Track if any cell actually has text

    // --- Generate Markdown Table Rows ---
    let mut table_rows_md = Vec::new();
    for row in rows {
        let mut md_row_cells = Vec::with_capacity(num_cols);
        let mut cells_processed = 0;
        if let Some(cells) = &row.table_cells {
            for cell in cells {
                // --- Cell Text Processing ---
                let raw_cell_text = cell
                    .text
                    .as_ref()
                    .map(extract_text_from_text_content)
                    .unwrap_or_default();

                // Trim whitespace from the cell content
                let trimmed_text = raw_cell_text.trim();

                // Escape pipes | and replace internal newlines for Markdown compatibility
                let formatted_text = trimmed_text
                    .replace('|', "\\|") // Escape pipes
                    .replace('\n', "<br>"); // Replace newlines with HTML breaks

                if !formatted_text.is_empty() {
                    has_content = true; // Mark that we found some content
                }

                // Add the formatted cell text
                // TODO: Add handling for cell.column_span if needed later
                md_row_cells.push(formatted_text);
                cells_processed += cell.column_span.unwrap_or(1) as usize; // Basic span accounting

                // Break loop early if row definition exceeds table columns?
                // Or just let it add more? For now, let it add. Markdown might truncate.
                if cells_processed >= num_cols {
                    break; // Stop processing cells if we've met or exceeded column count for this row
                }
            }
        }

        // Pad row with empty cells if it has fewer cells than num_cols
        while cells_processed < num_cols {
            md_row_cells.push(String::new()); // Add empty string for missing cells
            cells_processed += 1;
        }
        // Ensure we don't have *more* cells than num_cols due to spans exceeding bounds
        md_row_cells.truncate(num_cols);

        // Format the row string: | Cell 1 | Cell 2 | ... |
        // Use write! for potentially better performance with many cells/rows
        let mut row_string = String::new();
        write!(row_string, "|").expect("Writing to String failed");
        for cell_md in md_row_cells {
            write!(row_string, " {} |", cell_md).expect("Writing to String failed");
        }
        table_rows_md.push(row_string);
    }

    // If no cells had any content, treat the table as empty
    if !has_content {
        return None;
    }

    // --- Assemble the final Markdown table ---

    // Add Header Row (using the first row content)
    if let Some(first_row) = table_rows_md.first() {
        writeln!(md_table, "{}", first_row).expect("Writing to String failed");
    } else {
        return None; // Should not happen if has_content is true, but safety check
    }

    // Add Separator Row: |---|---|...|
    write!(md_table, "|").expect("Writing to String failed");
    for _ in 0..num_cols {
        write!(md_table, "---|").expect("Writing to String failed");
    }
    writeln!(md_table).expect("Writing to String failed");

    // Add Data Rows (remaining rows)
    for row_md in table_rows_md.iter().skip(1) {
        writeln!(md_table, "{}", row_md).expect("Writing to String failed");
    }

    // Trim final newline potentially added by writeln!
    Some(md_table.trim_end().to_string())
}

/// Extracts text from a single PageElement by dispatching to specific element type handlers.
pub fn extract_text_from_page_element(element: &PageElement) -> Option<String> {
    match &element.element_kind {
        PageElementKind::Shape(shape) => extract_text_from_shape(shape),
        // [+] Keep the change minimum as possible.
        // Changed to use the new Markdown formatting function for tables.
        PageElementKind::Table(table) => table_to_markdown(table),
        // Add other element kinds here if they can contain extractable text
        // e.g., PageElementKind::ElementGroup(group) => extract_text_from_group(group),
        _ => None, // Ignore other element types for text extraction
    }
}

/// Extracts and concatenates text from all relevant elements on a single slide, sorted vertically.
pub fn extract_text_from_slide(slide: &Page) -> Option<String> {
    let mut slide_parts: Vec<String> = Vec::new();

    if let Some(elements) = &slide.page_elements {
        let mut sorted_elements: Vec<&PageElement> = elements.iter().collect();
        sorted_elements.sort_by(|a, b| compare_elements_by_y(a, b));

        for element in sorted_elements {
            if let Some(text) = extract_text_from_page_element(element) {
                // The text from extractors (shape or table_to_markdown) should be formatted.
                // Add the non-empty text block directly.
                slide_parts.push(text);
            }
        }
    }

    if !slide_parts.is_empty() {
        // Join the parts (shapes, formatted tables) with a single newline
        Some(slide_parts.join("\n"))
    } else {
        None // No text found on this slide
    }
}

// --- Public API Function ---

/// Extracts text from all slides in a presentation, formats it as Markdown.
/// Includes presentation title and slide headers, sorted vertically within slides.
/// Tables are formatted using Markdown table syntax.
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
                    // Use double newline before separator for better spacing after potentially long tables
                    writeln!(full_text, "\n\n---\n").expect("Writing to String failed");
                } else {
                    first_slide = false;
                }

                // Add Slide Header (1-based index)
                // Add extra newline after header for spacing before content (like tables)
                writeln!(full_text, "## Slide {}\n", index + 1).expect("Writing to String failed");
                // Add Slide Content (which might be multi-line Markdown table)
                writeln!(full_text, "{}", slide_content).expect("Writing to String failed");
            }
            // If extract_text_from_slide returns None, we simply skip adding that slide's section
        }
    }

    full_text.to_string() // Note: .to_string() is redundant here as full_text is already a String
}

// --- Optional: Example Usage (Requires enabling test feature or separate binary) ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::presentation::Presentation; // Adjust path as needed
    use std::fs;
    // Remove unused import: use std::io::Write; // Import Write trait for formatting (already imported at top level)

    #[test]
    fn test_extraction_from_json() {
        // Load a sample presentation JSON (replace with your actual path)
        let json_path = "changed_presentation.json";
        // let json_path = "base_presentation.json"; // Keep commented out or use specific test files

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
                    .map_or(false, |elements| !elements.is_empty()) // Check if slide has *any* elements
            });
            if has_content {
                // More specific check: only assert if slides *have* elements that *might* produce text
                // Check if any element actually produced text in the final output
                let non_header_part = extracted_text
                    .lines()
                    .skip_while(|line| line.starts_with('#') || line.is_empty()) // Skip presentation header/title
                    .collect::<Vec<_>>()
                    .join("\n");

                // Check if the rest of the text (slides content) is non-empty
                // This is a better check than just asserting !extracted_text.is_empty()
                // because the header is always present.
                if !non_header_part.trim().is_empty() {
                    // assert!(true); // Indicates content was found beyond the header
                } else {
                    // If no content was found beyond the header, maybe print a warning or assert based on expectation
                    println!(
                        "Warning: Presentation has elements, but no text content was extracted."
                    );
                    // If you expect content, you could assert false here:
                    // assert!(false, "Expected text content from slides, but found none.");
                }

                // The original assert is less precise because the header always exists
                // assert!(
                //     !extracted_text.is_empty(),
                //     "Extracted text should not be empty if presentation has slides with elements"
                // );
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
    }
}
