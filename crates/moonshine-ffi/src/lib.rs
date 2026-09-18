use moonshine_core::{GraphicsBackend, Prefix, SyncMode, WindowsVersion, WineBackendConfig};
use std::sync::atomic::{AtomicBool, Ordering};

static LAST_INSTALL_FAILED: AtomicBool = AtomicBool::new(false);

/// Initialize tracing once. Safe to call multiple times.
/// Sends Rust logs to stderr so Swift can capture them during downloads.
pub fn init_logging() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .with_target(false)
            .with_ansi(false)
            .init();
    });
}


#[swift_bridge::bridge]
mod ffi {
    enum SwiftWindowsVersion {
        Win10,
        Win11,
    }

    enum SwiftGraphicsBackend {
        D3DMetal,
        DXVK,
    }

    enum SwiftSyncMode {
        Default,
        ESync,
        MSync,
    }

    enum SwiftWineBackend {
        Auto,
        WineHQ,
        GPTK,
        CrossOver,
        Custom,
    }

    extern "Rust" {
        type RustPrefix;
        #[swift_bridge(init)]
        fn new_prefix(name: &str) -> Option<RustPrefix>;
        fn get_id(&self) -> String;
        fn get_name(&self) -> String;
        fn get_path(&self) -> String;
        fn get_windows_version(&self) -> SwiftWindowsVersion;
        fn set_windows_version(&mut self, version: SwiftWindowsVersion);
        fn get_graphics_backend(&self) -> SwiftGraphicsBackend;
        fn set_graphics_backend(&mut self, backend: SwiftGraphicsBackend);
        fn get_sync_mode(&self) -> SwiftSyncMode;
        fn set_sync_mode(&mut self, mode: SwiftSyncMode);
        fn get_metal_fx(&self) -> bool;
        fn set_metal_fx(&mut self, enabled: bool);
        fn get_dxvk_hud(&self) -> bool;
        fn set_dxvk_hud(&mut self, enabled: bool);
        fn get_wine_path(&self) -> Option<String>;
        fn set_wine_path(&mut self, path: &str);
        fn get_wine_backend(&self) -> SwiftWineBackend;
        fn set_wine_backend(&mut self, backend: SwiftWineBackend);
        fn get_hid_controllers(&self) -> bool;
        fn set_hid_controllers(&mut self, enabled: bool);
        fn get_reduce_wine_debug(&self) -> bool;
        fn set_reduce_wine_debug(&mut self, enabled: bool);
        fn save(&self) -> bool;
        fn delete_prefix(&self) -> bool;
        fn reinit_prefix(&self) -> String;
        fn list_executables(&self) -> Vec<String>;
        fn run_program(&self, program_path: &str) -> bool;
        fn launch_program(&self, program_path: &str) -> u32;
        fn init_prefix(&self) -> bool;
        fn install_steam(&self) -> String;
        fn launch_steam(&self) -> u32;
        fn run_winetricks(&self, verb: &str) -> String;
        fn run_winetricks_preset(&self, preset: &str) -> String;
        fn find_steam_exe(&self) -> Option<String>;
    }

    extern "Rust" {
        type WineBackendInfo;
        fn get_backend_name(&self) -> String;
        fn get_wine_path(&self) -> String;
        fn get_version(&self) -> Option<String>;
        fn has_wo64_support(&self) -> bool;
    }

    extern "Rust" {
        fn init_logging();
        fn detect_wine() -> Option<String>;
        fn wine_version() -> Option<String>;
        fn get_base_dir() -> String;
        fn list_all_prefixes() -> Vec<RustPrefix>;
        fn download_file(url: &str, dest: &str) -> bool;
        fn get_wine_dir() -> String;
        fn get_gptk_dir() -> String;
        fn install_wine(url: &str) -> bool;
        fn is_wine_installed() -> bool;
        fn last_install_error() -> bool;
        fn get_available_verbs() -> Vec<String>;
        fn detect_all_wine_backends() -> Vec<WineBackendInfo>;
        fn get_best_wine_backend() -> Option<WineBackendInfo>;
        fn has_wine_msvcrt_bug() -> bool;
        fn kill_process(pid: u32) -> bool;
    }
}

