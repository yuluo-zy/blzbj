use crate::TError;

#[derive(Debug, TError)]
pub enum LiveError {
    #[error("HTTP request failed")]
    HttpRequestError(#[from] reqwest::Error),
    #[error("JSON deserialization failed")]
    JsonError(#[from] serde_json::Error),
    #[error("No stream available")]
    NoStreamAvailable,
    #[error("No stream format available")]
    NoStreamFormatAvailable,
    #[error("No stream codec available")]
    NoStreamCodecAvailable,
    #[error("No stream quality available")]
    NoStreamQualityAvailable,
    #[error("Live room is hidden")]
    LiveRoomHidden,
    #[error("Live room is locked")]
    LiveRoomLocked,
    #[error("Live room is encrypted")]
    LiveRoomEncrypted,
    #[error("Invalid room info response")]
    InvalidRoomInfoResponse,
    #[error("Cannot extract info from HTML page")]
    CannotExtractInfo,
}

#[derive(Debug, TError)]
pub enum ApiRequestError {
    #[error("HTTP request failed")]
    HttpRequestError(#[from] reqwest::Error),
    #[error("JSON deserialization failed")]
    JsonError(#[from] serde_json::Error),
    #[error("API request error: code {0}, message {1}")]
    ApiError(i32, String),
    #[error("No base URLs provided")]
    NoBaseUrls,
}