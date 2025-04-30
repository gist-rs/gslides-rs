use crate::errors::{Result, SlidesApiError};
use crate::models::presentation::Presentation;
// use log::debug;
use reqwest::header::{ACCEPT, AUTHORIZATION};
use serde::Deserialize;
use std::env;
use std::fs; // Import the file system module
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "yup-oauth2")]
use yup_oauth2::{read_service_account_key, ServiceAccountAuthenticator};

/// Helper struct to attempt parsing standard Google API error responses.
#[derive(Deserialize, Debug)]
struct GoogleApiErrorResponse {
    error: GoogleApiErrorDetail,
}

/// Details within a standard Google API error response.
#[allow(unused)]
#[derive(Deserialize, Debug)]
struct GoogleApiErrorDetail {
    code: i32,
    message: String,
    status: String,
}

/// Fetches a presentation resource from the Google Slides API using Service Account credentials.
///
/// Reads the service account key file path from the `GOOGLE_APPLICATION_CREDENTIALS`
/// environment variable. Ensure `dotenvy::dotenv().ok();` has been called beforehand.
/// If JSON deserialization fails, writes the raw JSON response to `deserialization_error.json`.
///
/// # Arguments
///
/// * `presentation_id` - The ID of the presentation to fetch.
/// * `http_client` - An asynchronous `reqwest::Client` instance (used for the final API call).
///
/// # Errors
///
/// Returns `SlidesApiError` for various issues.
///
/// # Returns
///
/// A `Result` containing the parsed `Presentation` on success, or a `SlidesApiError` on failure.
pub async fn get_presentation_sa(
    presentation_id: &str,
    http_client: &reqwest::Client, // Keep reqwest client for the main API call
) -> Result<Presentation> {
    if presentation_id.is_empty() {
        return Err(SlidesApiError::InvalidInput(
            "Presentation ID cannot be empty".to_string(),
        ));
    }

    // --- Service Account Authentication (yup-oauth2 v12+) ---
    let key_file_path = env::var("GOOGLE_APPLICATION_CREDENTIALS")?;
    let sa_key = read_service_account_key(Path::new(&key_file_path))
        .await
        .map_err(|e| {
            SlidesApiError::AuthSetupError(format!(
                "Failed to read service account key from '{}': {}",
                key_file_path, e
            ))
        })?;
    let auth = ServiceAccountAuthenticator::builder(sa_key).build().await?;
    let scopes = &[
        "https://www.googleapis.com/auth/presentations.readonly",
        "https://www.googleapis.com/auth/drive.readonly",
    ];
    let token = auth.token(scopes).await?;
    let access_token = token
        .token()
        .expect("OAuth token unexpectedly missing token field after successful retrieval");
    // --- End Authentication Section ---

    let api_url = format!(
        "https://slides.googleapis.com/v1/presentations/{}",
        presentation_id
    );

    // Perform the main API GET request
    let response = http_client
        .get(&api_url)
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .header(ACCEPT, "application/json")
        .send()
        .await
        .map_err(SlidesApiError::Network)?;

    // Handle response
    let status = response.status();
    if status.is_success() {
        let bytes = response.bytes().await.map_err(SlidesApiError::Network)?;

        // Attempt to deserialize.
        match serde_json::from_slice::<Presentation>(&bytes) {
            Ok(presentation) => {
                let filename = "changed_presentation.json";
                fs::write(filename, &bytes).expect("Error write file");
                Ok(presentation)
            }
            Err(e) => {
                // --- Write failing JSON to file ---
                let filename = "deserialization_error.json";
                eprintln!("-----------------------------------------");
                eprintln!("JSON Deserialization Error: {}", e);
                match fs::write(filename, &bytes) {
                    Ok(_) => eprintln!(
                        "Raw JSON response body saved to '{}' for debugging.",
                        filename
                    ),
                    Err(io_err) => eprintln!(
                        "Failed to write error JSON to file '{}': {}",
                        filename, io_err
                    ),
                }
                // Optionally print a snippet to stderr as well
                let json_snippet = String::from_utf8_lossy(&bytes[..bytes.len().min(500)]);
                eprintln!("Failing JSON snippet:\n{}", json_snippet);
                eprintln!("-----------------------------------------");

                // Return the specific deserialization error
                Err(SlidesApiError::JsonDeserialization(e))
            }
        }
        // --- End corrected success handling ---
    } else {
        // Handle API-level errors (non-2xx status codes)
        let error_text = response.text().await.map_err(SlidesApiError::Network)?;
        let message = match serde_json::from_str::<GoogleApiErrorResponse>(&error_text) {
            Ok(google_error) => google_error.error.message,
            Err(_) => format!("API request failed with status {}: {}", status, error_text),
        };
        Err(SlidesApiError::ApiError { status, message })
    }
}
