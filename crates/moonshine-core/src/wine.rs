use crate::config::{BottleConfig, GraphicsBackend, SyncMode, WineBackendConfig};
use crate::error::{Result, MoonshineError};
use crate::prefix::Prefix;
use std::collections::HashMap;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Wine backend type - determines WoW64 support and priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WineBackend {
    /// CrossOver - commercial, full WoW64 support
    CrossOver,
    /// WineHQ via Homebrew - free, WoW64 works in Wine 9+
    WineHQ,
    /// Whisky - free, uses CrossOver or vanilla Wine
    Whisky,
    /// GPTK - Apple's Game Porting Toolkit, limited WoW64
    GPTK,
    /// Wine Stable app
    WineStable,
    /// Unknown/manual path
    Unknown,
}

impl WineBackend {
    /// Does this backend have working WoW64 (32-bit exe support)?
    pub fn has_wo64(&self) -> bool {
        match self {
            Self::CrossOver => true,
            Self::WineHQ => true,
            Self::Whisky => true,
            Self::GPTK => false,  // GPTK WoW64 is incomplete
            Self::WineStable => true,
            Self::Unknown => false,
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::CrossOver => "CrossOver",
            Self::WineHQ => "WineHQ (Homebrew)",
            Self::Whisky => "Whisky",
            Self::GPTK => "Game Porting Toolkit",
            Self::WineStable => "Wine Stable",
            Self::Unknown => "Custom",
        }
    }
}

impl std::fmt::Display for WineBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Information about a detected Wine installation
#[derive(Debug, Clone)]
pub struct WineInfo {
    pub backend: WineBackend,
    pub path: PathBuf,
    pub version: Option<String>,
    pub has_wo64: bool,
}

#[derive(Debug)]
pub struct WineRunner {
    wine_bin: PathBuf,
    backend: WineBackend,
}

impl WineRunner {
    pub fn new(wine_bin: PathBuf) -> Self {
        Self {
            wine_bin,
            backend: WineBackend::Unknown,
        }
    }

    pub fn with_backend(wine_bin: PathBuf, backend: WineBackend) -> Self {
        Self { wine_bin, backend }
    }

    pub fn wine_bin_path(&self) -> &PathBuf {
        &self.wine_bin
    }

    pub fn backend(&self) -> &WineBackend {
        &self.backend
    }

    /// Detect the best available Wine backend.
    /// Priority: CrossOver > WineHQ > Whisky > GPTK > WineStable
    /// BUT: skip backends with the Wine 11.0 msvcrt bug
    pub fn detect() -> Result<Self> {
        let info = Self::detect_best()?;
        Ok(Self::with_backend(info.path, info.backend))
    }

    /// Detect wine, preferring the path configured in BottleConfig if set.
    pub fn detect_for_config(config: &BottleConfig) -> Result<Self> {
        // 1. If custom wine_path is set, use it
        if let Some(ref path) = config.wine_path {
            let pb = PathBuf::from(path);
            if pb.exists() {
                let backend = Self::detect_backend_for_path(&pb);
                return Ok(Self::with_backend(pb, backend));
            }
        }

        // 2. If a specific backend is configured, use it
        match &config.wine_backend {
            WineBackendConfig::WineHQ => {
                return Self::find_backend(WineBackend::WineHQ);
            }
            WineBackendConfig::GPTK => {
                return Self::find_backend(WineBackend::GPTK);
            }
            WineBackendConfig::CrossOver => {
                return Self::find_backend(WineBackend::CrossOver);
            }
            WineBackendConfig::Custom => {
                // Custom without wine_path - fall through to auto-detect
            }
            WineBackendConfig::Auto => {
                // Auto-detect best available
            }
        }

        // 3. Auto-detect best backend
        Self::detect()
    }

    /// Find a specific backend type
    fn find_backend(target: WineBackend) -> Result<Self> {
        let backends = Self::detect_all();
        for info in backends {
            if info.backend == target {
                return Ok(Self::with_backend(info.path, info.backend));
            }
        }
        Err(MoonshineError::WineNotFound(
            crate::prefix::get_real_home()
                .join("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64")
        ))
    }

    /// Find any backend that has WoW64 (32-bit) support.
    /// Priority: CrossOver > WineHQ > Whisky > WineStable
    pub fn find_backend_with_wo64() -> Result<Self> {
        let backends = Self::detect_all();
        for info in backends {
            if info.has_wo64 {
                tracing::debug!(backend = %info.backend, path = %info.path.display(), "Found WoW64-capable backend");
                return Ok(Self::with_backend(info.path, info.backend));
            }
        }
        Err(MoonshineError::WineNotFound(
            crate::prefix::get_real_home()
                .join("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64")
        ))
    }

