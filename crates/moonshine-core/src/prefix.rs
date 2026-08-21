use crate::config::BottleConfig;
use crate::error::{Result, MoonshineError};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Prefix {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub config: BottleConfig,
}

impl Prefix {
    pub fn new(name: &str, base_dir: &PathBuf) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let prefix_dir = base_dir.join(&id);

        if prefix_dir.exists() {
            return Err(MoonshineError::PrefixAlreadyExists(name.to_string()));
        }

        fs::create_dir_all(&prefix_dir)?;
        fs::create_dir_all(prefix_dir.join("drive_c"))?;
        fs::create_dir_all(prefix_dir.join("drive_c/Program Files"))?;
        fs::create_dir_all(prefix_dir.join("drive_c/users"))?;

        let mut config = BottleConfig::default();
        config.name = name.to_string();
        config.save(&prefix_dir)?;

        Ok(Self {
            id,
            name: name.to_string(),
            path: prefix_dir,
            config,
        })
    }

    pub fn load(id: &str, base_dir: &PathBuf) -> Result<Self> {
        let prefix_dir = base_dir.join(id);

        if !prefix_dir.exists() {
            return Err(MoonshineError::PrefixNotFound(id.to_string()));
        }

        let config = BottleConfig::load(&prefix_dir)?;

        Ok(Self {
            id: id.to_string(),
            name: config.name.clone(),
            path: prefix_dir,
            config,
        })
    }

    pub fn list_all(base_dir: &PathBuf) -> Result<Vec<Self>> {
        let mut prefixes = Vec::new();

        if !base_dir.exists() {
            return Ok(prefixes);
        }

        for entry in fs::read_dir(base_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if let Ok(prefix) = Self::load(&dir_name, base_dir) {
                    prefixes.push(prefix);
                }
            }
        }

        prefixes.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(prefixes)
    }

    pub fn delete(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_dir_all(&self.path)?;
        }
        Ok(())
    }

    pub fn update_config(&mut self, config: BottleConfig) -> Result<()> {
        self.config = config;
        self.config.save(&self.path)?;
        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        self.config.save(&self.path)
    }

    pub fn drive_c(&self) -> PathBuf {
        self.path.join("drive_c")
    }

    pub fn programs_dir(&self) -> PathBuf {
        self.drive_c().join("Program Files")
    }

    pub fn list_executables(&self) -> Result<Vec<PathBuf>> {
        let mut exes = Vec::new();

        // Search in BOTH "Program Files" and "Program Files (x86)"
        let search_dirs = [
            self.drive_c().join("Program Files"),
            self.drive_c().join("Program Files (x86)"),
        ];

        for programs_dir in &search_dirs {
            if !programs_dir.exists() {
                continue;
            }

            for entry in walkdir::WalkDir::new(programs_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path().to_path_buf();
                if path.extension().map_or(false, |ext| ext == "exe") {
                    exes.push(path);
                }
            }
        }

        // Sort by name for consistent ordering
        exes.sort_by(|a, b| {
            let name_a = a.file_name().unwrap_or_default().to_string_lossy();
            let name_b = b.file_name().unwrap_or_default().to_string_lossy();
            name_a.cmp(&name_b)
        });

        Ok(exes)
    }

    /// Check if Steam is installed and return its path
    pub fn find_steam_exe(&self) -> Option<PathBuf> {
        let candidates = [
            self.drive_c().join("Program Files (x86)/Steam/steam.exe"),
            self.drive_c().join("Program Files/Steam/steam.exe"),
        ];
        for candidate in &candidates {
            if candidate.exists() {
                return Some(candidate.clone());
            }
        }
        None
    }
}

pub fn get_real_home() -> std::path::PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/Users/".to_string() + &std::env::var("USER").unwrap_or_default()));
    // If we're in a sandbox container, strip it to get the real home
    let home_str = home.to_string_lossy().to_string();
    if let Some(pos) = home_str.find("/Library/Containers/") {
        if let Some(_end) = home_str[pos..].find("/Data") {
            return std::path::PathBuf::from(&home_str[..pos]);
        }
    }
    home
}

