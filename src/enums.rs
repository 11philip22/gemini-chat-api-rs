//! Enums and constants for Gemini endpoints, headers, and models.

use reqwest::header::{
    HeaderMap, HeaderName, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, ORIGIN, REFERER,
    USER_AGENT,
};

/// Gemini web endpoints used by this crate.
#[derive(Debug, Clone, Copy)]
pub enum Endpoint {
    /// Initialize session and obtain the `SNlM0e` token.
    Init,
    /// Generate a chat response.
    Generate,
    /// Rotate authentication cookies.
    RotateCookies,
    /// Upload files/images.
    Upload,
}

impl Endpoint {
    /// Returns the URL for this endpoint.
    pub fn url(&self) -> &'static str {
        match self {
            Endpoint::Init => "https://gemini.google.com/app",
            Endpoint::Generate => "https://gemini.google.com/_/BardChatUi/data/assistant.lamda.BardFrontendService/StreamGenerate",
            Endpoint::RotateCookies => "https://accounts.google.com/RotateCookies",
            Endpoint::Upload => "https://content-push.googleapis.com/upload",
        }
    }
}

/// Returns default headers used by chat requests.
///
/// These headers mimic a modern browser; the web endpoint may reject requests
/// without them.
pub fn gemini_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/x-www-form-urlencoded;charset=utf-8"),
    );

    headers.insert(
        ORIGIN,
        HeaderValue::from_static("https://gemini.google.com"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://gemini.google.com/"),
    );
    headers.insert(
        HeaderName::from_static("x-same-domain"),
        HeaderValue::from_static("1"),
    );
    // Chrome-like User-Agent (critical for authentication)
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
    );
    // Browser headers
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
    headers.insert(
        HeaderName::from_static("sec-ch-ua"),
        HeaderValue::from_static(
            "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\", \"Google Chrome\";v=\"120\"",
        ),
    );
    headers.insert(
        HeaderName::from_static("sec-ch-ua-mobile"),
        HeaderValue::from_static("?0"),
    );
    headers.insert(
        HeaderName::from_static("sec-ch-ua-platform"),
        HeaderValue::from_static("\"Windows\""),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-dest"),
        HeaderValue::from_static("empty"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-mode"),
        HeaderValue::from_static("cors"),
    );
    headers.insert(
        HeaderName::from_static("sec-fetch-site"),
        HeaderValue::from_static("same-origin"),
    );
    headers
}

/// Returns headers for cookie rotation requests.
pub fn rotate_cookies_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}

/// Returns headers for file upload requests.
///
/// The `push-id` value is hard-coded to match the web endpoint's expectations.
pub fn upload_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("push-id"),
        HeaderValue::from_static("feeds/mcudyrk2a4khkz"),
    );
    headers
}

/// Available model configurations.
///
/// This enum controls both the model name and the extra header value used by
/// the web endpoint.
#[derive(Debug, Clone, Default)]
pub enum Model {
    /// Unspecified model - uses default.
    #[default]
    Unspecified,
    /// Gemini 2.0 Flash.
    G2_0Flash,
    /// Gemini 2.0 Flash Thinking.
    G2_0FlashThinking,
    /// Gemini 2.5 Flash.
    G2_5Flash,
    /// Gemini 2.5 Pro.
    G2_5Pro,
    /// Gemini 2.0 Experimental Advanced (requires advanced subscription).
    G2_0ExpAdvanced,
    /// Gemini 2.5 Experimental Advanced (requires advanced subscription).
    G2_5ExpAdvanced,
    /// Gemini 3.0 Pro.
    G3_0Pro,
    /// Gemini 3.0 Flash.
    G3_0Flash,
    /// Gemini 3.0 Flash Thinking.
    G3_0Thinking,
}