    /// Detect all available Wine backends and return the best one.
    /// Skips backends with the Wine 11.0 msvcrt crash bug.
    pub fn detect_best() -> Result<WineInfo> {
        let backends = Self::detect_all();

        // Warn about Wine 11.0 msvcrt bug
        for info in &backends {
            if let Some(ref version) = info.version {
                if version.contains("wine-11.0") {
                    tracing::warn!(version = %version, "Wine has known msvcrt.dll crash bug on macOS ARM64");
                    tracing::info!("All wine commands will fail with: Unhandled exception 0xc0000417");
                    tracing::info!("Fix: brew install --cask wine@devel");
                    tracing::info!("Or install CrossOver: https://www.codeweavers.com");
                }
            }
        }

        // Prefer backends WITHOUT the msvcrt bug
        let healthy = backends.iter().find(|info| {
            if let Some(ref version) = info.version {
                if version.starts_with("wine-11.0") && !version.starts_with("wine-11.0.0") {
                    tracing::debug!(backend = %info.backend, version = %version, "Skipping backend - has msvcrt bug");
                    return false;
                }
            }
            true
        });

        if let Some(best) = healthy {
            tracing::debug!(backend = %best.backend, path = %best.path.display(), "Best backend selected");
            return Ok(best.clone());
        }

        // Fallback: return first available even if it has the bug
        backends.into_iter().next()
            .ok_or_else(|| {
                let home = crate::prefix::get_real_home();
                MoonshineError::WineNotFound(
                    home.join("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64")
                )
            })
    }

    /// Detect all available Wine backends, sorted by priority.
    pub fn detect_all() -> Vec<WineInfo> {
        let home = crate::prefix::get_real_home();
        let mut candidates: Vec<WineInfo> = Vec::new();

        // 1. CrossOver detection (highest priority - best WoW64)
        candidates.extend(Self::detect_crossover());

        // 2. WineHQ via Homebrew (free, good WoW64)
        candidates.extend(Self::detect_winehq());

        // 3. Whisky detection
        candidates.extend(Self::detect_whisky());

        // 4. GPTK (our bundled version)
        candidates.extend(Self::detect_gptk(&home));

        // 5. Wine Stable app
        candidates.extend(Self::detect_wine_stable());

        // 6. Homebrew wine64 (fallback)
        candidates.extend(Self::detect_homebrew());

        // Sort by priority (CrossOver > WineHQ > Whisky > GPTK > Unknown)
        candidates.sort_by(|a, b| a.backend.cmp(&b.backend));

        // Remove duplicates (same path)
        let mut seen = std::collections::HashSet::new();
        candidates.retain(|info| seen.insert(info.path.clone()));

        candidates
    }

    fn detect_crossover() -> Vec<WineInfo> {
        let mut results = Vec::new();

        // CrossOver.app Frameworks (older versions)
        let crossover_base = PathBuf::from("/Applications/CrossOver.app/Contents/Frameworks");
        if let Ok(entries) = std::fs::read_dir(&crossover_base) {
            for entry in entries.filter_map(|e| e.ok()) {
                let wine64 = entry.path().join("bin/wine64");
                if wine64.exists() {
                    let version = Self::get_version(&wine64);
                    results.push(WineInfo {
                        backend: WineBackend::CrossOver,
                        path: wine64,
                        version,
                        has_wo64: true,
                    });
                }
            }
        }

        // CrossOver.app Resources
        let crossover_res = PathBuf::from("/Applications/CrossOver.app/Contents/Resources/wine/bin/wine64");
        if crossover_res.exists() {
            let version = Self::get_version(&crossover_res);
            results.push(WineInfo {
                backend: WineBackend::CrossOver,
                path: crossover_res,
                version,
                has_wo64: true,
            });
        }

        // CrossOver.app SharedSupport (newer versions store wine here)
        // Path: /Applications/CrossOver.app/Contents/SharedSupport/CrossOver/lib/wine/x86_64-unix/wine
        let crossover_shared = PathBuf::from("/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/lib/wine/x86_64-unix/wine");
        if crossover_shared.exists() {
            let version = Self::get_version(&crossover_shared);
            results.push(WineInfo {
                backend: WineBackend::CrossOver,
                path: crossover_shared,
                version,
                has_wo64: true,
            });
        }

        // Also check for wine64 in SharedSupport
        let crossover_shared64 = PathBuf::from("/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/lib/wine/x86_64-unix/wine64");
        if crossover_shared64.exists() {
            let version = Self::get_version(&crossover_shared64);
            results.push(WineInfo {
                backend: WineBackend::CrossOver,
                path: crossover_shared64,
                version,
                has_wo64: true,
            });
        }

        // CrossOver "Hosted Application" wine wrapper
        let crossover_hosted = PathBuf::from("/Applications/CrossOver.app/Contents/SharedSupport/CrossOver/CrossOver-Hosted Application/wine");
        if crossover_hosted.exists() {
            let version = Self::get_version(&crossover_hosted);
            results.push(WineInfo {
                backend: WineBackend::CrossOver,
                path: crossover_hosted,
                version,
                has_wo64: true,
            });
        }

        results
    }

