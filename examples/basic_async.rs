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