pub struct RustPrefix {
    inner: Prefix,
}

impl RustPrefix {
    pub fn new_prefix(name: &str) -> Option<Self> {
        let base_dir = moonshine_core::get_base_dir().ok()?;
        let prefix = Prefix::new(name, &base_dir).ok()?;
        Some(Self { inner: prefix })
    }

    pub fn get_id(&self) -> String {
        self.inner.id.clone()
    }

    pub fn get_name(&self) -> String {
        self.inner.name.clone()
    }

    pub fn get_path(&self) -> String {
        self.inner.path.to_string_lossy().to_string()
    }

    pub fn get_windows_version(&self) -> ffi::SwiftWindowsVersion {
        match self.inner.config.windows_version {
            WindowsVersion::Win10 => ffi::SwiftWindowsVersion::Win10,
            WindowsVersion::Win11 => ffi::SwiftWindowsVersion::Win11,
        }
    }

    pub fn set_windows_version(&mut self, version: ffi::SwiftWindowsVersion) {
        self.inner.config.windows_version = match version {
            ffi::SwiftWindowsVersion::Win10 => WindowsVersion::Win10,
            ffi::SwiftWindowsVersion::Win11 => WindowsVersion::Win11,
        };
    }

    pub fn get_graphics_backend(&self) -> ffi::SwiftGraphicsBackend {
        match self.inner.config.graphics_backend {
            GraphicsBackend::D3DMetal => ffi::SwiftGraphicsBackend::D3DMetal,
            GraphicsBackend::DXVK => ffi::SwiftGraphicsBackend::DXVK,
        }
    }

    pub fn set_graphics_backend(&mut self, backend: ffi::SwiftGraphicsBackend) {
        self.inner.config.graphics_backend = match backend {
            ffi::SwiftGraphicsBackend::D3DMetal => GraphicsBackend::D3DMetal,
            ffi::SwiftGraphicsBackend::DXVK => GraphicsBackend::DXVK,
        };
    }

    pub fn get_sync_mode(&self) -> ffi::SwiftSyncMode {
        match self.inner.config.sync_mode {
            SyncMode::Default => ffi::SwiftSyncMode::Default,
            SyncMode::ESync => ffi::SwiftSyncMode::ESync,
            SyncMode::MSync => ffi::SwiftSyncMode::MSync,
        }
    }

    pub fn set_sync_mode(&mut self, mode: ffi::SwiftSyncMode) {
        self.inner.config.sync_mode = match mode {
            ffi::SwiftSyncMode::Default => SyncMode::Default,
            ffi::SwiftSyncMode::ESync => SyncMode::ESync,
            ffi::SwiftSyncMode::MSync => SyncMode::MSync,
        };
    }

    pub fn get_metal_fx(&self) -> bool {
        self.inner.config.enable_metal_fx
    }

    pub fn set_metal_fx(&mut self, enabled: bool) {
        self.inner.config.enable_metal_fx = enabled;
    }

    pub fn get_dxvk_hud(&self) -> bool {
        self.inner.config.enable_dxvk_hud
    }

    pub fn set_dxvk_hud(&mut self, enabled: bool) {
        self.inner.config.enable_dxvk_hud = enabled;
    }

    pub fn get_wine_path(&self) -> Option<String> {
        self.inner.config.wine_path.clone()
    }

    pub fn set_wine_path(&mut self, path: &str) {
        self.inner.config.wine_path = if path.is_empty() {
            None
        } else {
            Some(path.to_string())
        };
    }

    pub fn get_wine_backend(&self) -> ffi::SwiftWineBackend {
        match self.inner.config.wine_backend {
            WineBackendConfig::Auto => ffi::SwiftWineBackend::Auto,
            WineBackendConfig::WineHQ => ffi::SwiftWineBackend::WineHQ,
            WineBackendConfig::GPTK => ffi::SwiftWineBackend::GPTK,
            WineBackendConfig::CrossOver => ffi::SwiftWineBackend::CrossOver,
            WineBackendConfig::Custom => ffi::SwiftWineBackend::Custom,
        }
    }

