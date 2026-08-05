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
