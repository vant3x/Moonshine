use crate::config::{BottleConfig, GraphicsBackend, SyncMode};
use crate::error::{Result, MoonshineError};
use crate::prefix::Prefix;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Output};

#[derive(Debug)]
pub struct WineRunner {
    wine_bin: PathBuf,
}

impl WineRunner {
    pub fn new(wine_bin: PathBuf) -> Self {
        Self { wine_bin }
    }

    pub fn wine_bin_path(&self) -> &PathBuf {
        &self.wine_bin
    }

    pub fn detect() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| {
            MoonshineError::Config("Could not determine home directory".to_string())
        })?;

        let candidates = vec![
            home.join("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64"),
            PathBuf::from("/opt/homebrew/bin/wine64"),
            PathBuf::from("/usr/local/bin/wine64"),
            PathBuf::from("/Applications/Wine Stable.app/Contents/Resources/wine/bin/wine64"),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(Self::new(candidate));
            }
        }

        Err(MoonshineError::WineNotFound(
            home.join("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64"),
        ))
    }

    pub fn build_env(prefix: &Prefix, config: &BottleConfig) -> HashMap<String, String> {
        let mut env = HashMap::new();

        env.insert("WINEPREFIX".to_string(), prefix.path.to_string_lossy().to_string());

        match config.sync_mode {
            SyncMode::ESync => {
                env.insert("WINEESYNC".to_string(), "1".to_string());
            }
            SyncMode::MSync => {
                env.insert("WINEMSYNC".to_string(), "1".to_string());
            }
            SyncMode::Default => {}
        }

        if config.enable_metal_fx {
            env.insert("D3DM_ENABLE_METALFX".to_string(), "1".to_string());
        }

        if config.enable_dxvk_hud {
            env.insert("DXVK_HUD".to_string(), "fps,frametimes,devinfo".to_string());
        }

        if !config.dll_overrides.is_empty() {
            let overrides: Vec<String> = config
                .dll_overrides
                .iter()
                .map(|(key, value)| format!("{}={}", key, value))
                .collect();
            env.insert("WINEDLLOVERRIDES".to_string(), overrides.join(";"));
        }

        for (key, value) in &config.env_vars {
            env.insert(key.clone(), value.clone());
        }

        env
    }

    pub fn run_program(&self, prefix: &Prefix, program_path: &PathBuf) -> Result<Output> {
        let env = Self::build_env(prefix, &prefix.config);

        let mut cmd = Command::new(&self.wine_bin);
        cmd.arg("start");
        cmd.arg("/unix");
        cmd.arg(program_path);

        for (key, value) in &env {
            cmd.env(key, value);
        }

        cmd.output()
            .map_err(|e| MoonshineError::Io(e))
    }

    pub fn init_prefix(&self, prefix: &Prefix) -> Result<Output> {
        let env = Self::build_env(prefix, &prefix.config);

        let mut cmd = Command::new(&self.wine_bin);
        cmd.arg("wineboot");
        cmd.arg("-u");

        for (key, value) in &env {
            cmd.env(key, value);
        }

        cmd.output()
            .map_err(|e| MoonshineError::Io(e))
    }

    pub fn set_windows_version(&self, prefix: &Prefix, version: &str) -> Result<Output> {
        let env = Self::build_env(prefix, &prefix.config);

        let mut cmd = Command::new(&self.wine_bin);
        cmd.arg("reg");
        cmd.arg("add");
        cmd.arg("HKEY_CURRENT_USER\\Software\\Wine");
        cmd.arg("/v");
        cmd.arg("Version");
        cmd.arg("/t");
        cmd.arg("REG_SZ");
        cmd.arg("/d");
        cmd.arg(version);
        cmd.arg("/f");

        for (key, value) in &env {
            cmd.env(key, value);
        }

        cmd.output()
            .map_err(|e| MoonshineError::Io(e))
    }

    pub fn wine_version(&self) -> Result<String> {
        let output = Command::new(&self.wine_bin)
            .arg("--version")
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