    fn detect_winehq() -> Vec<WineInfo> {
        let mut results = Vec::new();

        // Homebrew Cask wine-stable: Wine 11+ only ships `wine` (not `wine64`)
        // The `wine` binary handles both 32-bit and 64-bit in WoW64 mode
        let wine_stable_base = PathBuf::from("/Applications/Wine Stable.app/Contents/Resources/wine/bin");
        // Prefer wine64 if it exists (older WineHQ), else fall back to wine (Wine 11+)
        let wine_stable = if wine_stable_base.join("wine64").exists() {
            wine_stable_base.join("wine64")
        } else if wine_stable_base.join("wine").exists() {
            wine_stable_base.join("wine")
        } else {
            PathBuf::new()
        };
        if wine_stable.exists() {
            let version = Self::get_version(&wine_stable);
            results.push(WineInfo {
                backend: WineBackend::WineHQ,
                path: wine_stable,
                version,
                has_wo64: true,
            });
        }

        // wine@devel (Wine 11.1+ — msvcrt bug fixed)
        let wine_devel_base = PathBuf::from("/Applications/Wine Devel.app/Contents/Resources/wine/bin");
        let wine_devel = if wine_devel_base.join("wine64").exists() {
            wine_devel_base.join("wine64")
        } else if wine_devel_base.join("wine").exists() {
            wine_devel_base.join("wine")
        } else {
            PathBuf::new()
        };
        if wine_devel.exists() {
            let version = Self::get_version(&wine_devel);
            results.push(WineInfo {
                backend: WineBackend::WineHQ,
                path: wine_devel,
                version,
                has_wo64: true,
            });
        }

        // wine@staging (Wine 11.1+ staging — msvcrt bug fixed)
        let wine_staging_base = PathBuf::from("/Applications/Wine Staging.app/Contents/Resources/wine/bin");
        let wine_staging = if wine_staging_base.join("wine64").exists() {
            wine_staging_base.join("wine64")
        } else if wine_staging_base.join("wine").exists() {
            wine_staging_base.join("wine")
        } else {
            PathBuf::new()
        };
        if wine_staging.exists() {
            let version = Self::get_version(&wine_staging);
            results.push(WineInfo {
                backend: WineBackend::WineHQ,
                path: wine_staging,
                version,
                has_wo64: true,
            });
        }

        // Homebrew wine (opt path) - Wine 11+ symlinks to Wine Stable.app
        let homebrew_wine64 = PathBuf::from("/opt/homebrew/bin/wine64");
        let homebrew_wine = PathBuf::from("/opt/homebrew/bin/wine");
        let homebrew_path = if homebrew_wine64.exists() {
            homebrew_wine64
        } else if homebrew_wine.exists() {
            homebrew_wine
        } else {
            PathBuf::new()
        };
        if homebrew_path.exists() {
            let version = Self::get_version(&homebrew_path);
            results.push(WineInfo {
                backend: WineBackend::WineHQ,
                path: homebrew_path,
                version,
                has_wo64: true,
            });
        }

        // Homebrew wine (usr/local path - Intel Macs)
        let homebrew_local64 = PathBuf::from("/usr/local/bin/wine64");
        let homebrew_local = PathBuf::from("/usr/local/bin/wine");
        let homebrew_local_path = if homebrew_local64.exists() {
            homebrew_local64
        } else if homebrew_local.exists() {
            homebrew_local
        } else {
            PathBuf::new()
        };
        if homebrew_local_path.exists() {
            let version = Self::get_version(&homebrew_local_path);
            results.push(WineInfo {
                backend: WineBackend::WineHQ,
                path: homebrew_local_path,
                version,
                has_wo64: true,
            });
        }

        results
    }

    fn detect_whisky() -> Vec<WineInfo> {
        let mut results = Vec::new();
        let home = crate::prefix::get_real_home();

        // Whisky stores its Wine in ~/Library/Containers/com.isaacmarovitz.Whisky/Bottles/
        // or in ~/Library/Containers/com.isaacmarovitz.Whisky/
        let whisky_base = home.join("Library/Containers/com.isaacmarovitz.Whisky");
        if whisky_base.exists() {
            // Look for wine64 in various locations within Whisky
            let candidates = [
                whisky_base.join("Wine/bin/wine64"),
                whisky_base.join("Resources/Wine/bin/wine64"),
            ];

            for candidate in &candidates {
                if candidate.exists() {
                    let version = Self::get_version(candidate);
                    results.push(WineInfo {
                        backend: WineBackend::Whisky,
                        path: candidate.clone(),
                        version,
                        has_wo64: true,
                    });
                }
            }
        }

        // Also check for Whisky's bundled wine in ~/Library/Application Support/Whisky
        let whisky_app_support = home.join("Library/Application Support/Whisky");
        if whisky_app_support.exists() {
            let wine64 = whisky_app_support.join("Wine/bin/wine64");
            if wine64.exists() {
                let version = Self::get_version(&wine64);
                results.push(WineInfo {
                    backend: WineBackend::Whisky,
                    path: wine64,
                    version,
                    has_wo64: true,
                });
            }
        }

        results
    }

    fn detect_gptk(home: &std::path::Path) -> Vec<WineInfo> {
        let mut results = Vec::new();

        // Our bundled GPTK
        let gptk_wine = home.join("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64");
        if gptk_wine.exists() {
            let version = Self::get_version(&gptk_wine);
            results.push(WineInfo {
                backend: WineBackend::GPTK,
                path: gptk_wine,
                version,
                has_wo64: false,
            });
        }

        results
    }

