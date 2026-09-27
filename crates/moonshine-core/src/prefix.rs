use crate::config::{BottleConfig, WindowsArchitecture};
use crate::error::{Result, MoonshineError};
use crate::game::GameProfile;
use crate::process::ProgramRequest;
use crate::runtime::RuntimeManager;
use std::fs;
use std::path::{Path, PathBuf};
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
        validate_prefix_name(name)?;
        if prefix_name_exists(name, base_dir)? {
            return Err(MoonshineError::PrefixAlreadyExists(name.to_string()));
        }
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

    pub fn new_with_runtime(name: &str, base_dir: &PathBuf, mut config: BottleConfig) -> Result<Self> {
        validate_prefix_name(name)?;
        if prefix_name_exists(name, base_dir)? {
            return Err(MoonshineError::PrefixAlreadyExists(name.to_string()));
        }
        let runtime_path = RuntimeManager::selected_path(&config)?;
        config.name = name.to_string();
        config.wine_path = Some(runtime_path.to_string_lossy().to_string());
        Self::new_with_config(name, base_dir, config)
    }

    fn new_with_config(name: &str, base_dir: &PathBuf, config: BottleConfig) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let prefix_dir = base_dir.join(&id);
        fs::create_dir_all(&prefix_dir)?;
        fs::create_dir_all(prefix_dir.join("drive_c/Program Files"))?;
        fs::create_dir_all(prefix_dir.join("drive_c/users"))?;
        config.save(&prefix_dir)?;
        Ok(Self { id, name: name.to_string(), path: prefix_dir, config })
    }

    pub fn load(id: &str, base_dir: &PathBuf) -> Result<Self> {
        if id.is_empty() || id == "." || id == ".." || id.contains('/') || id.contains('\\') {
            return Err(MoonshineError::InvalidPath(base_dir.join(id)));
        }
        let prefix_dir = base_dir.join(id);

        if !prefix_dir.exists() {
            return Err(MoonshineError::PrefixNotFound(id.to_string()));
        }
        if !prefix_dir.is_dir() || !prefix_dir.join("drive_c").is_dir() {
            return Err(MoonshineError::PrefixCorrupt(id.to_string()));
        }

        let config = BottleConfig::load(&prefix_dir)
            .map_err(|_| MoonshineError::PrefixCorrupt(id.to_string()))?;

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
        if self.path.parent().is_none()
            || self.path.file_name().map(|name| name == "").unwrap_or(true)
            || self.path.symlink_metadata().map(|metadata| metadata.file_type().is_symlink()).unwrap_or(false)
        {
            return Err(MoonshineError::InvalidPath(self.path.clone()));
        }
        if self.path.exists() {
            fs::remove_dir_all(&self.path)?;
        }
        Ok(())
    }

    pub fn update_config(&mut self, config: BottleConfig) -> Result<()> {
        validate_architecture(&config)?;
        self.config = config;
        self.config.save(&self.path)?;
        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        self.config.save(&self.path)
    }

    pub fn set_architecture(&mut self, architecture: WindowsArchitecture) -> Result<()> {
        let mut config = self.config.clone();
        config.architecture = architecture;
        validate_architecture(&config)?;
        self.update_config(config)
    }

    pub fn backup_to(&self, backup_root: &Path) -> Result<PathBuf> {
        let backup_path = backup_root.join(&self.id);
        if backup_root.starts_with(&self.path) || backup_path.starts_with(&self.path) {
            return Err(MoonshineError::InvalidPath(backup_path));
        }
        if backup_path.exists() {
            return Err(MoonshineError::PrefixAlreadyExists(backup_path.display().to_string()));
        }
        copy_tree(&self.path, &backup_path)?;
        Ok(backup_path)
    }

    pub fn restore_from(&self, backup_path: &Path) -> Result<()> {
        if backup_path.starts_with(&self.path) {
            return Err(MoonshineError::InvalidPath(backup_path.to_path_buf()));
        }
        let restored = backup_path.join("bottle.json");
        if !restored.is_file() || !backup_path.join("drive_c").is_dir() {
            return Err(MoonshineError::PrefixCorrupt(backup_path.display().to_string()));
        }
        let config = BottleConfig::load(&backup_path.to_path_buf())?;
        validate_prefix_name(&config.name)?;
        let staging = self.path.with_extension("restore-tmp");
        if staging.exists() {
            fs::remove_dir_all(&staging)?;
        }
        copy_tree(backup_path, &staging)?;
        if self.path.exists() {
            fs::remove_dir_all(&self.path)?;
        }
        fs::rename(staging, &self.path)?;
        Ok(())
    }

    pub fn drive_c(&self) -> PathBuf {
        self.path.join("drive_c")
    }

    pub fn programs_dir(&self) -> PathBuf {
        self.drive_c().join("Program Files")
    }

    pub fn list_executables(&self) -> Result<Vec<PathBuf>> {
        let mut exes = Vec::new();

        // Scan all of drive_c, but skip system directories that contain
        // hundreds of irrelevant .exe files (system DLLs, installers, etc.)
        let drive_c = self.drive_c();
        let skip_dirs: &[&str] = &[
            "windows",       // System DLLs, regedit, notepad, etc.
            "users",          // User profile temp/installer cruft
            "ProgramData",    // Installer caches, package data
        ];

        for entry in walkdir::WalkDir::new(&drive_c)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    // Skip system directories at the drive_c root level
                    if let Some(name) = e.file_name().to_str() {
                        // Only skip at root level (depth 1 = direct children of drive_c)
                        if e.depth() == 1 {
                            return !skip_dirs.iter().any(|s| s.eq_ignore_ascii_case(name));
                        }
                    }
                }
                true
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();
            if path.extension().map_or(false, |ext| ext == "exe") {
                if is_steam_internal_executable(&path, &drive_c) {
                    continue;
                }
                exes.push(path);
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

    fn steam_install_roots(drive_c: &Path) -> [PathBuf; 2] {
        [
            drive_c.join("Program Files (x86)/Steam"),
            drive_c.join("Program Files/Steam"),
        ]
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

    pub fn program_request(&self, program_path: impl Into<PathBuf>) -> Result<ProgramRequest> {
        ProgramRequest::new(program_path)
    }

    pub fn game_profile(&self, program_path: impl Into<PathBuf>) -> Result<GameProfile> {
        GameProfile::new(&self.id, &program_path.into())
    }

    pub fn list_issues(base_dir: &Path) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        if !base_dir.exists() {
            return Ok(issues);
        }
        for entry in fs::read_dir(base_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let id = entry.file_name().to_string_lossy().to_string();
                if let Err(error) = Self::load(&id, &base_dir.to_path_buf()) {
                    issues.push(format!("{}: {}", id, error));
                }
            }
        }
        Ok(issues)
    }
}

