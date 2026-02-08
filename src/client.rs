//! Async client for Gemini web chat endpoints.

use crate::enums::{gemini_headers, rotate_cookies_headers, Endpoint, Model};
use crate::error::{Error, Result};
use crate::utils::upload_file;

use rand::Rng;
use regex::Regex;
use reqwest::cookie::Jar;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

const SNLM0E_PATTERN: &str = r#"["']SNlM0e["']\s*:\s*["']([^"']+)["']"#;

/// Response returned by [`AsyncChatbot::ask`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The primary response text.
    pub content: String,
    /// Conversation identifier returned by the server.
    pub conversation_id: String,
    /// Response identifier returned by the server.
    pub response_id: String,
    /// Optional structured data used by the backend for factuality checks.
    pub factuality_queries: Option<Value>,
    /// The text query echoed by the backend.
    pub text_query: String,
    /// Alternative response choices returned by the backend.
    pub choices: Vec<Choice>,
    /// Always `false` for successful responses.
    ///
    /// Errors are returned as [`Error`](crate::Error) instead of populating this field.
    pub error: bool,
}

/// An alternative response choice returned by the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    /// Backend choice identifier.
    pub id: String,
    /// Choice content text.
    pub content: String,
}

/// Persisted conversation state written by [`AsyncChatbot::save_conversation`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConversation {
    /// User-provided name used as the lookup key.
    pub conversation_name: String,
    /// Internal request ID used by the backend.
    #[serde(rename = "_reqid")]
    pub reqid: u32,
    /// Conversation identifier.
    pub conversation_id: String,
    /// Response identifier.
    pub response_id: String,
    /// Selected choice identifier.
    pub choice_id: String,
    /// Session token required for requests.
    #[serde(rename = "SNlM0e")]
    pub snlm0e: String,
    /// Model name string at the time of saving.
    pub model_name: String,
    /// Seconds since UNIX epoch, as a string.
    pub timestamp: String,
}

/// Async chatbot client for interacting with Gemini.
///
/// This client is stateful. Each successful call to [`ask`](Self::ask) updates
/// internal conversation IDs so the next call continues the same thread. Call
/// [`reset`](Self::reset) to start a new conversation while keeping cookies valid.
///
/// # Example
/// ```no_run
/// use gemini_chat_api::{load_cookies, AsyncChatbot, Model, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let (psid, psidts) = load_cookies("cookies.json")?;
///     let mut chatbot = AsyncChatbot::new(&psid, &psidts, Model::default(), None, 30).await?;
///
///     // Continues the same conversation thread across calls.
///     let first = chatbot.ask("Summarize Rust ownership in one paragraph.", None).await?;
///     println!("{}", first.content);
///
///     let followup = chatbot.ask("Now give me a short code example.", None).await?;
///     println!("{}", followup.content);
///
///     // Start a new conversation thread (cookies/session stay valid).
///     chatbot.reset();
///     let fresh = chatbot.ask("New topic: explain HTTP caching.", None).await?;
///     println!("{}", fresh.content);
///     Ok(())
/// }
/// ```
pub struct AsyncChatbot {
    client: Client,
    snlm0e: String,
    conversation_id: String,
    response_id: String,
    choice_id: String,
    reqid: u32,
    secure_1psidts: String,
    model: Model,
    proxy: Option<String>,
}

