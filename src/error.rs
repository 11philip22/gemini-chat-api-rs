//! Error types for the Gemini Chat API client.

use thiserror::Error;

/// Main error type for the Gemini client.
#[derive(Error, Debug)]
pub enum Error {
    /// Authentication failed - cookies are invalid or expired.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Network request failed.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Failed to parse response from Gemini.
    #[error("Parse error: {0}")]
    Parse(String),

    /// Request timed out.
    #[error("Request timed out")]
    Timeout,

    /// Cookie file not found or invalid.
    #[error("Cookie error: {0}")]
    Cookie(String),

    /// File I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Client not initialized properly.
    #[error("Client not initialized: {0}")]
    NotInitialized(String),

    /// File upload failed.
    #[error("Upload failed: {0}")]
    Upload(String),
}

/// Result type alias for Gemini operations.
pub type Result<T> = std::result::Result<T, Error>;
