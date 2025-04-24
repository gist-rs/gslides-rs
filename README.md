# gslides-rs
A Rust library for parsing Google Slides presentations using the v1 API.

## Overview

`gslides_rs` provides Rust data structures mirroring the Google Slides API v1 resources (like Presentations, Pages, Page Elements) and a basic client to fetch presentation data using service account authentication.

The primary goal is to deserialize the JSON response from the `presentations.get` API endpoint into strongly-typed Rust structs, enabling further processing, analysis, or integration.

## Features

* **Data Structures:** Defines comprehensive Rust structs for most Google Slides API v1 resources (Presentations, Pages, Shapes, Images, Tables, TextRuns, etc.).
* **Deserialization:** Uses `serde` for robust JSON parsing.
* **API Client:** Includes an asynchronous client function (`get_presentation_sa`) to fetch presentation data using `reqwest`.
* **Authentication:** Supports authentication via Google Service Accounts (using `yup-oauth2`).
* **Error Handling:** Provides a dedicated error enum (`SlidesApiError`) using `thiserror`.

*(Current Limitation: Primarily focused on parsing read-only data from `presentations.get`. Does not yet include methods for creating or modifying presentations via `batchUpdate`.)*

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
gslides_rs = "0.1.0" # Or the latest version

# You also need runtime and potentially other dependencies if using the client directly
tokio = { version = "1", features = ["full"] }
reqwest = "0.12"
dotenvy = "0.15"
````

## Authentication Setup (Service Account)

This library uses Google Service Accounts for authentication, which is suitable for server-to-server interactions or applications acting independently.

1.  **Google Cloud Project:**
      * Create or select a project in the [Google Cloud Console](https://console.cloud.google.com/).
      * Enable the **Google Slides API** and **Google Drive API** for your project (APIs & Services \> Library).
      * Configure the OAuth consent screen if you haven't already (APIs & Services \> OAuth consent screen). Even for service accounts, some basic configuration might be needed.
2.  **Service Account Credentials:**
      * Go to APIs & Services \> Credentials.
      * Click "Create Credentials" \> "Service account".
      * Give the service account a name (e.g., "slides-parser-service").
      * Grant necessary roles (optional, access is usually controlled by sharing).
      * Create a key for the service account (JSON format) and download the key file. **Keep this file secure\!**
3.  **Share Presentation:**
      * Share the specific Google Slides presentation(s) you want to access with the service account's email address (found in the key file or Cloud Console), giving it at least "Viewer" permission.
4.  **Environment Variable:**
      * Set the `GOOGLE_APPLICATION_CREDENTIALS` environment variable to the **absolute path** of the downloaded service account key file.
      * A convenient way to do this during development is using a `.env` file in your project root:
        ```dotenv
        # .env
        GOOGLE_APPLICATION_CREDENTIALS="/path/to/your/service-account-key.json"
        ```
      * Ensure your application loads this using `dotenvy::dotenv().ok();` at startup.
5.  **Required Scopes:** The client currently requests the following OAuth scopes:
      * `https://www.googleapis.com/auth/presentations.readonly`
      * `https://www.googleapis.com/auth/drive.readonly` (Needed to verify access permissions)

## Usage Example

This example fetches a presentation and prints basic information and element IDs from the first slide.

```rust
// examples/fetch_presentation.rs

use gslides_rs::{client, errors::SlidesApiError, models::presentation::Presentation};
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file (reads GOOGLE_APPLICATION_CREDENTIALS)
    dotenv().expect("Failed to load .env file.");

    // Get presentation ID from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example fetch_presentation -- <PRESENTATION_ID>");
        return Ok(());
    }
    let presentation_id = &args[1];

    println!("Attempting to fetch presentation: {}", presentation_id);

    // Create a reqwest client
    let http_client = reqwest::Client::new();

    // Fetch presentation using service account auth
    match client::get_presentation_sa(presentation_id, &http_client).await {
        Ok(presentation) => {
            println!("\nSuccessfully fetched presentation!");
            println!("Title: {}", presentation.title.unwrap_or_default());
            println!("ID: {}", presentation.presentation_id);
            println!("Number of Slides: {}", presentation.slides.as_ref().map_or(0, |s| s.len()));

            // Print element IDs from the first slide
            if let Some(slides) = &presentation.slides {
                if let Some(first_slide) = slides.first() {
                    println!("\nElements on first slide (ID: {}):", first_slide.object_id);
                    if let Some(elements) = &first_slide.page_elements {
                        for element in elements {
                            println!("  - Element ID: {} (Type: {:?})",
                                element.object_id,
                                // Optionally display the specific type using a helper or match
                                element.element_kind.as_ref().map(|k| match k {
                                    gslides_rs::models::elements::PageElementKind::Shape(_) => "Shape",
                                    gslides_rs::models::elements::PageElementKind::Image(_) => "Image",
                                    gslides_rs::models::elements::PageElementKind::Table(_) => "Table",
                                    gslides_rs::models::elements::PageElementKind::Line(_) => "Line",
                                    gslides_rs::models::elements::PageElementKind::Video(_) => "Video",
                                    _ => "Other"
                                })
                            );
                            // For full details:
                            // println!("{:#?}", element);
                        }
                    } else {
                        println!("  (No elements found)");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("\nError fetching presentation: {}", e);
            // Handle specific errors as needed
        }
    }

    Ok(())
}
```

## Run Example

```
cargo run --example fetch_presentation -- YOUR_PRESENTATION_ID
```

## Error Handling

The client functions return `Result<T, gslides_rs::errors::SlidesApiError>`. Check the `SlidesApiError` enum variants for details on possible failures (network, auth, API errors, JSON parsing, etc.).

## Contributing

Contributions are welcome\! Please feel free to submit issues or pull requests.

## License

MIT License
