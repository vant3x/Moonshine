use crate::error::{MoonshineError, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramRequest {
    pub executable: PathBuf,
}

impl ProgramRequest {
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self> {
        let executable = executable.into();
        if !executable.is_absolute()
            || !executable.is_file()
            || executable.extension().is_none()
        {
            return Err(MoonshineError::InvalidPath(executable));
        }

        Ok(Self { executable })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessLaunchResult {
    pub pid: u32,
}

impl ProcessLaunchResult {
    pub fn new(pid: u32) -> Result<Self> {
        if pid == 0 {
            return Err(MoonshineError::Config("process returned an invalid PID".to_string()));
        }
        Ok(Self { pid })
    }
}

pub fn validate_path_is_file(path: &Path) -> Result<()> {
    if path.is_absolute() && path.is_file() {
        Ok(())
    } else {
        Err(MoonshineError::InvalidPath(path.to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn program_request_accepts_existing_absolute_file() {
        let path = std::env::temp_dir().join(format!("moonshine-program-{}.exe", uuid::Uuid::new_v4()));
        fs::write(&path, b"fixture").unwrap();

        assert_eq!(ProgramRequest::new(&path).unwrap().executable, path);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn program_request_rejects_missing_file() {
        let path = std::env::temp_dir().join("moonshine-missing.exe");
        assert!(matches!(ProgramRequest::new(path), Err(MoonshineError::InvalidPath(_))));
    }

    #[test]
    fn process_result_rejects_zero_pid() {
        assert!(ProcessLaunchResult::new(0).is_err());
    }
}