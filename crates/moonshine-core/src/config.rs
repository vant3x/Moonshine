use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WindowsVersion {
    Win10,
    Win11,
}

impl Default for WindowsVersion {
    fn default() -> Self {
        Self::Win10
    }
}

impl std::fmt::Display for WindowsVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Win10 => write!(f, "win10"),
            Self::Win11 => write!(f, "win11"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GraphicsBackend {
    D3DMetal,
    DXVK,
}

impl Default for GraphicsBackend {
    fn default() -> Self {
        Self::D3DMetal
    }
}

impl std::fmt::Display for GraphicsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::D3DMetal => write!(f, "d3dmetal"),
            Self::DXVK => write!(f, "dxvk"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncMode {
    Default,
    ESync,
    MSync,
}

impl Default for SyncMode {
    fn default() -> Self {
        Self::Default
    }
}

impl std::fmt::Display for SyncMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::ESync => write!(f, "esync"),
            Self::MSync => write!(f, "msync"),
        }
    }
}

/// Wine backend configuration per prefix.
/// - Auto: automatically detect best backend
/// - WineHQ: use WineHQ (best for 32-bit apps, Steam, winetricks)
/// - GPTK: use Game Porting Toolkit (best for 64-bit gaming)
/// - CrossOver: use CrossOver (commercial, full WoW64)
/// - Custom: use custom wine_path
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WineBackendConfig {
    Auto,
    WineHQ,
    GPTK,
    CrossOver,
    Custom,
}

impl Default for WineBackendConfig {
    fn default() -> Self {
        Self::Auto
    }
}