    pub fn set_wine_backend(&mut self, backend: ffi::SwiftWineBackend) {
        self.inner.config.wine_backend = match backend {
            ffi::SwiftWineBackend::Auto => WineBackendConfig::Auto,
            ffi::SwiftWineBackend::WineHQ => WineBackendConfig::WineHQ,
            ffi::SwiftWineBackend::GPTK => WineBackendConfig::GPTK,
            ffi::SwiftWineBackend::CrossOver => WineBackendConfig::CrossOver,
            ffi::SwiftWineBackend::Custom => WineBackendConfig::Custom,
        };
    }

    pub fn get_hid_controllers(&self) -> bool {
        self.inner.config.enable_hid_controllers
    }

    pub fn set_hid_controllers(&mut self, enabled: bool) {
        self.inner.config.enable_hid_controllers = enabled;
    }

    pub fn get_reduce_wine_debug(&self) -> bool {
        self.inner.config.reduce_wine_debug
    }

    pub fn set_reduce_wine_debug(&mut self, enabled: bool) {
        self.inner.config.reduce_wine_debug = enabled;
    }

    pub fn save(&self) -> bool {
        self.inner.save_config().is_ok()
    }

    pub fn delete_prefix(&self) -> bool {
        self.inner.delete().is_ok()
    }

    pub fn reinit_prefix(&self) -> String {
        let prefix_path = self.inner.path.clone();
        let prefix_name = self.inner.name.clone();
        let config = self.inner.config.clone();

        tracing::info!(prefix = %prefix_name, path = %prefix_path.display(), "Reinitializing prefix");

        if prefix_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(&prefix_path) {
                let msg = format!("Failed to delete prefix: {}", e);
                tracing::error!(error = %e, "Failed to delete prefix");
                return msg;
            }
        }

        if let Err(e) = std::fs::create_dir_all(&prefix_path) {
            let msg = format!("Failed to create prefix dir: {}", e);
            tracing::error!(error = %e, "Failed to create prefix dir");
            return msg;
        }
        let _ = std::fs::create_dir_all(prefix_path.join("drive_c"));
        let _ = std::fs::create_dir_all(prefix_path.join("drive_c/Program Files"));
        let _ = std::fs::create_dir_all(prefix_path.join("drive_c/users"));

        if let Err(e) = config.save(&prefix_path) {
            let msg = format!("Failed to save config: {}", e);
            tracing::error!(error = %e, "Failed to save config");
            return msg;
        }

