use thiserror::Error;

#[derive(Error, Debug)]
pub enum MeicanError {
    #[error("Not logged in. Run `meican login` first.")]
    NotLoggedIn,

    #[error("Login failed: {0}")]
    LoginFailed(String),

    #[error("API error (HTTP {status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[allow(dead_code)]
    #[error("{0}")]
    Other(String),
}
