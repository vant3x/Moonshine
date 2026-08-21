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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_prefix_not_found() {
        let err = MoonshineError::PrefixNotFound("test".to_string());
        assert_eq!(err.to_string(), "Prefix not found: test");
    }

    #[test]
    fn test_error_display_prefix_already_exists() {
        let err = MoonshineError::PrefixAlreadyExists("test".to_string());
        assert_eq!(err.to_string(), "Prefix already exists: test");
    }

    #[test]
    fn test_error_display_wine_not_found() {
        let err = MoonshineError::WineNotFound(PathBuf::from("/usr/bin/wine"));
        assert_eq!(err.to_string(), "Wine not found at: /usr/bin/wine");
    }

    #[test]
    fn test_error_display_gptk_not_installed() {
        let err = MoonshineError::GptkNotInstalled;
        assert_eq!(err.to_string(), "GPTK not installed. Download from Apple Developer and run setup.");
    }

    #[test]
    fn test_error_display_pe_parse() {
        let err = MoonshineError::PeParse("invalid header".to_string());
        assert_eq!(err.to_string(), "PE parse error: invalid header");
    }

    #[test]
    fn test_error_display_wine_process_failed() {
        let err = MoonshineError::WineProcessFailed(Some(1));
        assert_eq!(err.to_string(), "Wine process failed with exit code: Some(1)");
    }

    #[test]
    fn test_error_display_download_failed() {
        let err = MoonshineError::DownloadFailed("timeout".to_string());
        assert_eq!(err.to_string(), "Runtime download failed: timeout");
    }

    #[test]
    fn test_error_display_invalid_path() {
        let err = MoonshineError::InvalidPath(PathBuf::from("/invalid"));
        assert_eq!(err.to_string(), "Invalid path: /invalid");
    }

    #[test]
    fn test_error_display_config() {
        let err = MoonshineError::Config("missing key".to_string());
        assert_eq!(err.to_string(), "Config error: missing key");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = MoonshineError::from(io_err);
        assert!(matches!(err, MoonshineError::Io(_)));
    }

    #[test]
    fn test_error_from_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err = MoonshineError::from(json_err);
        assert!(matches!(err, MoonshineError::Json(_)));
    }
}