pub fn get_base_dir() -> Result<PathBuf> {
    let home = get_real_home();
    let base = home.join("Library/Application Support/Moonshine/Prefixes");
    fs::create_dir_all(&base)?;
    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WindowsVersion;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("moonshine_prefix_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_prefix_new_creates_directories() {
        let dir = temp_dir();
        let prefix = Prefix::new("TestBottle", &dir).unwrap();

        assert!(!prefix.id.is_empty());
        assert_eq!(prefix.name, "TestBottle");
        assert!(prefix.path.exists());
        assert!(prefix.path.join("drive_c").exists());
        assert!(prefix.path.join("drive_c/Program Files").exists());
        assert!(prefix.path.join("drive_c/users").exists());
        assert!(prefix.path.join("bottle.json").exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_new_generates_unique_ids() {
        let dir = temp_dir();
        let prefix1 = Prefix::new("Bottle1", &dir).unwrap();
        let prefix2 = Prefix::new("Bottle2", &dir).unwrap();

        assert_ne!(prefix1.id, prefix2.id);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_load() {
        let dir = temp_dir();
        let created = Prefix::new("TestBottle", &dir).unwrap();
        let loaded = Prefix::load(&created.id, &dir).unwrap();

        assert_eq!(loaded.id, created.id);
        assert_eq!(loaded.name, "TestBottle");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_load_not_found() {
        let dir = temp_dir();
        let result = Prefix::load("nonexistent", &dir);
        assert!(result.is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_list_all() {
        let dir = temp_dir();
        Prefix::new("Bottle1", &dir).unwrap();
        Prefix::new("Bottle2", &dir).unwrap();

        let prefixes = Prefix::list_all(&dir).unwrap();
        assert_eq!(prefixes.len(), 2);

        let names: Vec<&str> = prefixes.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Bottle1"));
        assert!(names.contains(&"Bottle2"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_list_all_empty_dir() {
        let dir = temp_dir();
        let prefixes = Prefix::list_all(&dir).unwrap();
        assert!(prefixes.is_empty());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_list_all_nonexistent_dir() {
        let dir = PathBuf::from("/tmp/nonexistent_moonshine_test_dir");
        let prefixes = Prefix::list_all(&dir).unwrap();
        assert!(prefixes.is_empty());
    }

    #[test]
    fn test_prefix_delete() {
        let dir = temp_dir();
        let prefix = Prefix::new("TestBottle", &dir).unwrap();
        let prefix_path = prefix.path.clone();

        prefix.delete().unwrap();
        assert!(!prefix_path.exists());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_update_config() {
        let dir = temp_dir();
        let mut prefix = Prefix::new("TestBottle", &dir).unwrap();

        let mut new_config = prefix.config.clone();
        new_config.windows_version = WindowsVersion::Win11;

        prefix.update_config(new_config).unwrap();

        assert_eq!(prefix.config.windows_version, WindowsVersion::Win11);

        // Verify it persisted
        let loaded = Prefix::load(&prefix.id, &dir).unwrap();
        assert_eq!(loaded.config.windows_version, WindowsVersion::Win11);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_save_config() {
        let dir = temp_dir();
        let mut prefix = Prefix::new("TestBottle", &dir).unwrap();
        prefix.config.enable_metal_fx = false;
        prefix.save_config().unwrap();

        let loaded = Prefix::load(&prefix.id, &dir).unwrap();
        assert!(!loaded.config.enable_metal_fx);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_drive_c() {
        let dir = temp_dir();
        let prefix = Prefix::new("TestBottle", &dir).unwrap();
        assert_eq!(prefix.drive_c(), prefix.path.join("drive_c"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_programs_dir() {
        let dir = temp_dir();
        let prefix = Prefix::new("TestBottle", &dir).unwrap();
        assert_eq!(prefix.programs_dir(), prefix.path.join("drive_c/Program Files"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_list_executables_empty() {
        let dir = temp_dir();
        let prefix = Prefix::new("TestBottle", &dir).unwrap();
        let exes = prefix.list_executables().unwrap();
        assert!(exes.is_empty());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_find_steam_exe_none() {
        let dir = temp_dir();
        let prefix = Prefix::new("TestBottle", &dir).unwrap();
        assert!(prefix.find_steam_exe().is_none());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_prefix_find_steam_exe_found() {
        let dir = temp_dir();
        let prefix = Prefix::new("TestBottle", &dir).unwrap();

        // Create fake Steam directory
        let steam_dir = prefix.drive_c().join("Program Files (x86)/Steam");
        fs::create_dir_all(&steam_dir).unwrap();
        fs::write(steam_dir.join("steam.exe"), "").unwrap();

        assert!(prefix.find_steam_exe().is_some());

        fs::remove_dir_all(&dir).unwrap();
    }
}
