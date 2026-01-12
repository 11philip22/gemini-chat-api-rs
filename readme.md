# Gemini Chat API (Rust)

[![Crates.io](https://img.shields.io/crates/v/gemini-chat-api.svg)](https://crates.io/crates/gemini-chat-api)
[![Documentation](https://docs.rs/gemini-chat-api/badge.svg)](https://docs.rs/gemini-chat-api)

This Rust crate provides an unofficial client for interacting with Google's internal Gemini API. It is a port of the [Python gemini-chat-api](https://github.com/OEvortex/Gemini-Chat-API) and is built using `reqwest` for efficient and authenticated HTTP requests.

## Features

- **Asynchronous**: Built on `tokio` and `reqwest` for non-blocking I/O.
- **Conversation Management**: Maintains chat history and context.
- **File & Image Uploads**: Support for sending images and files in prompts.
- **Multiple Models**: Support for Gemini 2.0 Flash, 2.5 Pro, and others.
- **Auto-Rotation**: Automatically rotates cookies to keep the session alive.
- **Browser Impersonation**: Mimics Chrome headers to ensure successful authentication.

> **Note**: Image generation and downloading features from the Python library are **not** supported in this Rust port. This client focuses on chat and text interaction.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
gemini-chat-api = "0.1.0"
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "multipart", "cookies"] }
serde_json = "1.0"
```

(Or path dependency if working locally)

## Usage

### Prerequisites

You need to obtain your `__Secure-1PSID` and `__Secure-1PSIDTS` cookies from Google Gemini.

1.  Go to [https://gemini.google.com/app](https://gemini.google.com/app)
2.  Open your browser's developer tools (F12).
3.  Go to the "Application" (or "Storage") tab.
4.  Under "Cookies" -> "https://gemini.google.com", find the `__Secure-1PSID` and `__Secure-1PSIDTS` cookies.
5.  Create a JSON file (e.g., `cookies.json`) with the following format:

```json
[
    {
        "name": "__Secure-1PSID",
        "value": "YOUR_VALUE_HERE"
    },
    {
        "name": "__Secure-1PSIDTS",
        "value": "YOUR_VALUE_HERE"
    }
]
```

### Quick Start

```rust
use gemini_chat_api::{utils::load_cookies, AsyncChatbot, Model};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load cookies from file
    let (secure_1psid, secure_1psidts) = load_cookies("cookies.json")?;

    println!("Cookies loaded successfully.");

    // Initialize chatbot with 30s timeout
    let mut chatbot = AsyncChatbot::new(
        &secure_1psid,
        &secure_1psidts,
        Model::G2_5Pro,
        None, // No proxy
        30,   // Timeout in seconds
    )
    .await?;

    println!("Chatbot initialized.");

    // Ask a question
    println!("Sending message: 'Hello from Rust example!'");
    let response = chatbot.ask("Hello from Rust example!", None).await?;

    println!("--------------------------------------------------");
    println!("Gemini Response:");
    println!("{}", response.content);
    println!("--------------------------------------------------");

    Ok(())
}
```

## Modules

- **`client`**: Contains the `AsyncChatbot` struct for managing sessions.
- **`enums`**: Defines `Endpoint`, `Headers`, and `Model` enums.
- **`utils`**: Helpers like `load_cookies` and `upload_file`.
- **`error`**: Custom `Error` types.

## License

This project is licensed under the MIT License - see the [license](license) file for details.
