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
