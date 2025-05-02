# gslides-rs
A Rust library for parsing and comparing Google Slides presentations using the v1 API.

## Overview

`gslides_rs` provides Rust data structures mirroring the Google Slides API v1 resources (like Presentations, Pages, Page Elements) and utilities to interact with them. This includes:

1.  A basic client to fetch presentation data using service account authentication.
2.  A diffing engine to compare two presentation structures and report the differences.

The primary goals are to:
*   Deserialize the JSON response from the `presentations.get` API endpoint into strongly-typed Rust structs.
*   Provide tools to identify changes between different versions of a presentation (e.g., comparing a base to a modified version).

## Features

*   **Data Structures:** Defines comprehensive Rust structs for most Google Slides API v1 resources (Presentations, Pages, Shapes, Images, Tables, TextRuns, etc.).
*   **Deserialization:** Uses `serde` for robust JSON parsing.
*   **API Client:** Includes an asynchronous client function (`get_presentation_sa`) to fetch presentation data using `reqwest`.
*   **Authentication:** Supports authentication via Google Service Accounts (using `yup-oauth2`).
*   **Error Handling:** Provides dedicated error enums (`SlidesApiError`, `DiffError`) using `thiserror`.
*   **Presentation Diffing (requires `diff` feature):**
    *   Compares two `Presentation` objects structurally.
    *   Identifies added, removed, or modified elements/properties.
    *   Generates structured diff reports (list of changes with paths).
    *   Generates Git-style text diffs.
    *   Generates human-readable summaries of changes.

*(Current Limitation: Primarily focused on parsing read-only data from `presentations.get` and diffing. Does not yet include methods for creating or modifying presentations via `batchUpdate`.)*

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
gslides_rs = "0.1.0" # Or the latest version

# Include the diff feature if needed
# gslides_rs = { version = "0.1.0", features = ["diff"] }

# You also need runtime and potentially other dependencies if using the client/examples
tokio = { version = "1", features = ["full"] }
reqwest = "0.12"
dotenvy = "0.15"
```

By default, the `diff` feature is enabled. If you don't need the diffing capabilities, you can disable default features and explicitly include only what you need.

## Authentication Setup (Service Account)

This library uses Google Service Accounts for authentication, which is suitable for server-to-server interactions or applications acting independently.

1.  **Google Cloud Project:**
    *   Create or select a project in the [Google Cloud Console](https://console.cloud.google.com/).
    *   Enable the **Google Slides API** and **Google Drive API** for your project (APIs & Services > Library).
    *   Configure the OAuth consent screen if you haven't already (APIs & Services > OAuth consent screen). Even for service accounts, some basic configuration might be needed.
2.  **Service Account Credentials:**
    *   Go to APIs & Services > Credentials.
    *   Click "Create Credentials" > "Service account".
    *   Give the service account a name (e.g., "slides-parser-service").
    *   Grant necessary roles (optional, access is usually controlled by sharing).
    *   Create a key for the service account (JSON format) and download the key file. **Keep this file secure!**
3.  **Share Presentation:**
    *   Share the specific Google Slides presentation(s) you want to access with the service account's email address (found in the key file or Cloud Console), giving it at least "Viewer" permission.
4.  **Environment Variable:**
    *   Set the `GOOGLE_APPLICATION_CREDENTIALS` environment variable to the **absolute path** of the downloaded service account key file.
    *   A convenient way to do this during development is using a `.env` file in your project root:
        ```dotenv
        # .env
        GOOGLE_APPLICATION_CREDENTIALS="/path/to/your/service-account-key.json"
        ```
    *   Ensure your application loads this using `dotenvy::dotenv().ok();` at startup.
5.  **Required Scopes:** The client currently requests the following OAuth scopes:
    *   `https://www.googleapis.com/auth/presentations.readonly`
    *   `https://www.googleapis.com/auth/drive.readonly` (Needed to verify access permissions)

## Usage Example (Fetching Presentation)

This example fetches a presentation and prints basic information and element details from the first slide.

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

            // Print full element details from the first slide
            if let Some(slides) = &presentation.slides {
                if let Some(first_slide) = slides.first() {
                    println!("\n--- Elements on first slide (ID: {}) ---", first_slide.object_id);
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
        }
        Err(e) => {
            eprintln!("\nError fetching presentation: {}", e);
            // Handle specific errors as needed
            match e {
                SlidesApiError::Network(err) => eprintln!("  Network/Request Error: {}", err),
                SlidesApiError::JsonDeserialization(err) => {
                    eprintln!("  JSON Parsing Error: {}", err);
                    eprintln!("  (Check deserialization_error.json if it was created)");
                }
                SlidesApiError::ApiError { status, message } => {
                    eprintln!("  API Error ({}): {}", status, message)
                }
                _ => eprintln!("  Other error: {:?}", e), // Catch-all for other variants
            }
        }
    }

    Ok(())
}
```

## Usage Example (Diffing Presentations)

*(See the full example in `gslides-rs/examples/diff_presentation.rs` for more details and error handling.)*

## Run Examples

```bash
# Fetch a presentation (replace with a real ID)
cargo run --example fetch_presentation -- YOUR_PRESENTATION_ID

# Compare two JSON files (ensure base_presentation.json and changed_presentation.json exist)
# You might need to fetch presentations first and save them as JSON
# Example:
# cargo run --example fetch_presentation -- BASE_ID > base_presentation.json
# cargo run --example fetch_presentation -- CHANGED_ID > changed_presentation.json
cargo run --example diff_presentation
```

## Error Handling

The client functions return `Result<T, gslides_rs::errors::SlidesApiError>`. Check the `SlidesApiError` enum variants for details on possible failures (network, auth, API errors, JSON parsing, etc.).

The diff functions return `Result<T, gslides_rs::diff::error::DiffError>`. Check the `DiffError` enum variants for diff-specific issues (serialization, diffing logic, formatting).


## WASM

### Setup
```
cargo install wasm-pack
```

### Build/Release
```
# Build
wasm-pack build --scope gist-rs --release --target=nodejs

# Release
wasm-pack publish --access=public
```

### Build/Release (no wasm-pack)
```
# Build
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/gslides_rs.wasm --out-dir pkg --nodejs

# Release
npm publish pkg --access=public
```

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

MIT License
