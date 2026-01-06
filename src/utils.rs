//! Utility functions for cookie loading and file upload.

use crate::enums::{upload_headers, Endpoint};
use crate::error::{Error, Result};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Cookie entry from browser export JSON format.
#[derive(Debug, Deserialize)]
struct CookieEntry {
    name: String,
    value: String,
}

/// Loads authentication cookies from a JSON file.
///
/// The file should be in the browser cookie export format:
/// ```json
/// [
///   { "name": "__Secure-1PSID", "value": "..." },
///   { "name": "__Secure-1PSIDTS", "value": "..." }
/// ]
/// ```
///
/// # Arguments
/// * `cookie_path` - Path to the JSON cookie file
///
/// # Returns
/// A tuple of (secure_1psid, secure_1psidts) values
///
/// # Errors
/// Returns an error if the file is not found, invalid JSON, or missing required cookies.
pub fn load_cookies(cookie_path: &str) -> Result<(String, String)> {
    let path = Path::new(cookie_path);
    if !path.exists() {
        return Err(Error::Cookie(format!(
            "Cookie file not found at path: {}",
            cookie_path
        )));
    }

    let content = std::fs::read_to_string(path)?;
    let cookies: Vec<CookieEntry> = serde_json::from_str(&content)
        .map_err(|e| Error::Cookie(format!("Invalid JSON format in cookie file: {}", e)))?;

    let mut secure_1psid: Option<String> = None;
    let mut secure_1psidts: Option<String> = None;

    for cookie in cookies {
        match cookie.name.to_uppercase().as_str() {
            "__SECURE-1PSID" => secure_1psid = Some(cookie.value),
            "__SECURE-1PSIDTS" => secure_1psidts = Some(cookie.value),
            _ => {}
        }
    }

    match (secure_1psid, secure_1psidts) {
        (Some(psid), Some(psidts)) => Ok((psid, psidts)),
        (None, _) => Err(Error::Cookie(
            "Required cookie __Secure-1PSID not found".to_string(),
        )),
        (_, None) => Err(Error::Cookie(
            "Required cookie __Secure-1PSIDTS not found".to_string(),
        )),
    }
}

/// Uploads a file to Google's Gemini server and returns its identifier.
///
/// # Arguments
/// * `file_data` - The file content as bytes
/// * `proxy` - Optional proxy URL
///
/// # Returns
/// The file identifier string from the server
///
/// # Errors
/// Returns an error if the upload fails.
pub async fn upload_file(file_data: &[u8], proxy: Option<&str>) -> Result<String> {
    let mut builder = Client::builder();

    if let Some(proxy_url) = proxy {
        builder = builder
            .proxy(reqwest::Proxy::all(proxy_url).map_err(|e| Error::Upload(e.to_string()))?);
    }

    let client = builder.build().map_err(|e| Error::Upload(e.to_string()))?;

    // Create multipart form with the file
    let part = reqwest::multipart::Part::bytes(file_data.to_vec()).file_name("file");
    let form = reqwest::multipart::Form::new().part("file", part);

    let response: reqwest::Response = client
        .post(Endpoint::Upload.url())
        .headers(upload_headers())
        .multipart(form)
        .send()
        .await
        .map_err(|e| Error::Upload(e.to_string()))?;

    if !response.status().is_success() {
        return Err(Error::Upload(format!(
            "Upload failed with status: {}",
            response.status()
        )));
    }

    let text = response
        .text()
        .await
        .map_err(|e| Error::Upload(e.to_string()))?;
    Ok(text)
}

/// Loads cookies from file and returns them as a HashMap for reqwest.
pub fn cookies_to_map(secure_1psid: &str, secure_1psidts: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("__Secure-1PSID".to_string(), secure_1psid.to_string());
    map.insert("__Secure-1PSIDTS".to_string(), secure_1psidts.to_string());
    map
}
