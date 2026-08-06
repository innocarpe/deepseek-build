use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API status {status}: {body}")]
    ApiStatus { status: u16, body: String },

    #[error("invalid SSE: {0}")]
    InvalidSse(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("missing API key")]
    MissingApiKey,

    #[error("stream interrupted: {0}")]
    StreamInterrupted(String),

    #[error("{0}")]
    Message(String),
}