impl Model {
    /// Returns the model name string used in saved conversation data.
    pub fn name(&self) -> &'static str {
        match self {
            Model::Unspecified => "unspecified",
            Model::G2_0Flash => "gemini-2.0-flash",
            Model::G2_0FlashThinking => "gemini-2.0-flash-thinking",
            Model::G2_5Flash => "gemini-2.5-flash",
            Model::G2_5Pro => "gemini-2.5-pro",
            Model::G2_0ExpAdvanced => "gemini-2.0-exp-advanced",
            Model::G2_5ExpAdvanced => "gemini-2.5-exp-advanced",
            Model::G3_0Pro => "gemini-3.0-pro",
            Model::G3_0Flash => "gemini-3.0-flash",
            Model::G3_0Thinking => "gemini-3.0-flash-thinking",
        }
    }

    /// Returns model-specific headers for the web endpoint.
    ///
    /// The header key is `x-goog-ext-525001261-jspb`. `Model::Unspecified`
    /// returns `None`, which means no extra model header is added.
    pub fn headers(&self) -> Option<HeaderMap> {
        let header_value = match self {
            Model::Unspecified => return None,
            Model::G2_0Flash => r#"[1,null,null,null,"f299729663a2343f"]"#,
            Model::G2_0FlashThinking => r#"[null,null,null,null,"7ca48d02d802f20a"]"#,
            Model::G2_5Flash => r#"[1,null,null,null,"35609594dbe934d8"]"#,
            Model::G2_5Pro => r#"[1,null,null,null,"2525e3954d185b3c"]"#,
            Model::G2_0ExpAdvanced => r#"[null,null,null,null,"b1e46a6037e6aa9f"]"#,
            Model::G2_5ExpAdvanced => r#"[null,null,null,null,"203e6bb81620bcfe"]"#,
            Model::G3_0Pro => {
                r#"[1,null,null,null,"e6fa609c3fa255c0",null,null,0,[4],null,null,2]"#
            }
            Model::G3_0Thinking => {
                r#"[1,null,null,null,"e051ce1aa80aa576",null,null,0,[4],null,null,2]"#
            }
            Model::G3_0Flash => {
                r#"[1,null,null,null,"56fdd199312815e2",null,null,0,[4],null,null,2]"#
            }
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-goog-ext-525001261-jspb"),
            HeaderValue::from_str(header_value).unwrap(),
        );
        Some(headers)
    }

    /// Returns `true` if this model requires an advanced subscription.
    pub fn is_advanced_only(&self) -> bool {
        matches!(self, Model::G2_0ExpAdvanced | Model::G2_5ExpAdvanced)
    }

    /// Creates a model from a name string.
    ///
    /// Accepted names are:
    /// - `unspecified`
    /// - `gemini-2.0-flash`
    /// - `gemini-2.0-flash-thinking`
    /// - `gemini-2.5-flash`
    /// - `gemini-2.5-pro`
    /// - `gemini-2.0-exp-advanced`
    /// - `gemini-2.5-exp-advanced`
    /// - `gemini-3.0-pro`
    /// - `gemini-3.0-flash`
    /// - `gemini-3.0-thinking`
    ///
    /// Note that `gemini-3.0-thinking` maps to [`Model::G3_0Thinking`], while
    /// [`Model::name`] for that variant returns `gemini-3.0-flash-thinking`.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "unspecified" => Some(Model::Unspecified),
            "gemini-2.0-flash" => Some(Model::G2_0Flash),
            "gemini-2.0-flash-thinking" => Some(Model::G2_0FlashThinking),
            "gemini-2.5-flash" => Some(Model::G2_5Flash),
            "gemini-2.5-pro" => Some(Model::G2_5Pro),
            "gemini-2.0-exp-advanced" => Some(Model::G2_0ExpAdvanced),
            "gemini-2.5-exp-advanced" => Some(Model::G2_5ExpAdvanced),
            "gemini-3.0-pro" => Some(Model::G3_0Pro),
            "gemini-3.0-flash" => Some(Model::G3_0Flash),
            "gemini-3.0-thinking" => Some(Model::G3_0Thinking),
            _ => None,
        }
    }
}
