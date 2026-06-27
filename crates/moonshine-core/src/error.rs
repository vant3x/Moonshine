use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum MoonshineError {
    #[error("Prefix not found: {0}")]
    PrefixNotFound(String),

    #[error("Prefix already exists: {0}")]
    PrefixAlreadyExists(String),

    #[error("Wine not found at: {0}")]
    WineNotFound(PathBuf),

    #[error("GPTK not installed. Download from Apple Developer and run setup.")]
    GptkNotInstalled,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("PE parse error: {0}")]
    PeParse(String),

    #[error("Wine process failed with exit code: {0:?}")]
    WineProcessFailed(Option<i32>),

    #[error("Runtime download failed: {0}")]
    DownloadFailed(String),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Config error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, MoonshineError>;
