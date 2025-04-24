// examples/fetch_presentation.rs

use gslides_rs::{
    client,
    errors::SlidesApiError,
    models::{
        common::Unit,
        elements::{PageElement, PageElementKind}, // Import PageElementKind
        presentation::Presentation,
        shape::Shape, // Import Shape
    },
};

use dotenvy::dotenv;
use serde_json::Value; // <-- Add this import
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect("Failed to load .env file. Make sure it exists and is in the project root.");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example fetch_presentation -- <PRESENTATION_ID>");
        eprintln!("Ensure GOOGLE_APPLICATION_CREDENTIALS is set in your environment or .env file.");
        return Ok(());
    }
    let presentation_id = &args[1];

    println!("Attempting to fetch presentation: {}", presentation_id);
    let http_client = reqwest::Client::new();

    match client::get_presentation_sa(presentation_id, &http_client).await {
        Ok(presentation) => {
            println!("\nSuccessfully fetched presentation!");
            println!(
                "Title: {}",
                presentation
                    .title
                    .unwrap_or_else(|| "[Untitled]".to_string())
            );
            // ... (print other presentation info) ...

            // --- Debugging: Print raw 'outline' JSON if found ---
            if let Some(slides) = &presentation.slides {
                if let Some(first_slide) = slides.first() {
                    if let Some(elements) = &first_slide.page_elements {
                        for element in elements {
                            if let PageElementKind::Shape(shape) = &element.element_kind {
                                if let Some(props) = &shape.shape_properties {
                                    let outline_value = &props.outline;

                                    println!(
                                        "\nFound raw 'outline' JSON for element '{}':\n{}",
                                        element.object_id,
                                        serde_json::to_string_pretty(outline_value)? // Pretty print the Value
                                    );
                                }
                            }
                        }
                    }
                }
            }
            // --- End Debugging ---
        }
        Err(e) => {
            eprintln!("\nError fetching presentation:");
            match e {
                SlidesApiError::Network(err) => eprintln!("  Network/Request Error: {}", err),
                SlidesApiError::JsonDeserialization(err) => {
                    // Error still occurs, but now we might get further?
                    // Or the error might occur when printing the Value if parsing failed earlier.
                    eprintln!("  JSON Parsing Error: {}", err);
                    eprintln!("  (Check deserialization_error.json if it was created)");
                }
                SlidesApiError::ApiError { status, message } => {
                    eprintln!("  API Error ({}): {}", status, message)
                }
                SlidesApiError::AuthSetupError(msg) => {
                    eprintln!("  Authentication Setup Error: {}", msg)
                }
                SlidesApiError::AuthLibError(err) => {
                    eprintln!("  Authentication Library Error: {}", err)
                }
                SlidesApiError::InvalidInput(msg) => eprintln!("  Invalid Input: {}", msg),
                SlidesApiError::EnvVarError(err) => eprintln!(
                    "  Environment Variable Error ({:?}): Check GOOGLE_APPLICATION_CREDENTIALS.",
                    err
                ),
                SlidesApiError::IoError(err) => eprintln!("  I/O Error: {}", err),
                SlidesApiError::Unknown(msg) => eprintln!("  Unknown Error: {}", msg),
            }
        }
    }

    Ok(())
}
