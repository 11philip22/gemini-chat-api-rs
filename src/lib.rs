//! Async Rust client for the Gemini web chat endpoints.
//!
//! This crate talks to the same web endpoints used by Gemini's browser UI and
//! authenticates with browser cookies. It does not use API keys.
//!
//! **Key points**
//! - Async-first (`reqwest` + `tokio`).
//! - Requires `__Secure-1PSID`; `__Secure-1PSIDTS` is expected but may be rotated if missing.
//! - Stateful: the client keeps conversation IDs between calls.
//!
//! # Quick start
//! ```no_run
//! use gemini_chat_api::{AsyncChatbot, Model, Result, load_cookies};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let (psid, psidts) = load_cookies("cookies.json")?;
//!     let mut chatbot = AsyncChatbot::new(&psid, &psidts, Model::default(), None, 30).await?;
//!
//!     let response = chatbot.ask("Hello! Tell me a joke.", None).await?;
//!     println!("{}", response.content);
//!     Ok(())
//! }
//! ```
//!
//! # Image input
//! ```no_run
//! use gemini_chat_api::{AsyncChatbot, Model, Result, load_cookies};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let (psid, psidts) = load_cookies("cookies.json")?;
//!     let mut chatbot = AsyncChatbot::new(&psid, &psidts, Model::default(), None, 30).await?;
//!
//!     let image = std::fs::read("image.png")?;
//!     let response = chatbot.ask("Describe this image.", Some(&image)).await?;
//!     println!("{}", response.content);
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