impl AsyncChatbot {
    /// Creates a new `AsyncChatbot` instance and fetches the session token.
    ///
    /// # Arguments
    /// * `secure_1psid` - The `__Secure-1PSID` cookie value
    /// * `secure_1psidts` - The `__Secure-1PSIDTS` cookie value (may be empty)
    /// * `model` - The model configuration to use
    /// * `proxy` - Optional proxy URL (applied to all requests)
    /// * `timeout` - Request timeout in seconds
    ///
    /// # Returns
    /// A fully initialized client with a valid session token.
    ///
    /// # Errors
    /// Returns an error if authentication fails (`Error::Authentication`),
    /// if the network request fails (`Error::Network`), or if the session token
    /// cannot be extracted (`Error::Parse`).
    ///
    /// There is no retry or polling behavior; a single request is attempted.
    /// Timeouts are handled by `reqwest` and surface as `Error::Network`.
    pub async fn new(
        secure_1psid: &str,
        secure_1psidts: &str,
        model: Model,
        proxy: Option<&str>,
        timeout: u64,
    ) -> Result<Self> {
        if secure_1psid.is_empty() {
            return Err(Error::Authentication(
                "__Secure-1PSID cookie is required".to_string(),
            ));
        }

        // Build cookie jar with proper Secure cookie attributes
        let jar = Jar::default();
        let url: Url = "https://gemini.google.com".parse().unwrap();
        // Secure cookies need proper attributes in the cookie string
        jar.add_cookie_str(
            &format!(
                "__Secure-1PSID={}; Domain=.google.com; Path=/; Secure; SameSite=None",
                secure_1psid
            ),
            &url,
        );
        jar.add_cookie_str(
            &format!(
                "__Secure-1PSIDTS={}; Domain=.google.com; Path=/; Secure; SameSite=None",
                secure_1psidts
            ),
            &url,
        );

        // Build headers
        let mut headers = gemini_headers();
        if let Some(model_headers) = model.headers() {
            headers.extend(model_headers);
        }

        // Build client
        let mut builder = Client::builder()
            .cookie_provider(Arc::new(jar))
            .default_headers(headers)
            .timeout(Duration::from_secs(timeout));

        if let Some(proxy_url) = proxy {
            builder = builder.proxy(reqwest::Proxy::all(proxy_url)?);
        }

        let client = builder.build()?;

        let mut chatbot = Self {
            client,
            snlm0e: String::new(),
            conversation_id: String::new(),
            response_id: String::new(),
            choice_id: String::new(),
            reqid: rand::thread_rng().gen_range(1000000..9999999),
            secure_1psidts: secure_1psidts.to_string(),
            model,
            proxy: proxy.map(|s| s.to_string()),
        };

        // Fetch the SNlM0e token
        chatbot.snlm0e = chatbot.get_snlm0e().await?;

        Ok(chatbot)
    }

