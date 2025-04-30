#[cfg(feature = "yup-oauth2")]
use gslides_rs::{
    client,
    errors::SlidesApiError,
    // Import specific element kinds if you want to match on them, otherwise Debug print works
    // models::elements::{PageElement, PageElementKind},
};

#[cfg(feature = "yup-oauth2")]
use dotenvy::dotenv;

#[cfg(feature = "yup-oauth2")]
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "yup-oauth2")]
    {
        dotenv()
            .expect("Failed to load .env file. Make sure it exists and is in the project root.");

        let args: Vec<String> = env::args().collect();
        if args.len() < 2 {
            eprintln!("Usage: cargo run --example fetch_presentation -- <PRESENTATION_ID>");
            eprintln!(
                "Ensure GOOGLE_APPLICATION_CREDENTIALS is set in your environment or .env file."
            );
            return Ok(());
        }
        let presentation_id = &args[1];
        // let output_filename = "output.json";

        println!("Attempting to fetch presentation: {}", presentation_id);
        let http_client = reqwest::Client::new();

        match client::get_presentation_sa(presentation_id, &http_client).await {
            Ok(presentation) => {
                println!("\nSuccessfully fetched presentation!");
                println!(
                    "Title: {}",
                    presentation
                        .title
                        .clone()
                        .unwrap_or_else(|| "[Untitled]".to_string())
                );
                println!("ID: {}", presentation.presentation_id);
                println!(
                    "Locale: {}",
                    presentation
                        .locale
                        .clone()
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

                // // --- Serialize and Write to File ---
                // println!("\nSerializing presentation to JSON...");
                // // Use `to_string_pretty` for readable output
                // let json_output = serde_json::to_string_pretty(&presentation)?; // Propagate serialization errors

                // println!("Writing presentation data to {}...", output_filename);
                // // Create or truncate the output file
                // let mut file = File::create(output_filename)?; // Propagate file creation errors
                //                                                // Write the JSON string bytes to the file
                // file.write_all(json_output.as_bytes())?; // Propagate file writing errors

                // println!(
                //     "Successfully wrote presentation data to {}.",
                //     output_filename
                // );
                // // --- End Serialize and Write to File ---

                // --- Updated section to print full element details ---
                if let Some(slides) = &presentation.slides {
                    if let Some(first_slide) = slides.first() {
                        println!(
                            "\n--- Elements on first slide (ID: {}) ---",
                            first_slide.object_id
                        );
                        if let Some(elements) = &first_slide.page_elements {
                            if elements.is_empty() {
                                println!("  (No elements found on this slide)");
                            } else {
                                for (index, element) in elements.iter().enumerate() {
                                    println!("\n[Element {} ID: {}]", index + 1, element.object_id);
                                    // Pretty-print the entire deserialized element structure
                                    println!("{:#?}", element);
                                }
                            }
                        } else {
                            println!("  (No elements array found for this slide)");
                        }
                        println!("\n--- End of elements for first slide ---");
                    } else {
                        println!("\nPresentation has no slides.");
                    }
                } else {
                    println!("\nPresentation has no slides array.");
                }
                // --- End updated section ---
            }
            Err(e) => {
                eprintln!("\nError fetching presentation:");
                match e {
                    SlidesApiError::Network(err) => eprintln!("  Network/Request Error: {}", err),
                    SlidesApiError::JsonDeserialization(err) => {
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
    }

    Ok(())
}
