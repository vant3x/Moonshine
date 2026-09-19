use crate::error::Result;
use crate::process::ProgramRequest;
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameProfile {
    pub id: String,
    pub name: String,
    pub prefix_id: String,
    pub executable: String,
}

impl GameProfile {
    pub fn new(prefix_id: &str, executable: &Path) -> Result<Self> {
        let request = ProgramRequest::new(executable.to_path_buf())?;
        let name = request
            .executable
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Game")
            .to_string();
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            prefix_id: prefix_id.to_string(),
            executable: request.executable.to_string_lossy().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn game_profile_associates_executable_with_prefix() {
        let path = std::env::temp_dir().join(format!("moonshine-game-{}.exe", Uuid::new_v4()));
        fs::write(&path, b"game").unwrap();
        let profile = GameProfile::new("prefix-1", &path).unwrap();

        assert_eq!(profile.prefix_id, "prefix-1");
        assert_eq!(profile.name, path.file_stem().unwrap().to_string_lossy());
        fs::remove_file(path).unwrap();
    }
}