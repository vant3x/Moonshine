use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PeInfo {
    pub name: String,
    pub is_64bit: bool,
}

impl PeInfo {
    pub fn parse(path: &PathBuf) -> crate::error::Result<Self> {
        let buffer = fs::read(path)
            .map_err(|e| crate::error::MoonshineError::Io(e))?;

        let pe = goblin::pe::PE::parse(&buffer)
            .map_err(|e| crate::error::MoonshineError::PeParse(e.to_string()))?;

        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Ok(Self {
            name,
            is_64bit: pe.is_64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("moonshine_pe_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_pe_parse_nonexistent_file() {
        let path = PathBuf::from("/tmp/nonexistent.exe");
        let result = PeInfo::parse(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_pe_parse_invalid_pe() {
        let dir = temp_dir();
        let path = dir.join("notape.exe");
        fs::write(&path, b"this is not a PE file").unwrap();

        let result = PeInfo::parse(&path);
        assert!(result.is_err());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_pe_parse_empty_file() {
        let dir = temp_dir();
        let path = dir.join("empty.exe");
        fs::write(&path, b"").unwrap();

        let result = PeInfo::parse(&path);
        assert!(result.is_err());

        fs::remove_dir_all(&dir).unwrap();
    }
}