        match moonshine_core::WineRunner::detect_for_config(&config) {
            Ok(runner) => {
                let new_prefix = moonshine_core::Prefix {
                    id: self.inner.id.clone(),
                    name: prefix_name.clone(),
                    path: prefix_path,
                    config,
                };
                match runner.init_prefix(&new_prefix) {
                    Ok(output) => {
                        let status = output.status;
                        if status.success() {
                            format!("Prefix '{}' reinitialized successfully", prefix_name)
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            format!("wineboot completed with errors: {}", stderr.trim())
                        }
                    }
                    Err(e) => format!("wineboot failed: {}", e),
                }
            }
            Err(e) => format!("Wine not found: {}", e),
        }
    }

    pub fn list_executables(&self) -> Vec<String> {
        self.inner
            .list_executables()
            .unwrap_or_default()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }

    pub fn run_program(&self, program_path: &str) -> bool {
        if let Ok(runner) = moonshine_core::WineRunner::detect_for_config(&self.inner.config) {
            let path = std::path::PathBuf::from(program_path);
            runner.run_program(&self.inner, &path).is_ok()
        } else {
            false
        }
    }

    /// Launch a program as a detached background process. Returns PID (0 = failed).
    pub fn launch_program(&self, program_path: &str) -> u32 {
        if let Ok(runner) = moonshine_core::WineRunner::detect_for_config(&self.inner.config) {
            let path = std::path::PathBuf::from(program_path);
            runner.launch_program(&self.inner, &path).unwrap_or(0)
        } else {
            tracing::error!("Wine not found for launch_program");
            0
        }
    }

    pub fn init_prefix(&self) -> bool {
        match moonshine_core::WineRunner::detect_for_config(&self.inner.config) {
            Ok(runner) => {
                tracing::info!(prefix = %self.inner.name, "Running wineboot");
                match runner.init_prefix(&self.inner) {
                    Ok(output) => {
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            tracing::error!(stderr = %stderr, "wineboot stderr");
                        }
                        tracing::debug!(status = %output.status, "wineboot completed");
                        true
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "wineboot failed");
                        false
                    }
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Wine not found, cannot init prefix");
                false
            }
        }
    }

    pub fn install_steam(&self) -> String {
        match moonshine_core::installer::install_steam(&self.inner) {
            Ok(path) => {
                tracing::info!(path = %path, "Steam installed");
                path
            }
            Err(e) => {
                let msg = format!("Steam install failed: {}", e);
                tracing::error!(error = %e, "Steam install failed");
                msg
            }
        }
    }

    pub fn run_winetricks(&self, verb: &str) -> String {
        match moonshine_core::installer::run_winetricks(&self.inner, verb) {
            Ok(output) => output,
            Err(e) => {
                let msg = format!("winetricks {} failed: {}", verb, e);
                tracing::error!(verb = %verb, error = %e, "winetricks failed");
                msg
            }
        }
    }

    pub fn run_winetricks_preset(&self, preset: &str) -> String {
        match moonshine_core::installer::run_winetricks_preset(&self.inner, preset) {
            Ok(output) => output,
            Err(e) => {
                let msg = format!("preset {} failed: {}", preset, e);
                tracing::error!(preset = %preset, error = %e, "winetricks preset failed");
                msg
            }
        }
    }

    pub fn find_steam_exe(&self) -> Option<String> {
        self.inner.find_steam_exe()
            .map(|p| p.to_string_lossy().to_string())
    }

    /// Launch Steam as a detached background process. Returns PID (0 = failed).
    pub fn launch_steam(&self) -> u32 {
        match moonshine_core::installer::launch_steam_detached(&self.inner) {
            Ok(pid) => {
                tracing::info!(pid = pid, "Steam launched");
                pid
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to launch Steam");
                0
            }
        }
    }
}

impl RustPrefix {
    pub fn list_all_prefixes() -> Vec<RustPrefix> {
        let base_dir = moonshine_core::get_base_dir().unwrap_or_default();
        Prefix::list_all(&base_dir)
            .unwrap_or_default()
            .into_iter()
            .map(|p| RustPrefix { inner: p })
            .collect()
    }
}

pub struct WineBackendInfo {
    inner: moonshine_core::WineInfo,
}

impl WineBackendInfo {
    pub fn get_backend_name(&self) -> String {
        self.inner.backend.display_name().to_string()
    }

    pub fn get_wine_path(&self) -> String {
        self.inner.path.to_string_lossy().to_string()
    }

    pub fn get_version(&self) -> Option<String> {
        self.inner.version.clone()
    }

    pub fn has_wo64_support(&self) -> bool {
        self.inner.has_wo64
    }
}

pub fn list_all_prefixes() -> Vec<RustPrefix> {
    RustPrefix::list_all_prefixes()
}

pub fn detect_wine() -> Option<String> {
    moonshine_core::WineRunner::detect()
        .ok()
        .map(|r| r.wine_bin_path().to_string_lossy().to_string())
}

pub fn wine_version() -> Option<String> {
    moonshine_core::WineRunner::detect()
        .ok()
        .and_then(|r| r.wine_version().ok())
}