fn is_steam_internal_executable(path: &Path, drive_c: &Path) -> bool {
    let is_steam_root = Prefix::steam_install_roots(drive_c)
        .iter()
        .any(|root| path.starts_with(root));
    if !is_steam_root {
        return false;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| !name.eq_ignore_ascii_case("steam.exe"))
        .unwrap_or(true)
}

fn validate_prefix_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || name.contains('/') || name.contains('\\') || name.contains('\0') {
        return Err(MoonshineError::InvalidPrefixName(name.to_string()));
    }
    Ok(())
}

fn prefix_name_exists(name: &str, base_dir: &Path) -> Result<bool> {
    if !base_dir.exists() {
        return Ok(false);
    }
    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Ok(config) = BottleConfig::load(&entry.path()) {
                if config.name == name {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

fn validate_architecture(config: &BottleConfig) -> Result<()> {
    if config.architecture == WindowsArchitecture::Win32 {
        let runner = crate::wine::WineRunner::detect_for_config(config)
            .map_err(|_| MoonshineError::Config("Win32 prefixes require a selected runtime".to_string()))?;
        if !runner.backend().has_wo64() {
            return Err(MoonshineError::Config("selected runtime does not support Win32 prefixes".to_string()));
        }
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            let target = fs::read_link(source_path)?;
            if target.is_absolute() || target.components().any(|component| component == std::path::Component::ParentDir) {
                return Err(MoonshineError::InvalidPath(target));
            }
            std::os::unix::fs::symlink(target, destination_path)?;
        } else if file_type.is_dir() {
            copy_tree(&source_path, &destination_path)?;
        } else {
            fs::copy(source_path, destination_path)?;
        }
    }
    Ok(())
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
    fn test_duplicate_prefix_name_is_rejected() {
        let dir = temp_dir();
        Prefix::new("Same Name", &dir).unwrap();
        assert!(matches!(Prefix::new("Same Name", &dir), Err(MoonshineError::PrefixAlreadyExists(_))));
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
    fn test_corrupt_prefix_is_rejected() {
        let dir = temp_dir();
        let corrupt = dir.join("corrupt");
        fs::create_dir_all(&corrupt).unwrap();
        fs::write(corrupt.join("bottle.json"), b"not json").unwrap();
        assert!(matches!(Prefix::load("corrupt", &dir), Err(MoonshineError::PrefixCorrupt(_))));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_backup_and_restore_preserve_prefix_files() {
        let dir = temp_dir();
        let backup_root = temp_dir();
        let prefix = Prefix::new("Backup", &dir).unwrap();
        let marker = prefix.drive_c().join("marker.txt");
        fs::write(&marker, b"original").unwrap();
        let backup = prefix.backup_to(&backup_root).unwrap();
        fs::write(&marker, b"changed").unwrap();

        prefix.restore_from(&backup).unwrap();
        assert_eq!(fs::read(marker).unwrap(), b"original");
        fs::remove_dir_all(dir).unwrap();
        fs::remove_dir_all(backup_root).unwrap();
    }

    #[test]
    fn test_invalid_runtime_is_rejected_without_fallback() {
        let dir = temp_dir();
        let mut config = BottleConfig::default();
        config.wine_path = Some(dir.join("missing/bin/wine64").to_string_lossy().to_string());
        assert!(Prefix::new_with_runtime("Invalid Runtime", &dir, config).is_err());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_prefix_load_rejects_path_traversal() {
        let dir = temp_dir();
        let result = Prefix::load("../outside", &dir);
        assert!(matches!(result, Err(MoonshineError::InvalidPath(_))));
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
    fn test_prefix_list_executables_scans_all_drive_c_and_skips_system() {
        let dir = temp_dir();
        let prefix = Prefix::new("TestBottle", &dir).unwrap();

        // Game installed in a custom location (not Program Files)
        fs::create_dir_all(prefix.drive_c().join("Games/MyGame")).unwrap();
        fs::write(prefix.drive_c().join("Games/MyGame/mygame.exe"), "").unwrap();

        let steam_dir = prefix.drive_c().join("Program Files (x86)/Steam");
        fs::create_dir_all(steam_dir.join("bin/cef")).unwrap();
        fs::write(steam_dir.join("steam.exe"), "").unwrap();
        fs::write(steam_dir.join("steamservice.exe"), "").unwrap();
        fs::write(steam_dir.join("bin/cef/steamwebhelper.exe"), "").unwrap();

        // Launcher installed in Program Files
        fs::create_dir_all(prefix.drive_c().join("Program Files (x86)/Epic/Launcher/Portal/Binaries/Win32")).unwrap();
        fs::write(prefix.drive_c().join("Program Files (x86)/Epic/Launcher/Portal/Binaries/Win32/EpicGamesLauncher.exe"), "").unwrap();

        // System .exe files that should be EXCLUDED
        fs::create_dir_all(prefix.drive_c().join("windows/system32")).unwrap();
        fs::write(prefix.drive_c().join("windows/system32/notepad.exe"), "").unwrap();
        fs::create_dir_all(prefix.drive_c().join("users/Public/AppData/Local/Temp")).unwrap();
        fs::write(prefix.drive_c().join("users/Public/AppData/Local/Temp/installer.exe"), "").unwrap();

        let exes = prefix.list_executables().unwrap();
        let names: Vec<String> = exes.iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        // Should find the game and launcher, but NOT system exes
        assert!(names.contains(&"mygame.exe".to_string()), "Should find game in custom dir");
        assert!(names.contains(&"EpicGamesLauncher.exe".to_string()), "Should find launcher in Program Files");
        assert!(names.contains(&"steam.exe".to_string()), "Should retain the Steam launcher");
        assert!(!names.contains(&"steamservice.exe".to_string()), "Should skip Steam services");
        assert!(!names.contains(&"steamwebhelper.exe".to_string()), "Should skip Steam helpers");
        assert!(!names.contains(&"notepad.exe".to_string()), "Should skip windows/ directory");
        assert!(!names.contains(&"installer.exe".to_string()), "Should skip users/ directory");

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
