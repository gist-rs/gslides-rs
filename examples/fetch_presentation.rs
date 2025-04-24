// Use the chosen crate name
use gslides_rs::{client, errors::SlidesApiError};

use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file (especially GOOGLE_APPLICATION_CREDENTIALS)
    dotenv().expect("Failed to load .env file. Make sure it exists and is in the project root.");

    // Get presentation ID from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example fetch_presentation -- <PRESENTATION_ID>");
        eprintln!("Ensure GOOGLE_APPLICATION_CREDENTIALS is set in your environment or .env file.");
        return Ok(()); // Exit gracefully if no ID provided
    }
    let presentation_id = &args[1];

    println!("Attempting to fetch presentation: {}", presentation_id);

    // Create a reqwest client (usually shared across the application)
    let http_client = reqwest::Client::new();

    // Call the library function using Service Account authentication
    match client::get_presentation_sa(presentation_id, &http_client).await {
        Ok(presentation) => {
            println!("\nSuccessfully fetched presentation!");
            println!(
                "Title: {}",
                presentation
                    .title
                    .unwrap_or_else(|| "[Untitled]".to_string())
            );
            println!("ID: {}", presentation.presentation_id);
            println!(
                "Locale: {}",
                presentation
                    .locale
                    .unwrap_or_else(|| "[Not Set]".to_string())
            );
            if let Some(size) = &presentation.page_size {
                println!(
                    "Page Size: {}x{} {}",
                    size.width
                        .as_ref()
                        .map(|d| d.magnitude.unwrap_or(0.0))
                        .unwrap_or(0.0),
                    size.height
                        .as_ref()
                        .map(|d| d.magnitude.unwrap_or(0.0))
                        .unwrap_or(0.0),
                    // Display Unit enum nicely
                    size.width
                        .as_ref()
                        .and_then(|d| d.unit.as_ref())
                        .map(|u| format!("{:?}", u))
                        .unwrap_or_else(|| "UNIT_UNSPECIFIED".to_string())
                );
            }
            println!(
                "Number of Slides: {}",
                presentation.slides.as_ref().map_or(0, |s| s.len())
            );
            println!(
                "Number of Masters: {}",
                presentation.masters.as_ref().map_or(0, |m| m.len())
            );
            println!(
                "Number of Layouts: {}",
                presentation.layouts.as_ref().map_or(0, |l| l.len())
            );

            // Example: Print Object IDs of top-level elements on the first slide
            if let Some(slides) = &presentation.slides {
                if let Some(first_slide) = slides.first() {
                    println!("\nElements on first slide (ID: {}):", first_slide.object_id);
                    if let Some(elements) = &first_slide.page_elements {
                        for element in elements {
                            println!("  - Element ID: {}", element.object_id);
                        }
                    } else {
                        println!("  (No elements found)");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("\nError fetching presentation:");
            // --- Updated match statement ---
            match e {
                SlidesApiError::Network(err) => eprintln!("  Network/Request Error: {}", err),
                SlidesApiError::JsonDeserialization(err) => {
                    eprintln!("  JSON Parsing Error: {}", err)
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
                SlidesApiError::IoError(err) => eprintln!("  I/O Error: {}", err), // Added this arm
                SlidesApiError::Unknown(msg) => eprintln!("  Unknown Error: {}", msg),
            }
            // --- End updated match statement ---
        }
    }

    Ok(())
}
