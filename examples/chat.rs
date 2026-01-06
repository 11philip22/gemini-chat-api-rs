//! Example: Basic chat with Gemini

use gemini_chat_api::{load_cookies, AsyncChatbot, Model};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load cookies from cookies.json (copy from your Python project)
    let cookie_path = "cookies.json";
    
    println!("Loading cookies from {}...", cookie_path);
    let (psid, psidts) = load_cookies(cookie_path)?;
    println!("Cookies loaded successfully.");

    // Create chatbot with default model
    println!("Initializing Gemini chatbot...");
    let mut chatbot = AsyncChatbot::new(
        &psid,
        &psidts,
        Model::G2_0Flash, // or Model::default() for unspecified
        None,             // No proxy
        30,               // 30 second timeout
    )
    .await?;
    println!("Chatbot initialized successfully!\n");

    // Interactive chat loop
    println!("=== Gemini Chat ===");
    println!("Type your message and press Enter. Type 'quit' to exit.\n");

    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("Goodbye!");
            break;
        }

        if input.is_empty() {
            continue;
        }

        match chatbot.ask(input, None).await {
            Ok(response) => {
                println!("\nGemini: {}\n", response.content);
            }
            Err(e) => {
                eprintln!("\nError: {}\n", e);
            }
        }
    }

    Ok(())
}
