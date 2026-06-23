<h1 align="center">gemini-chat-api</h1>

<p align="center">
  <strong>Async Rust client for Google's Gemini web chat endpoints.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/gemini-chat-api"><img src="https://img.shields.io/crates/v/gemini-chat-api?style=for-the-badge&logo=rust&label=crates.io&color=F59E0B" alt="Crates.io"></a>
  <a href="https://docs.rs/gemini-chat-api"><img src="https://img.shields.io/docsrs/gemini-chat-api?style=for-the-badge&logo=readthedocs&label=docs.rs&color=3B82F6" alt="docs.rs"></a>
  <a href="https://github.com/11philip22/gemini-chat-api-rs"><img src="https://img.shields.io/badge/repo-github-181717?style=for-the-badge&logo=github" alt="GitHub repository"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/license-MIT-22C55E?style=for-the-badge" alt="MIT License"></a>
</p>

<p align="center">
  <a href="#features">Features</a>
  &middot;
  <a href="#installation">Installation</a>
  &middot;
  <a href="#authentication">Authentication</a>
  &middot;
  <a href="#quick-start">Quick Start</a>
  &middot;
  <a href="#image-input">Image Input</a>
  &middot;
  <a href="#conversation-state">Conversation State</a>
  &middot;
  <a href="#models">Models</a>
</p>

---

`gemini-chat-api` is an async Rust client for the same Gemini web chat endpoints used by the browser UI. It authenticates with Gemini cookies, sends text prompts, optionally uploads image bytes, and keeps conversation metadata so follow-up messages continue the same thread.

> [!IMPORTANT]
> This crate talks to Gemini's web endpoints, not the official Google Gemini API. Endpoint behavior can change without notice, and valid browser cookies are required.

## Features

- Async-first client built on `tokio` and `reqwest`.
- Cookie-based authentication with `__Secure-1PSID` and `__Secure-1PSIDTS`.
- Stateful conversations across multiple `ask` calls.
- Optional image upload support for multimodal prompts.
- Selectable Gemini model headers, including Flash, Pro, and thinking variants.
- Proxy and timeout configuration through the client constructor.
- JSON save/load helpers for conversation state.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
gemini-chat-api = "0.1.5"
tokio = { version = "1", features = ["full"] }
```

## Authentication

Export the required cookies from an authenticated Gemini browser session at [gemini.google.com](https://gemini.google.com/app):

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

Save them as `cookies.json`, then load them with `load_cookies`.

> [!CAUTION]
> Treat `cookies.json` like a password. Do not commit it or share it.

## Quick Start

```rust
use gemini_chat_api::{load_cookies, AsyncChatbot, Model, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let (psid, psidts) = load_cookies("cookies.json")?;
    let mut chatbot = AsyncChatbot::new(&psid, &psidts, Model::G3_0Pro, None, 30).await?;

    let response = chatbot.ask("Hello from Rust.", None).await?;
    println!("{}", response.content);

    Ok(())
}
```

Run the included interactive example:

```bash
cargo run --example chat
```

## Image Input

Pass image bytes as the second argument to `ask`:

```rust
use gemini_chat_api::{load_cookies, AsyncChatbot, Model, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let (psid, psidts) = load_cookies("cookies.json")?;
    let mut chatbot = AsyncChatbot::new(&psid, &psidts, Model::default(), None, 30).await?;

    let image = std::fs::read("image.png")?;
    let response = chatbot.ask("Describe this image.", Some(&image)).await?;
    println!("{}", response.content);

    Ok(())
}
```

## Conversation State

`AsyncChatbot` keeps the latest conversation IDs internally, so follow-up prompts continue the same Gemini thread:

```rust
let first = chatbot.ask("Explain Rust ownership in one paragraph.", None).await?;
let second = chatbot.ask("Now give me a short example.", None).await?;

chatbot.reset();
let fresh = chatbot.ask("Start a new topic.", None).await?;
```

You can also persist and restore conversations:

```rust
chatbot.save_conversation("data/conversations.json", "rust-notes").await?;
let loaded = chatbot.load_conversation("data/conversations.json", "rust-notes").await?;
```

## Models

Use `Model::default()` for the endpoint default, or select a specific model:

```rust
use gemini_chat_api::Model;

let model = Model::G3_0Pro;
```

Available variants include:

- `Model::G2_0Flash`
- `Model::G2_0FlashThinking`
- `Model::G2_5Flash`
- `Model::G2_5Pro`
- `Model::G2_0ExpAdvanced`
- `Model::G2_5ExpAdvanced`
- `Model::G3_0Pro`
- `Model::G3_0Flash`
- `Model::G3_0Thinking`

## Acknowledgements

This crate is a Rust port inspired by [OEvortex/Gemini-Chat-API](https://github.com/OEvortex/Gemini-Chat-API).