    fn detect_wine_stable() -> Vec<WineInfo> {
        let mut results = Vec::new();

        // Wine 11+ only ships `wine` (not `wine64`), handles both architectures
        let base = PathBuf::from("/Applications/Wine Stable.app/Contents/Resources/wine/bin");
        let wine_bin = if base.join("wine64").exists() {
            base.join("wine64")
        } else if base.join("wine").exists() {
            base.join("wine")
        } else {
            PathBuf::new()
        };
        if wine_bin.exists() {
            let version = Self::get_version(&wine_bin);
            results.push(WineInfo {
                backend: WineBackend::WineStable,
                path: wine_bin,
                version,
                has_wo64: true,
            });
        }

        results
    }

    fn detect_homebrew() -> Vec<WineInfo> {
        let mut results = Vec::new();

        // Try wine64 first, then wine (Wine 11+ only ships wine)
        if let Ok(output) = Command::new("which").arg("wine64").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let path = PathBuf::from(&path_str);
                if path.exists() {
                    let version = Self::get_version(&path);
                    results.push(WineInfo {
                        backend: WineBackend::Unknown,
                        path,
                        version,
                        has_wo64: true,
                    });
                }
            }
        } else if let Ok(output) = Command::new("which").arg("wine").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let path = PathBuf::from(&path_str);
                if path.exists() && Self::get_version(&path).is_some() {
                    let version = Self::get_version(&path);
                    results.push(WineInfo {
                        backend: WineBackend::Unknown,
                        path,
                        version,
                        has_wo64: true,
                    });
                }
            }
        }

        results
    }

    fn detect_backend_for_path(path: &std::path::Path) -> WineBackend {
        let path_str = path.to_string_lossy().to_string();

        if path_str.contains("CrossOver") {
            WineBackend::CrossOver
        } else if path_str.contains("Whisky") || path_str.contains("whisky") {
            WineBackend::Whisky
        } else if path_str.contains("Moonshine") || path_str.contains("GPTK") || path_str.contains("game-porting-toolkit") {
            WineBackend::GPTK
        } else if path_str.contains("homebrew") || path_str.contains("/opt/homebrew") || path_str.contains("Wine Stable") {
            WineBackend::WineHQ
        } else {
            WineBackend::Unknown
        }
    }

    fn get_version(wine_bin: &std::path::Path) -> Option<String> {
        Command::new(wine_bin)
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            })
    }

    /// Check if this wine binary has the known Wine 11.0 msvcrt crash bug.
    /// Wine 11.0.x on macOS ARM64 crashes with 0xc0000417 (_invalid_parameter)
    /// in msvcrt.dll, preventing any wine command from running.
    /// This was fixed in Wine 11.1+.
    pub fn has_known_msvcrt_bug(&self) -> bool {
        if let Ok(version) = self.wine_version() {
            // Wine 11.0.x has the msvcrt crash on macOS ARM64
            // Match exactly "wine-11.0" or "wine-11.0." but NOT "wine-11.1", "wine-11.10", etc.
            if (version.starts_with("wine-11.0") && !version.starts_with("wine-11.0.0"))
                || version == "wine-11.0" {
                tracing::warn!(version = %version, "Detected Wine has known msvcrt bug (Wine 11.0)");
                return true;
            }
        }
        false
    }

    pub fn build_env(prefix: &Prefix, config: &BottleConfig, backend: &WineBackend) -> HashMap<String, String> {
        let mut env = HashMap::new();

        env.insert("WINEPREFIX".to_string(), prefix.path.to_string_lossy().to_string());

        // Set WINE to wine64 binary for winetricks and other tools
        if let Ok(runner) = Self::detect_for_config(config) {
            env.insert("WINE".to_string(), runner.wine_bin_path().to_string_lossy().to_string());
            // Add wine bin directory to PATH so wineserver and other tools are found
            if let Some(bin_dir) = runner.wine_bin_path().parent() {
                let bin_str = bin_dir.to_string_lossy().to_string();
                let existing_path = std::env::var("PATH").unwrap_or_default();
                if !existing_path.contains(&bin_str) {
                    env.insert("PATH".to_string(), format!("{}:{}", bin_str, existing_path));
                }
            }
        }

        // Use wow64 for WineHQ/WineStable (Wine 11+ has native WoW64)
        // Use win64 for GPTK (no 32-bit support)
        let arch = match backend {
            WineBackend::GPTK => "win64",
            _ => "wow64",
        };
        env.insert("WINEARCH".to_string(), arch.to_string());

        match config.sync_mode {
            SyncMode::ESync => {
                env.insert("WINEESYNC".to_string(), "1".to_string());
            }
            SyncMode::MSync => {
                env.insert("WINEMSYNC".to_string(), "1".to_string());
            }
            SyncMode::Default => {}
        }

        match config.graphics_backend {
            GraphicsBackend::D3DMetal => {
                env.insert("MTL_HUD_ENABLED".to_string(), "0".to_string());
            }
            GraphicsBackend::DXVK => {
                let mut overrides: Vec<String> = config
                    .dll_overrides
                    .iter()
                    .map(|(key, value)| format!("{}={}", key, value))
                    .collect();
                for dll in &["d3d11", "d3d10core", "dxgi"] {
                    if !overrides.iter().any(|o| o.starts_with(&format!("{}=", dll))) {
                        overrides.push(format!("{}=native", dll));
                    }
                }
                env.insert("DXVK_HUD".to_string(), if config.enable_dxvk_hud { "fps,frametimes,devinfo".to_string() } else { String::new() });
                env.insert("WINEDLLOVERRIDES".to_string(), overrides.join(";"));
            }
        }

        if config.enable_metal_fx && config.graphics_backend == GraphicsBackend::D3DMetal {
            env.insert("D3DM_ENABLE_METALFX".to_string(), "1".to_string());
        }

        if config.enable_dxvk_hud && config.graphics_backend == GraphicsBackend::DXVK {
            env.insert("DXVK_HUD".to_string(), "fps,frametimes,devinfo".to_string());
        }

        if !config.dll_overrides.is_empty() && config.graphics_backend != GraphicsBackend::DXVK {
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

        if !env.contains_key("PATH") {
            if let Ok(path) = std::env::var("PATH") {
                env.insert("PATH".to_string(), path);
            }
        }

        // --- HID Controller support (GameSir Nova, Xbox, PS5, DualSense, etc.) ---
        // SDL_JOYSTICK_MFI=0 : disables Apple MFI joystick stack so Wine can own the device
        // SDL_GAMECONTROLLERCONFIG : can be set per-game if needed
        // WINE_HIDE_CURSOR=1 : hides cursor while gamepad is active (optional)
        if config.enable_hid_controllers {
            env.insert("SDL_JOYSTICK_MFI".to_string(), "0".to_string());
            env.insert("SDL_GAMECONTROLLERCONFIG_FILE".to_string(), "".to_string());
            // Tell Wine to use SDL for controller input
            env.insert("WINE_SDL_JOY_ID_EVENT_DELAY".to_string(), "0".to_string());
            // Ensure XInput verbs work in winetricks
            env.insert("WINEDLLOVERRIDES".to_string(), {
                let existing = env.get("WINEDLLOVERRIDES").cloned().unwrap_or_default();
                if existing.is_empty() {
                    "xinput1_4=native,builtin;xinput9_1_0=native,builtin".to_string()
                } else if !existing.contains("xinput") {
                    format!("{};xinput1_4=native,builtin;xinput9_1_0=native,builtin", existing)
                } else {
                    existing
                }
            });
        }

        // --- Reduce Wine debug noise for better gaming performance ---
        if config.reduce_wine_debug {
            // Only suppress noisy channels; keep fixme:ntdll for crash debugging
            env.entry("WINEDEBUG".to_string())
                .or_insert_with(|| "-all,+err,+warn".to_string());
        }

        env
    }

    /// Run a Windows program through Wine and **wait** for it to exit.
    /// Use this for installers where you need to know when they finish.
    pub fn run_program(&self, prefix: &Prefix, program_path: &PathBuf) -> Result<Output> {
        let mut env = Self::build_env(prefix, &prefix.config, &self.backend);

        let home = crate::prefix::get_real_home();
        let homebrew_bin = home.join(".homebrew/bin").to_string_lossy().to_string();
        let existing_path = std::env::var("PATH").unwrap_or_default();
        env.insert("PATH".to_string(), format!("/opt/homebrew/bin:/usr/local/bin:{}:{}", homebrew_bin, existing_path));

        let unix_path = program_path.to_string_lossy().to_string();

        // Method 1: Try direct execution first
        let mut cmd = Command::new(&self.wine_bin);
        cmd.arg(&unix_path);
        for (key, value) in &env {
            cmd.env(key, value);
        }

        let output = cmd.output().map_err(|e| MoonshineError::Io(e))?;

        // If direct execution failed, try with start /unix
        if !output.status.success() {
            tracing::debug!("Direct execution failed, trying start /unix...");
            let mut cmd2 = Command::new(&self.wine_bin);
            cmd2.arg("start");
            cmd2.arg("/unix");
            cmd2.arg(&unix_path);
            for (key, value) in &env {
                cmd2.env(key, value);
            }
            return cmd2.output().map_err(|e| MoonshineError::Io(e));
        }

        Ok(output)
    }

    /// Launch a Windows program through Wine as a **detached background process**.
    /// Returns immediately — does NOT wait for the process to exit.
    /// Use this for Steam, games, and any long-running Windows apps.
    /// Returns the child process PID so the caller can track or kill it.
    pub fn launch_program(&self, prefix: &Prefix, program_path: &PathBuf) -> Result<u32> {
        let mut env = Self::build_env(prefix, &prefix.config, &self.backend);

        let home = crate::prefix::get_real_home();
        let homebrew_bin = home.join(".homebrew/bin").to_string_lossy().to_string();
        let existing_path = std::env::var("PATH").unwrap_or_default();
        env.insert("PATH".to_string(), format!("/opt/homebrew/bin:/usr/local/bin:{}:{}", homebrew_bin, existing_path));

        let unix_path = program_path.to_string_lossy().to_string();

        tracing::debug!(path = %unix_path, "Launching program (detached)");
        tracing::debug!(wine = %self.wine_bin.display(), "Using Wine binary");
        tracing::debug!(hid_controllers = prefix.config.enable_hid_controllers, "HID controller setting");

        let mut cmd = Command::new(&self.wine_bin);
        cmd.arg(&unix_path);
        for (key, value) in &env {
            cmd.env(key, value);
        }

        // spawn() returns immediately, child runs independently
        let child = cmd.spawn().map_err(|e| MoonshineError::Io(e))?;
        let pid = child.id();
        tracing::debug!(pid = pid, "Process launched");

        // Intentionally forget the child handle — process runs independently
        // macOS will clean up when the process exits
        std::mem::forget(child);

        Ok(pid)
    }

    pub fn init_prefix(&self, prefix: &Prefix) -> Result<Output> {
        let env = Self::build_env(prefix, &prefix.config, &self.backend);

        tracing::info!(prefix = %prefix.path.display(), "Running wineboot");
        tracing::debug!(wine = %self.wine_bin.display(), "Wine binary");
        tracing::debug!(backend = %self.backend, "Wine backend");

        // Pre-check: detect Wine 11.0 msvcrt bug before attempting wineboot
        if self.has_known_msvcrt_bug() {
            tracing::warn!("Wine 11.0 msvcrt bug detected, skipping direct wineboot...");
            // Fall through to GPTK fallback below
        } else {
            // First: wineboot (creates prefix structure)
            let output = Command::new(&self.wine_bin)
                .arg("wineboot")
                .envs(&env)
                .output()?;

            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::debug!(stdout = %stdout, stderr = %stderr, status = %output.status, "wineboot completed");

            // If wineboot failed, kill stale wineserver before fallback
            if !output.status.success() {
                tracing::debug!("wineboot failed, killing stale wineserver...");
                let wineserver = self.wine_bin.parent().unwrap_or(&self.wine_bin).join("wineserver");
                let _ = Command::new(&wineserver)
                    .arg("-k")
                    .arg("-w")
                    .envs(&env)
                    .output();
            }

            // If wineboot succeeded, run wineboot -u
            if output.status.success() {
                let output2 = Command::new(&self.wine_bin)
                    .arg("wineboot")
                    .arg("-u")
                    .envs(&env)
                    .output()?;

                let stderr2 = String::from_utf8_lossy(&output2.stderr);
                let stdout2 = String::from_utf8_lossy(&output2.stdout);
                tracing::debug!(stdout = %stdout2, stderr = %stderr2, status = %output2.status, "wineboot -u completed");

                // Populate syswow64 if backend lacks WoW64
                if !self.backend.has_wo64() {
                    Self::populate_syswow64(prefix);
                }

                return Ok(output);
            }

            // Wineboot failed — if this is WineHQ, try GPTK as fallback
            if self.backend == WineBackend::WineHQ {
                tracing::info!("WineHQ wineboot failed, falling back to GPTK for wineboot...");
            } else {
                // Non-WineHQ backend failed and no fallback — return the error
                return Ok(output);
            }
        }

        // GPTK fallback for wineboot (WineHQ 11.0 msvcrt bug or wineboot failure)
        if let Ok(gptk_runner) = Self::find_backend(WineBackend::GPTK) {
            let mut gptk_env = env.clone();
            gptk_env.insert("WINE".to_string(), gptk_runner.wine_bin_path().to_string_lossy().to_string());
            gptk_env.insert("WINEARCH".to_string(), "win64".to_string());
            let gptk_output = Command::new(gptk_runner.wine_bin_path())
                .arg("wineboot")
                .envs(&gptk_env)
                .output();
            if let Ok(gptk_result) = gptk_output {
                tracing::debug!(stderr = %String::from_utf8_lossy(&gptk_result.stderr), status = %gptk_result.status, "GPTK wineboot completed");

                if gptk_result.status.success() {
                    tracing::debug!("GPTK wineboot -u (Mono/Gecko install)...");
                    let gptk_update = Command::new(gptk_runner.wine_bin_path())
                        .arg("wineboot")
                        .arg("-u")
                        .envs(&gptk_env)
                        .output();
                    if let Ok(gptk_u) = gptk_update {
                        tracing::debug!(status = %gptk_u.status, "GPTK wineboot -u completed");
                    }

                    // Kill GPTK wineserver after initialization
                    tracing::debug!("Stopping GPTK wineserver...");
                    let _ = Command::new(gptk_runner.wine_bin_path().parent().unwrap_or(gptk_runner.wine_bin_path()).join("wineserver"))
                        .arg("-k")
                        .arg("-w")
                        .envs(&gptk_env)
                        .output();

                    // Populate syswow64 for GPTK (no real 32-bit support)
                    Self::populate_syswow64(prefix);

                    // Return a SUCCESS output since GPTK wineboot worked
                    // The prefix is now initialized and ready
                    return Ok(std::process::Output {
                        status: std::process::ExitStatus::from_raw(0),
                        stdout: Vec::new(),
                        stderr: b"[Moonshine] Prefix initialized via GPTK fallback. Note: 32-bit apps require WineHQ/CrossOver.".to_vec(),
                    });
                }
            }
            tracing::error!("GPTK wineboot also failed");
        } else {
            tracing::error!("GPTK not available as fallback");
        }

        // All attempts failed — return error
        Err(MoonshineError::WineProcessFailed(Some(53)))
    }

    /// Populate syswow64 with symlinks to system32 executables.
    /// Only needed for GPTK which doesn't include 32-bit Wine DLLs.
    fn populate_syswow64(prefix: &Prefix) {
        let syswow64 = prefix.path.join("drive_c/windows/syswow64");
        let system32 = prefix.path.join("drive_c/windows/system32");
        let windows_dir = prefix.path.join("drive_c/windows");

        tracing::debug!("Populating syswow64 with symlinks...");

        let critical_files = [
            "regedit.exe",
            "rundll32.exe",
            "rundll.exe",
            "cmd.exe",
            "msiexec.exe",
            "explorer.exe",
            "notepad.exe",
            "wineboot.exe",
            "winecfg.exe",
        ];

        for filename in &critical_files {
            let _source = if system32.join(filename).exists() {
                system32.join(filename)
            } else if windows_dir.join(filename).exists() {
                windows_dir.join(filename)
            } else {
                continue;
            };

            let target = syswow64.join(filename);
            if !target.exists() {
                let relative_source = PathBuf::from("..").join(
                    if system32.join(filename).exists() {
                        "system32"
                    } else {
                        ""
                    }
                ).join(filename);

                let _ = std::os::unix::fs::symlink(&relative_source, &target);
            }
        }

        let critical_dlls = [
            "kernel32.dll",
            "ntdll.dll",
            "user32.dll",
            "gdi32.dll",
            "advapi32.dll",
            "shell32.dll",
            "ole32.dll",
            "oleaut32.dll",
        ];

        for dll in &critical_dlls {
            let source = system32.join(dll);
            let target = syswow64.join(dll);
            if source.exists() && !target.exists() {
                let relative = PathBuf::from("../system32").join(dll);
                let _ = std::os::unix::fs::symlink(&relative, &target);
            }
        }

        tracing::debug!("syswow64 population complete");
    }

    pub fn set_windows_version(&self, prefix: &Prefix, version: &str) -> Result<Output> {
        let env = Self::build_env(prefix, &prefix.config, &self.backend);

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

    /// Check if WoW64 is properly set up in the prefix.
    pub fn check_wo64_status(prefix: &Prefix) -> Wo64Status {
        let syswow64 = prefix.path.join("drive_c/windows/syswow64");
        let system32 = prefix.path.join("drive_c/windows/system32");

        let syswow64_populated = if let Ok(entries) = std::fs::read_dir(&syswow64) {
            entries.count() > 0
        } else {
            false
        };

        let has_regedit32 = syswow64.join("regedit.exe").exists();
        let has_rundll32 = syswow64.join("rundll32.exe").exists();
        let has_regedit64 = system32.join("regedit.exe").exists() ||
                           prefix.path.join("drive_c/windows/regedit.exe").exists();

        if syswow64_populated && has_regedit32 && has_rundll32 {
            Wo64Status::Working
        } else if !syswow64.exists() {
            Wo64Status::NotCreated
        } else if !syswow64_populated {
            Wo64Status::Empty
        } else {
            Wo64Status::Partial {
                has_regedit: has_regedit32,
                has_rundll32,
                has_regedit64,
            }
        }
    }
}

#[derive(Debug)]
pub enum Wo64Status {
    Working,
    NotCreated,
    Empty,
    Partial {
        has_regedit: bool,
        has_rundll32: bool,
        has_regedit64: bool,
    },
}

impl std::fmt::Display for Wo64Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Working => write!(f, "WoW64 is working (syswow64 populated)"),
            Self::NotCreated => write!(f, "WoW64 not created (syswow64 directory missing)"),
            Self::Empty => write!(f, "WoW64 broken (syswow64 is empty - GPTK limitation)"),
            Self::Partial { has_regedit, has_rundll32, has_regedit64 } => {
                write!(f, "WoW64 partial: regedit32={}, rundll32={}, regedit64={}",
                       has_regedit, has_rundll32, has_regedit64)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wine_backend_has_wo64() {
        assert!(WineBackend::CrossOver.has_wo64());
        assert!(WineBackend::WineHQ.has_wo64());
        assert!(WineBackend::Whisky.has_wo64());
        assert!(!WineBackend::GPTK.has_wo64());
        assert!(WineBackend::WineStable.has_wo64());
        assert!(!WineBackend::Unknown.has_wo64());
    }

    #[test]
    fn test_wine_backend_display_name() {
        assert_eq!(WineBackend::CrossOver.display_name(), "CrossOver");
        assert_eq!(WineBackend::WineHQ.display_name(), "WineHQ (Homebrew)");
        assert_eq!(WineBackend::Whisky.display_name(), "Whisky");
        assert_eq!(WineBackend::GPTK.display_name(), "Game Porting Toolkit");
        assert_eq!(WineBackend::WineStable.display_name(), "Wine Stable");
        assert_eq!(WineBackend::Unknown.display_name(), "Custom");
    }

    #[test]
    fn test_wine_backend_display_trait() {
        assert_eq!(WineBackend::CrossOver.to_string(), "CrossOver");
        assert_eq!(WineBackend::WineHQ.to_string(), "WineHQ (Homebrew)");
    }

    #[test]
    fn test_wine_backend_ordering() {
        // CrossOver > WineHQ > Whisky > GPTK > WineStable > Unknown
        assert!(WineBackend::CrossOver < WineBackend::WineHQ);
        assert!(WineBackend::WineHQ < WineBackend::Whisky);
        assert!(WineBackend::Whisky < WineBackend::GPTK);
        assert!(WineBackend::GPTK < WineBackend::WineStable);
        assert!(WineBackend::WineStable < WineBackend::Unknown);
    }

    #[test]
    fn test_wine_runner_new() {
        let runner = WineRunner::new(PathBuf::from("/usr/bin/wine64"));
        assert_eq!(runner.wine_bin_path(), &PathBuf::from("/usr/bin/wine64"));
        assert_eq!(runner.backend(), &WineBackend::Unknown);
    }

    #[test]
    fn test_wine_runner_with_backend() {
        let runner = WineRunner::with_backend(PathBuf::from("/usr/bin/wine64"), WineBackend::GPTK);
        assert_eq!(runner.wine_bin_path(), &PathBuf::from("/usr/bin/wine64"));
        assert_eq!(runner.backend(), &WineBackend::GPTK);
    }

    #[test]
    fn test_wo64_status_display() {
        assert_eq!(Wo64Status::Working.to_string(), "WoW64 is working (syswow64 populated)");
        assert_eq!(Wo64Status::NotCreated.to_string(), "WoW64 not created (syswow64 directory missing)");
        assert_eq!(Wo64Status::Empty.to_string(), "WoW64 broken (syswow64 is empty - GPTK limitation)");
    }

    #[test]
    fn test_wo64_status_partial_display() {
        let status = Wo64Status::Partial {
            has_regedit: true,
            has_rundll32: false,
            has_regedit64: true,
        };
        assert_eq!(status.to_string(), "WoW64 partial: regedit32=true, rundll32=false, regedit64=true");
    }

    #[test]
    fn test_detect_backend_for_path() {
        assert_eq!(
            WineRunner::detect_backend_for_path(&PathBuf::from("/Applications/CrossOver.app/Contents/Frameworks/bin/wine64")),
            WineBackend::CrossOver
        );
        assert_eq!(
            WineRunner::detect_backend_for_path(&PathBuf::from("/Applications/Whisky.app/Contents/Resources/wine64")),
            WineBackend::Whisky
        );
        assert_eq!(
            WineRunner::detect_backend_for_path(&PathBuf::from("/Users/test/Library/Application Support/Moonshine/Libraries/Wine/bin/wine64")),
            WineBackend::GPTK
        );
        assert_eq!(
            WineRunner::detect_backend_for_path(&PathBuf::from("/opt/homebrew/bin/wine64")),
            WineBackend::WineHQ
        );
    }

    #[test]
    fn test_build_env_wineprefix() {
        let dir = std::env::temp_dir().join(format!("moonshine_env_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let prefix = Prefix::new("Test", &dir).unwrap();
        let config = prefix.config.clone();
        let env = WineRunner::build_env(&prefix, &config, &WineBackend::GPTK);

        assert!(env.contains_key("WINEPREFIX"));
        assert_eq!(env.get("WINEPREFIX").unwrap(), &prefix.path.to_string_lossy().to_string());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_build_env_esync() {
        let dir = std::env::temp_dir().join(format!("moonshine_env_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let mut prefix = Prefix::new("Test", &dir).unwrap();
        prefix.config.sync_mode = SyncMode::ESync;
        let config = prefix.config.clone();
        let env = WineRunner::build_env(&prefix, &config, &WineBackend::GPTK);

        assert!(env.contains_key("WINEESYNC"));
        assert_eq!(env.get("WINEESYNC").unwrap(), "1");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_build_env_msync() {
        let dir = std::env::temp_dir().join(format!("moonshine_env_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let mut prefix = Prefix::new("Test", &dir).unwrap();
        prefix.config.sync_mode = SyncMode::MSync;
        let config = prefix.config.clone();
        let env = WineRunner::build_env(&prefix, &config, &WineBackend::GPTK);

        assert!(env.contains_key("WINEMSYNC"));
        assert_eq!(env.get("WINEMSYNC").unwrap(), "1");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_build_env_gptk_arch() {
        let dir = std::env::temp_dir().join(format!("moonshine_env_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let prefix = Prefix::new("Test", &dir).unwrap();
        let config = prefix.config.clone();
        let env = WineRunner::build_env(&prefix, &config, &WineBackend::GPTK);

        assert_eq!(env.get("WINEARCH").unwrap(), "win64");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_build_env_non_gptk_arch() {
        let dir = std::env::temp_dir().join(format!("moonshine_env_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let prefix = Prefix::new("Test", &dir).unwrap();
        let config = prefix.config.clone();
        let env = WineRunner::build_env(&prefix, &config, &WineBackend::WineHQ);

        assert_eq!(env.get("WINEARCH").unwrap(), "wow64");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