    /// Fetches the SNlM0e value required for API requests.
    async fn get_snlm0e(&mut self) -> Result<String> {
        // Proactively try to rotate cookies if PSIDTS is missing
        if self.secure_1psidts.is_empty() {
            let _ = self.rotate_cookies().await;
        }

        let response = self.client.get(Endpoint::Init.url()).send().await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(Error::Authentication(format!(
                    "Authentication failed (status {}). Check cookies.",
                    status
                )));
            }
            return Err(Error::Parse(format!("HTTP error: {}", status)));
        }

        // Check for authentication redirect - be precise to avoid false positives
        // Only trigger if it's an actual login page, not just any page with google.com links
        if text.contains("\"identifier-shown\"")
            || text.contains("SignIn?continue")
            || text.contains("Sign in - Google Accounts")
        {
            return Err(Error::Authentication(
                "Authentication failed. Cookies might be invalid or expired.".to_string(),
            ));
        }

        // Extract SNlM0e using regex
        let re = Regex::new(SNLM0E_PATTERN).unwrap();
        match re.captures(&text) {
            Some(caps) => Ok(caps.get(1).unwrap().as_str().to_string()),
            None => {
                if text.contains("429") {
                    Err(Error::Parse(
                        "SNlM0e not found. Rate limit likely exceeded.".to_string(),
                    ))
                } else {
                    Err(Error::Parse(
                        "SNlM0e value not found in response. Check cookie validity.".to_string(),
                    ))
                }
            }
        }
    }

    /// Rotates the __Secure-1PSIDTS cookie.
    async fn rotate_cookies(&mut self) -> Result<Option<String>> {
        let response = self
            .client
            .post(Endpoint::RotateCookies.url())
            .headers(rotate_cookies_headers())
            .body(r#"[000,"-0000000000000000000"]"#)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        // Check for new cookie in response
        // Note: Reqwest's cookie store automatically handles Set-Cookie headers for the client
        // But we want to update our struct field too
        for cookie in response.cookies() {
            if cookie.name() == "__Secure-1PSIDTS" {
                let new_value = cookie.value().to_string();
                self.secure_1psidts = new_value.clone();
                return Ok(Some(new_value));
            }
        }

        Ok(None)
    }

    /// Sends a message and returns the parsed response.
    ///
    /// # Arguments
    /// * `message` - The message text to send
    /// * `image` - Optional image bytes to upload and attach
    ///
    /// # Returns
    /// A [`ChatResponse`] containing the reply and metadata.
    ///
    /// # Errors
    /// Returns an error if the client is not initialized (`Error::NotInitialized`),
    /// if the image upload fails (`Error::Upload`), if the network request fails
    /// (`Error::Network`), or if the backend response cannot be parsed
    /// (`Error::Parse`).
    ///
    /// There is no retry or polling behavior. Timeouts are handled by `reqwest`
    /// and surface as `Error::Network`. External failures (Gemini backend,
    /// upload endpoint, or connectivity issues) are returned as errors.
    pub async fn ask(&mut self, message: &str, image: Option<&[u8]>) -> Result<ChatResponse> {
        if self.snlm0e.is_empty() {
            return Err(Error::NotInitialized(
                "AsyncChatbot not properly initialized. SNlM0e is missing.".to_string(),
            ));
        }

        // Handle image upload if provided
        let image_upload_id = if let Some(img_data) = image {
            Some(upload_file(img_data, self.proxy.as_deref()).await?)
        } else {
            None
        };

        // Prepare message structure
        let message_struct: Value = if let Some(ref upload_id) = image_upload_id {
            serde_json::json!([
                [message],
                [[[upload_id, 1]]],
                [&self.conversation_id, &self.response_id, &self.choice_id]
            ])
        } else {
            serde_json::json!([
                [message],
                null,
                [&self.conversation_id, &self.response_id, &self.choice_id]
            ])
        };

        // Prepare request
        let freq_value = serde_json::json!([null, serde_json::to_string(&message_struct)?]);
        let params = [
            ("bl", "boq_assistant-bard-web-server_20240625.13_p0"),
            ("_reqid", &self.reqid.to_string()),
            ("rt", "c"),
        ];

        let form_data = [
            ("f.req", serde_json::to_string(&freq_value)?),
            ("at", self.snlm0e.clone()),
        ];

        let response = self
            .client
            .post(Endpoint::Generate.url())
            .query(&params)
            .form(&form_data)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Network(response.error_for_status().unwrap_err()));
        }

        let text = response.text().await?;
        self.parse_response(&text)
    }

    /// Parses the Gemini API response text.
    fn parse_response(&mut self, text: &str) -> Result<ChatResponse> {
        let lines: Vec<&str> = text.lines().collect();
        if lines.len() < 3 {
            return Err(Error::Parse(format!(
                "Unexpected response format. Content: {}...",
                &text[..text.len().min(200)]
            )));
        }

        // Find the main response body
        let mut body: Option<Value> = None;

        for line in &lines {
            // Skip empty lines and security prefix
            if line.is_empty() || *line == ")]}" {
                continue;
            }

            let mut clean_line = *line;
            if clean_line.starts_with(")]}") {
                clean_line = clean_line.get(4..).unwrap_or("").trim();
            }

            if !clean_line.starts_with('[') {
                continue;
            }

            if let Ok(response_json) = serde_json::from_str::<Value>(clean_line) {
                if let Some(arr) = response_json.as_array() {
                    for part in arr {
                        if let Some(part_arr) = part.as_array() {
                            if part_arr.len() > 2
                                && part_arr.first().and_then(|v| v.as_str()) == Some("wrb.fr")
                            {
                                if let Some(inner_str) = part_arr.get(2).and_then(|v| v.as_str()) {
                                    if let Ok(main_part) = serde_json::from_str::<Value>(inner_str)
                                    {
                                        if main_part
                                            .as_array()
                                            .map(|a| a.len() > 4 && !a[4].is_null())
                                            .unwrap_or(false)
                                        {
                                            body = Some(main_part);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if body.is_some() {
                    break;
                }
            }
        }

        let body = body.ok_or_else(|| {
            Error::Parse("Failed to parse response body. No valid data found.".to_string())
        })?;

        // Extract data
        let body_arr = body.as_array().unwrap();

        // Extract content
        // Structure: body[4][0][1][0] -> content
        let content = body_arr
            .get(4)
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_array())
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract conversation metadata
        let conversation_id = body_arr
            .get(1)
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or(&self.conversation_id)
            .to_string();

        let response_id = body_arr
            .get(1)
            .and_then(|v| v.as_array())
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_str())
            .unwrap_or(&self.response_id)
            .to_string();

        // Extract other data
        let factuality_queries = body_arr.get(3).cloned();
        let text_query = body_arr
            .get(2)
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract choices
        let mut choices = Vec::new();
        if let Some(candidates) = body_arr.get(4).and_then(|v| v.as_array()) {
            for candidate in candidates {
                if let Some(cand_arr) = candidate.as_array() {
                    if cand_arr.len() > 1 {
                        let id = cand_arr
                            .first()
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let choice_content = cand_arr
                            .get(1)
                            .and_then(|v| v.as_array())
                            .and_then(|a| a.first())
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        choices.push(Choice {
                            id,
                            content: choice_content,
                        });
                    }
                }
            }
        }

        let choice_id = choices
            .first()
            .map(|c| c.id.clone())
            .unwrap_or_else(|| self.choice_id.clone());

        // Update state
        self.conversation_id = conversation_id.clone();
        self.response_id = response_id.clone();
        self.choice_id = choice_id;
        self.reqid += rand::thread_rng().gen_range(1000..9000);

        Ok(ChatResponse {
            content,
            conversation_id,
            response_id,
            factuality_queries,
            text_query,
            choices,
            error: false,
        })
    }

    /// Saves the current conversation state to a JSON file.
    ///
    /// The file format is a JSON array of [`SavedConversation`] values. If an
    /// entry with the same `conversation_name` already exists, it is replaced.
    pub async fn save_conversation(&self, file_path: &str, conversation_name: &str) -> Result<()> {
        let mut conversations = self.load_conversations(file_path).await?;

        let conversation_data = SavedConversation {
            conversation_name: conversation_name.to_string(),
            reqid: self.reqid,
            conversation_id: self.conversation_id.clone(),
            response_id: self.response_id.clone(),
            choice_id: self.choice_id.clone(),
            snlm0e: self.snlm0e.clone(),
            model_name: self.model.name().to_string(),
            timestamp: chrono_now(),
        };

        // Update or add conversation
        let mut found = false;
        for conv in &mut conversations {
            if conv.conversation_name == conversation_name {
                *conv = conversation_data.clone();
                found = true;
                break;
            }
        }
        if !found {
            conversations.push(conversation_data);
        }

        // Ensure parent directory exists
        if let Some(parent) = Path::new(file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&conversations)?;
        std::fs::write(file_path, json)?;

        Ok(())
    }

    /// Loads all saved conversations from a JSON file.
    ///
    /// If the file does not exist, this returns an empty vector.
    pub async fn load_conversations(&self, file_path: &str) -> Result<Vec<SavedConversation>> {
        if !Path::new(file_path).exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(file_path)?;
        let conversations: Vec<SavedConversation> = serde_json::from_str(&content)?;
        Ok(conversations)
    }

    /// Loads a specific conversation by name and applies it to this client.
    ///
    /// If the saved model name is unrecognized, the current model is left
    /// unchanged.
    ///
    /// Returns `true` if the conversation was found and loaded.
    pub async fn load_conversation(
        &mut self,
        file_path: &str,
        conversation_name: &str,
    ) -> Result<bool> {
        let conversations = self.load_conversations(file_path).await?;

        for conv in conversations {
            if conv.conversation_name == conversation_name {
                self.reqid = conv.reqid;
                self.conversation_id = conv.conversation_id;
                self.response_id = conv.response_id;
                self.choice_id = conv.choice_id;
                self.snlm0e = conv.snlm0e;

                if let Some(model) = Model::from_name(&conv.model_name) {
                    self.model = model;
                }

                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Returns the current conversation ID.
    pub fn conversation_id(&self) -> &str {
        &self.conversation_id
    }

    /// Returns the current model configuration.
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// Resets conversation IDs to start a fresh session.
    ///
    /// Authentication (cookies and session token) is preserved.
    pub fn reset(&mut self) {
        self.conversation_id.clear();
        self.response_id.clear();
        self.choice_id.clear();
        self.reqid = rand::thread_rng().gen_range(1000000..9999999);
    }
}

/// Simple timestamp function (avoids adding chrono dependency).
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