impl std::fmt::Display for WineBackendConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::WineHQ => write!(f, "winehq"),
            Self::GPTK => write!(f, "gptk"),
            Self::CrossOver => write!(f, "crossover"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl WineBackendConfig {
    /// Does this backend have working WoW64 (32-bit exe support)?
    pub fn has_wo64(&self) -> bool {
        match self {
            Self::Auto => false, // depends on what's detected
            Self::WineHQ => true,
            Self::GPTK => false,
            Self::CrossOver => true,
            Self::Custom => false, // depends on custom path
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::Auto => "Auto (best available)",
            Self::WineHQ => "WineHQ (Homebrew)",
            Self::GPTK => "Game Porting Toolkit",
            Self::CrossOver => "CrossOver",
            Self::Custom => "Custom path",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleConfig {
    pub name: String,
    pub windows_version: WindowsVersion,
    pub graphics_backend: GraphicsBackend,
    pub sync_mode: SyncMode,
    pub enable_metal_fx: bool,
    pub enable_dxvk_hud: bool,
    pub dll_overrides: Vec<(String, String)>,
    pub env_vars: Vec<(String, String)>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wine_path: Option<String>,
    /// Which Wine backend to use for this prefix.
    /// Defaults to Auto (detect best available).
    #[serde(default)]
    pub wine_backend: WineBackendConfig,
    /// Enable HID controller passthrough for gamepads (GameSir, Xbox, PS5, etc.)
    /// Sets SDL_JOYSTICK_MFI=0 and enables XInput env vars so Wine can see the controller.
    #[serde(default = "default_true")]
    pub enable_hid_controllers: bool,
    /// Reduce WINEDEBUG output for better performance during gaming.
    #[serde(default = "default_true")]
    pub reduce_wine_debug: bool,
}

fn default_true() -> bool { true }

impl Default for BottleConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            windows_version: WindowsVersion::default(),
            graphics_backend: GraphicsBackend::default(),
            sync_mode: SyncMode::default(),
            enable_metal_fx: true,
            enable_dxvk_hud: false,
            dll_overrides: Vec::new(),
            env_vars: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            wine_path: None,
            wine_backend: WineBackendConfig::default(),
            enable_hid_controllers: true,
            reduce_wine_debug: true,
        }
    }
}

impl BottleConfig {
    pub fn config_path(prefix_dir: &PathBuf) -> PathBuf {
        prefix_dir.join("bottle.json")
    }

    pub fn save(&self, prefix_dir: &PathBuf) -> crate::error::Result<()> {
        let path = Self::config_path(prefix_dir);
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn load(prefix_dir: &PathBuf) -> crate::error::Result<Self> {
        let path = Self::config_path(prefix_dir);
        let json = std::fs::read_to_string(&path)?;
        let config: Self = serde_json::from_str(&json)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("moonshine_test_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_default_windows_version() {
        assert_eq!(WindowsVersion::default(), WindowsVersion::Win10);
    }

    #[test]
    fn test_windows_version_display() {
        assert_eq!(WindowsVersion::Win10.to_string(), "win10");
        assert_eq!(WindowsVersion::Win11.to_string(), "win11");
    }

    #[test]
    fn test_default_graphics_backend() {
        assert_eq!(GraphicsBackend::default(), GraphicsBackend::D3DMetal);
    }

    #[test]
    fn test_graphics_backend_display() {
        assert_eq!(GraphicsBackend::D3DMetal.to_string(), "d3dmetal");
        assert_eq!(GraphicsBackend::DXVK.to_string(), "dxvk");
    }

    #[test]
    fn test_default_sync_mode() {
        assert_eq!(SyncMode::default(), SyncMode::Default);
    }

    #[test]
    fn test_sync_mode_display() {
        assert_eq!(SyncMode::Default.to_string(), "default");
        assert_eq!(SyncMode::ESync.to_string(), "esync");
        assert_eq!(SyncMode::MSync.to_string(), "msync");
    }

    #[test]
    fn test_wine_backend_config_default() {
        assert_eq!(WineBackendConfig::default(), WineBackendConfig::Auto);
    }

    #[test]
    fn test_wine_backend_config_display() {
        assert_eq!(WineBackendConfig::Auto.to_string(), "auto");
        assert_eq!(WineBackendConfig::WineHQ.to_string(), "winehq");
        assert_eq!(WineBackendConfig::GPTK.to_string(), "gptk");
        assert_eq!(WineBackendConfig::CrossOver.to_string(), "crossover");
        assert_eq!(WineBackendConfig::Custom.to_string(), "custom");
    }

    #[test]
    fn test_wine_backend_config_has_wo64() {
        assert!(!WineBackendConfig::Auto.has_wo64());
        assert!(WineBackendConfig::WineHQ.has_wo64());
        assert!(!WineBackendConfig::GPTK.has_wo64());
        assert!(WineBackendConfig::CrossOver.has_wo64());
        assert!(!WineBackendConfig::Custom.has_wo64());
    }

    #[test]
    fn test_wine_backend_config_display_name() {
        assert_eq!(WineBackendConfig::Auto.display_name(), "Auto (best available)");
        assert_eq!(WineBackendConfig::WineHQ.display_name(), "WineHQ (Homebrew)");
        assert_eq!(WineBackendConfig::GPTK.display_name(), "Game Porting Toolkit");
        assert_eq!(WineBackendConfig::CrossOver.display_name(), "CrossOver");
        assert_eq!(WineBackendConfig::Custom.display_name(), "Custom path");
    }

    #[test]
    fn test_bottle_config_default() {
        let config = BottleConfig::default();
        assert!(config.name.is_empty());
        assert_eq!(config.windows_version, WindowsVersion::Win10);
        assert_eq!(config.graphics_backend, GraphicsBackend::D3DMetal);
        assert_eq!(config.sync_mode, SyncMode::Default);
        assert!(config.enable_metal_fx);
        assert!(!config.enable_dxvk_hud);
        assert!(config.dll_overrides.is_empty());
        assert!(config.env_vars.is_empty());
        assert!(config.wine_path.is_none());
        assert_eq!(config.wine_backend, WineBackendConfig::Auto);
        assert!(config.enable_hid_controllers);
        assert!(config.reduce_wine_debug);
    }

    #[test]
    fn test_bottle_config_save_and_load() {
        let dir = temp_dir();
        let mut config = BottleConfig::default();
        config.name = "TestBottle".to_string();
        config.windows_version = WindowsVersion::Win11;
        config.graphics_backend = GraphicsBackend::DXVK;

        config.save(&dir).unwrap();
        let loaded = BottleConfig::load(&dir).unwrap();

        assert_eq!(loaded.name, "TestBottle");
        assert_eq!(loaded.windows_version, WindowsVersion::Win11);
        assert_eq!(loaded.graphics_backend, GraphicsBackend::DXVK);

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_bottle_config_serialization_roundtrip() {
        let config = BottleConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BottleConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.windows_version, deserialized.windows_version);
        assert_eq!(config.graphics_backend, deserialized.graphics_backend);
    }

    #[test]
    fn test_bottle_config_path() {
        let prefix_dir = PathBuf::from("/tmp/test");
        assert_eq!(BottleConfig::config_path(&prefix_dir), PathBuf::from("/tmp/test/bottle.json"));
    }
}