pub fn get_base_dir() -> String {
    moonshine_core::get_base_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn download_file(url: &str, dest: &str) -> bool {
    let dest_path = std::path::PathBuf::from(dest);
    moonshine_core::downloader::download_file(url, &dest_path).is_ok()
}

pub fn get_wine_dir() -> String {
    moonshine_core::downloader::get_wine_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn get_gptk_dir() -> String {
    moonshine_core::downloader::get_gptk_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn install_wine(url: &str) -> bool {
    match moonshine_core::runtime::Runtime::download_wine(url) {
        Ok(path) => {
            tracing::info!(path = %path.display(), "Wine installed successfully");
            LAST_INSTALL_FAILED.store(false, Ordering::Relaxed);
            true
        }
        Err(e) => {
            tracing::error!(error = %e, "install_wine FAILED");
            LAST_INSTALL_FAILED.store(true, Ordering::Relaxed);
            false
        }
    }
}

pub fn last_install_error() -> bool {
    LAST_INSTALL_FAILED.load(Ordering::Relaxed)
}

pub fn is_wine_installed() -> bool {
    moonshine_core::runtime::Runtime::is_wine_installed()
}

pub fn get_available_verbs() -> Vec<String> {
    moonshine_core::installer::list_available_verbs()
        .into_iter()
        .map(|(verb, desc)| format!("{}: {}", verb, desc))
        .collect()
}

pub fn detect_all_wine_backends() -> Vec<WineBackendInfo> {
    moonshine_core::WineRunner::detect_all()
        .into_iter()
        .map(|info| WineBackendInfo { inner: info })
        .collect()
}

pub fn get_best_wine_backend() -> Option<WineBackendInfo> {
    moonshine_core::WineRunner::detect_best()
        .ok()
        .map(|info| WineBackendInfo { inner: info })
}

pub fn has_wine_msvcrt_bug() -> bool {
    match moonshine_core::WineRunner::detect() {
        Ok(runner) => runner.has_known_msvcrt_bug(),
        Err(_) => false,
    }
}

/// Kill a process by PID. Use to stop Steam or a game launched via launch_program/launch_steam.
/// Returns true if the signal was sent successfully.
pub fn kill_process(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    // Send SIGTERM first (graceful shutdown)
    let result = unsafe { libc::kill(pid as libc::pid_t, libc::SIGTERM) };
    if result == 0 {
        tracing::debug!(pid = pid, "Sent SIGTERM");
        true
    } else {
        tracing::error!(pid = pid, error = %std::io::Error::last_os_error(), "kill failed");
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kill_process_zero_pid() {
        assert!(!kill_process(0));
    }

    #[test]
    fn test_get_base_dir() {
        let base_dir = get_base_dir();
        assert!(!base_dir.is_empty());
    }

    #[test]
    fn test_get_wine_dir() {
        let wine_dir = get_wine_dir();
        assert!(!wine_dir.is_empty());
    }

    #[test]
    fn test_get_gptk_dir() {
        let gptk_dir = get_gptk_dir();
        assert!(!gptk_dir.is_empty());
    }

    #[test]
    fn test_get_available_verbs() {
        let verbs = get_available_verbs();
        assert!(!verbs.is_empty());
        assert!(verbs.iter().any(|v| v.contains("steam")));
        assert!(verbs.iter().any(|v| v.contains("vcrun2019")));
    }

    #[test]
    fn test_rust_prefix_new_prefix_returns_option() {
        let prefix = RustPrefix::new_prefix("TestFFI");
        // May be Some or None depending on filesystem permissions
        // The important thing is it doesn't panic
        if let Some(p) = prefix {
            assert_eq!(p.get_name(), "TestFFI");
            assert!(!p.get_id().is_empty());
            let _ = p.delete_prefix();
        }
    }

    #[test]
    fn test_wine_backend_info() {
        let info = WineBackendInfo {
            inner: moonshine_core::WineInfo {
                backend: moonshine_core::WineBackend::GPTK,
                path: std::path::PathBuf::from("/test/wine64"),
                version: Some("wine-9.0".to_string()),
                has_wo64: false,
            },
        };

        assert_eq!(info.get_backend_name(), "Game Porting Toolkit");
        assert_eq!(info.get_wine_path(), "/test/wine64");
        assert_eq!(info.get_version(), Some("wine-9.0".to_string()));
        assert!(!info.has_wo64_support());
    }

    #[test]
    fn test_list_all_prefixes_returns_vec() {
        let prefixes = list_all_prefixes();
        assert!(prefixes.is_empty() || !prefixes.is_empty()); // Just verify it doesn't panic
    }

    #[test]
    fn test_last_install_error_default() {
        // Reset the atomic
        LAST_INSTALL_FAILED.store(false, Ordering::Relaxed);
        assert!(!last_install_error());
    }
}
