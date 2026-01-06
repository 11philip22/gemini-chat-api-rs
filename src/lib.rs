//! Async Rust client for Google Gemini Chat API.
//!
//! This library provides an async HTTP client for interacting with Google Gemini,
//! similar to the Python `gemini_client` library.
//!
//! # Example
//! ```no_run
//! use gemini_chat_api::{AsyncChatbot, Model, load_cookies};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load cookies from file
//!     let (psid, psidts) = load_cookies("cookies.json")?;
//!
//!     // Create chatbot
//!     let mut chatbot = AsyncChatbot::new(&psid, &psidts, Model::default(), None, 30).await?;
//!
//!     // Send message
//!     let response = chatbot.ask("Hello! Tell me a joke.", None).await?;
//!     println!("{}", response.content);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod enums;
pub mod error;
pub mod utils;

// Re-exports for convenience
pub use client::{AsyncChatbot, ChatResponse, Choice, SavedConversation};
pub use enums::{Endpoint, Model};
pub use error::{Error, Result};
pub use utils::load_cookies;
