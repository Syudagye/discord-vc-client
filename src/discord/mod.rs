use thiserror::Error;

pub mod guilds;
pub mod login;

pub const API_ENDPOINT: &str = "https://discord.com/api/v10";

/// Creates url for a given endpoint
pub fn endpoint(endpoint: &str) -> String {
    format!("{}{}", API_ENDPOINT, endpoint)
}

#[derive(Debug, Error)]
pub enum Error {
    // API errors
    #[error("Operation not permitted")]
    Unauthorized,

    // Login errors
    #[error("Login failed")]
    UserLoginFailed,
    #[error("The token \"{0}\" is invalid")]
    InvalidTOTP(String),

    // Other crates errors
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
