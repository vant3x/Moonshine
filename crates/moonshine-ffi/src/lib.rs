use moonshine_core::{GraphicsBackend, Prefix, SyncMode, WindowsVersion};

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

    extern "Rust" {
        type RustPrefix;
        #[swift_bridge(init)]
        fn new_prefix(name: &str) -> RustPrefix;
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
        fn save(&self) -> bool;
        fn delete_prefix(&self) -> bool;
        fn list_executables(&self) -> Vec<String>;
    }

    extern "Rust" {
        fn detect_wine() -> Option<String>;
        fn wine_version() -> Option<String>;
        fn get_base_dir() -> String;
        fn list_all_prefixes() -> Vec<RustPrefix>;
        fn download_file(url: &str, dest: &str) -> bool;
        fn get_wine_dir() -> String;
        fn get_gptk_dir() -> String;
        fn install_wine(url: &str) -> bool;
        fn is_wine_installed() -> bool;
    }
}

pub struct RustPrefix {
    inner: Prefix,
}

impl RustPrefix {
    pub fn new_prefix(name: &str) -> Self {
        let base_dir = moonshine_core::get_base_dir().expect("Could not get base dir");
        let prefix = Prefix::new(name, &base_dir).expect("Could not create prefix");
        Self { inner: prefix }
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

    pub fn save(&self) -> bool {
        self.inner.save_config().is_ok()
    }

    pub fn delete_prefix(&self) -> bool {
        self.inner.delete().is_ok()
    }

    pub fn list_executables(&self) -> Vec<String> {
        self.inner
            .list_executables()
            .unwrap_or_default()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect()
    }
}

impl RustPrefix {
    pub fn list_all_prefixes() -> Vec<RustPrefix> {
        let base_dir = moonshine_core::get_base_dir().expect("Could not get base dir");
        Prefix::list_all(&base_dir)
            .unwrap_or_default()
            .into_iter()
            .map(|p| RustPrefix { inner: p })
            .collect()
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
    moonshine_core::runtime::Runtime::download_wine(url).is_ok()
}

pub fn is_wine_installed() -> bool {
    moonshine_core::runtime::Runtime::is_wine_installed()
}
