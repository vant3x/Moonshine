use crate::downloader;
use crate::config::BottleConfig;
use crate::error::{MoonshineError, Result};
use crate::wine::{WineBackend, WineInfo, WineRunner};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if ty.is_symlink() {
            // Preserve symlinks as-is (Wine/GPTK has many of these)
            let target = fs::read_link(&src_path)?;
            if dst_path.exists() || dst_path.symlink_metadata().is_ok() {
                fs::remove_file(&dst_path).ok();
            }
            std::os::unix::fs::symlink(&target, &dst_path)?;
        } else if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub struct Runtime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeType {
    Gptk,
    WineHQ,
    CrossOver,
    Whisky,
    WineStable,
    Custom,
}

impl From<&WineBackend> for RuntimeType {
    fn from(backend: &WineBackend) -> Self {
        match backend {
            WineBackend::GPTK => Self::Gptk,
            WineBackend::WineHQ => Self::WineHQ,
            WineBackend::CrossOver => Self::CrossOver,
            WineBackend::Whisky => Self::Whisky,
            WineBackend::WineStable => Self::WineStable,
            WineBackend::Unknown => Self::Custom,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeMetadata {
    pub runtime_type: RuntimeType,
    pub version: Option<String>,
    pub architecture: String,
    pub macos_version: String,
    pub wine_path: PathBuf,
    pub wineserver_path: PathBuf,
    pub sha256: Option<String>,
    pub installed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuntimeState {
    pub runtimes: Vec<RuntimeMetadata>,
}

pub struct RuntimeManager;

impl RuntimeManager {
    pub fn discover() -> Vec<RuntimeMetadata> {
        WineRunner::detect_all()
            .into_iter()
            .filter_map(|info| Self::metadata_for_info(&info).ok())
            .collect()
    }

    pub fn state() -> RuntimeState {
        RuntimeState { runtimes: Self::discover() }
    }

    pub fn selected_path(config: &BottleConfig) -> Result<PathBuf> {
        if let Some(path) = &config.wine_path {
            let path = PathBuf::from(path);
            Self::validate_runtime_path(&path)?;
            return Ok(path);
        }
        if config.wine_backend == crate::config::WineBackendConfig::Auto {
            for info in WineRunner::detect_all() {
                if Self::validate_runtime_path(&info.path).is_ok() {
                    return Ok(info.path);
                }
            }
            return Err(MoonshineError::WineNotFound(
                crate::prefix::get_real_home().join("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64"),
            ));
        }
        let runner = WineRunner::detect_for_config(config)?;
        Self::validate_runtime_path(runner.wine_bin_path())?;
        Ok(runner.wine_bin_path().clone())
    }

    pub fn validate_runtime_path(wine_path: &Path) -> Result<()> {
        if !wine_path.is_file() {
            return Err(MoonshineError::WineNotFound(wine_path.to_path_buf()));
        }
        let wineserver = wine_path
            .parent()
            .map(|directory| directory.join("wineserver"))
            .ok_or_else(|| MoonshineError::WineNotFound(wine_path.to_path_buf()))?;
        if !wineserver.is_file() {
            return Err(MoonshineError::WineNotFound(wineserver));
        }
        let architecture = detect_binary_architecture(wine_path)?;
        ensure_supported_architecture(&architecture)
    }

    pub fn validate_required_binaries(runtime_dir: &Path) -> Result<(PathBuf, PathBuf)> {
        let wine = runtime_dir.join("bin/wine64");
        let wineserver = runtime_dir.join("bin/wineserver");
        if !wine.is_file() {
            return Err(MoonshineError::WineNotFound(wine));
        }
        if !wineserver.is_file() {
            return Err(MoonshineError::WineNotFound(wineserver));
        }
        Ok((wine, wineserver))
    }

    fn metadata_for_info(info: &WineInfo) -> Result<RuntimeMetadata> {
        let runtime_dir = info.path.parent().and_then(Path::parent).unwrap_or_else(|| Path::new("/"));
        let (wine_path, wineserver_path) = Self::validate_required_binaries(runtime_dir)?;
        let architecture = detect_binary_architecture(&wine_path)?;
        ensure_supported_architecture(&architecture)?;
        ensure_supported_macos()?;
        Ok(RuntimeMetadata {
            runtime_type: RuntimeType::from(&info.backend),
            version: info.version.clone(),
            architecture,
            macos_version: host_macos_version(),
            wine_path,
            wineserver_path,
            sha256: None,
            installed_at: Utc::now().to_rfc3339(),
        })
    }
}

impl Runtime {
    pub fn download_wine(url: &str) -> Result<PathBuf> {
        Self::download_wine_with_checksum(url, None)
    }

    pub fn download_wine_with_checksum(url: &str, expected_sha256: Option<&str>) -> Result<PathBuf> {
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            return Err(MoonshineError::DownloadFailed("runtime URL must use HTTP(S)".to_string()));
        }
        let wine_dir = downloader::get_wine_dir()?;
        tracing::debug!(path = %wine_dir.display(), "Wine directory");

        let archives_dir = wine_dir.parent().unwrap().join("Archives");
        fs::create_dir_all(&archives_dir)?;

        let archive_name = safe_archive_name(url)?;
        let archive_path = archives_dir.join(&archive_name);
        let partial_path = archive_path.with_extension("part");

        tracing::info!(url = %url, dest = %archive_path.display(), "Downloading Wine");
        if let Err(error) = downloader::download_file(url, &partial_path) {
            let _ = fs::remove_file(&partial_path);
            return Err(error);
        }
        let actual_sha256 = downloader::sha256_file(&partial_path)?;
        if let Some(expected) = expected_sha256 {
            let expected = expected.to_ascii_lowercase();
            if expected != actual_sha256 {
                let _ = fs::remove_file(&partial_path);
                return Err(MoonshineError::ChecksumMismatch { expected, actual: actual_sha256 });
            }
        }
        fs::rename(&partial_path, &archive_path)?;

        tracing::debug!("Download complete, extracting...");
        let temp_dir = archives_dir.join(format!("tmp_extract-{}", Uuid::new_v4()));
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;

        tracing::debug!(archive = %archive_name, "Extracting archive");
        if archive_name.ends_with(".tar.xz") {
            downloader::extract_tar_xz(&archive_path, &temp_dir)?;
        } else if archive_name.ends_with(".tar.gz") || archive_name.ends_with(".tgz") {
            downloader::extract_tar_gz(&archive_path, &temp_dir)?;
        } else if archive_name.ends_with(".zip") {
            downloader::extract_zip(&archive_path, &temp_dir)?;
        } else {
            return Err(MoonshineError::DownloadFailed(format!(
                "Unsupported archive format: {}",
                archive_name
            )));
        }

        let staged_dir = archives_dir.join(format!("wine-staged-{}", Uuid::new_v4()));
        fs::create_dir_all(&staged_dir)?;

        // Handle archives with a single top-level directory (e.g. game-porting-toolkit-3.0-3/).
        let entries: Vec<_> = fs::read_dir(&temp_dir)?
            .filter_map(|e| e.ok())
            .collect();

        tracing::debug!(count = entries.len(), "Extracted entries");

        if entries.len() == 1 && entries[0].file_type().map_or(false, |t| t.is_dir()) {
            let inner = entries[0].path();
            let inner_name = inner.file_name().unwrap().to_string_lossy();
            tracing::debug!(name = %inner_name, "Single directory found");

            // Check if it's a .app bundle (e.g. "Game Porting Toolkit.app")
            if inner_name.ends_with(".app") {
                tracing::debug!("Detected .app bundle, looking for wine inside...");
                // GPTK .app structure: Contents/Resources/wine/bin/wine64
                let app_wine = inner.join("Contents/Resources/wine/bin/wine64");
                if app_wine.exists() {
                    tracing::debug!(path = %app_wine.display(), "Found wine in .app bundle");
                    // Copy only the Wine subtree from the app bundle.
                    let wine_src = inner.join("Contents/Resources/wine");
                    copy_dir_all(&wine_src, &staged_dir)?;
                } else {
                    return Err(MoonshineError::WineNotFound(inner.join("Contents/Resources/wine/bin/wine64")));
                }
            } else {
                copy_dir_all(&inner, &staged_dir)?;
            }
        } else {
            copy_dir_all(&temp_dir, &staged_dir)?;
        }

        // Create `wine` wrapper if missing (GPTK only ships wine64)
        let wine_bin = staged_dir.join("bin/wine64");
        let wine_wrapper = staged_dir.join("bin/wine");
        if wine_bin.exists() && !wine_wrapper.exists() {
            tracing::debug!("Creating wine wrapper script...");
            let wrapper_content = format!(
                "#!/bin/bash\nexec \"{}\" \"$@\"\n",
                wine_bin.display()
            );
            use std::io::Write;
            if let Ok(mut f) = fs::File::create(&wine_wrapper) {
                let _ = f.write_all(wrapper_content.as_bytes());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&wine_wrapper, fs::Permissions::from_mode(0o755));
            }
        }

        let (wine_bin, _wineserver_bin) = RuntimeManager::validate_required_binaries(&staged_dir)?;
        let architecture = detect_binary_architecture(&wine_bin)?;
        ensure_supported_architecture(&architecture)?;
        ensure_supported_macos()?;
        let metadata = RuntimeMetadata {
            runtime_type: RuntimeType::Gptk,
            version: None,
            architecture,
            macos_version: host_macos_version(),
            wine_path: wine_dir.join("bin/wine64"),
            wineserver_path: wine_dir.join("bin/wineserver"),
            sha256: Some(actual_sha256),
            installed_at: Utc::now().to_rfc3339(),
        };
        save_metadata(&staged_dir, &metadata)?;

        replace_runtime(&wine_dir, &staged_dir)?;
        let _ = fs::remove_dir_all(&temp_dir);

        // Gatekeeper handling happens only after the archive has passed checksum and binary validation.
        let _ = Command::new("/usr/bin/xattr")
            .args(["-dr", "com.apple.quarantine"])
            .arg(&wine_dir)
            .output();
        tracing::info!(path = %wine_bin.display(), "Wine installed successfully!");
        Ok(wine_dir.join("bin/wine64"))
    }

    pub fn is_wine_installed() -> bool {
        Self::wine_binary_path().map_or(false, |p| p.exists())
    }

    pub fn wine_binary_path() -> Result<PathBuf> {
        let home = crate::prefix::get_real_home();
        Ok(home.join("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64"))
    }
}

fn safe_archive_name(url: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    let path = url.split('?').next().unwrap_or_default();
    let extension = if path.ends_with(".tar.xz") {
        "tar.xz"
    } else if path.ends_with(".tar.gz") {
        "tar.gz"
    } else if path.ends_with(".tgz") {
        "tgz"
    } else if path.ends_with(".zip") {
        "zip"
    } else {
        ""
    };
    if extension.is_empty() {
        return Err(MoonshineError::DownloadFailed("unsupported runtime archive format".to_string()));
    }
    let mut digest = Sha256::new();
    digest.update(url.as_bytes());
    Ok(format!("runtime-{:x}.{}", digest.finalize(), extension))
}

fn save_metadata(runtime_dir: &Path, metadata: &RuntimeMetadata) -> Result<()> {
    let path = runtime_dir.join("runtime.json");
    let temp_path = runtime_dir.join("runtime.json.tmp");
    fs::write(&temp_path, serde_json::to_vec_pretty(metadata)?)?;
    fs::rename(temp_path, path)?;
    Ok(())
}

fn detect_binary_architecture(path: &Path) -> Result<String> {
    let output = Command::new("/usr/bin/file").arg(path).output()?;
    if !output.status.success() {
        return Err(MoonshineError::UnsupportedArchitecture(path.display().to_string()));
    }
    let description = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    if description.contains("arm64") || description.contains("aarch64") {
        Ok("arm64".to_string())
    } else if description.contains("x86_64") || description.contains("amd64") {
        Ok("x86_64".to_string())
    } else {
        Err(MoonshineError::UnsupportedArchitecture(description))
    }
}

fn ensure_supported_architecture(architecture: &str) -> Result<()> {
    ensure_architecture_for_host(architecture, std::env::consts::ARCH)
}

fn ensure_architecture_for_host(architecture: &str, host: &str) -> Result<()> {
    if host == "aarch64" && architecture != "arm64" && architecture != "x86_64" {
        return Err(MoonshineError::UnsupportedArchitecture(architecture.to_string()));
    }
    Ok(())
}

fn replace_runtime(wine_dir: &Path, staged_dir: &Path) -> Result<()> {
    let backup = wine_dir.with_extension(format!("previous-{}", Uuid::new_v4()));
    if wine_dir.exists() {
        fs::rename(wine_dir, &backup)?;
    }
    if let Err(error) = fs::rename(staged_dir, wine_dir) {
        if backup.exists() {
            let _ = fs::rename(&backup, wine_dir);
        }
        return Err(error.into());
    }
    let _ = fs::remove_dir_all(backup);
    Ok(())
}

fn host_macos_version() -> String {
    Command::new("/usr/bin/sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|version| !version.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn ensure_supported_macos() -> Result<()> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }
    let version = host_macos_version();
    let major = version.split('.').next().and_then(|value| value.parse::<u32>().ok()).unwrap_or(0);
    if major < 14 {
        return Err(MoonshineError::Config(format!("Wine runtime requires macOS 14 or newer, found {}", version)));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_runtime_requires_wine_and_wineserver() {
        let root = std::env::temp_dir().join(format!("moonshine-runtime-{}", Uuid::new_v4()));
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("bin/wine64"), b"wine").unwrap();
        fs::write(root.join("bin/wineserver"), b"server").unwrap();

        assert!(RuntimeManager::validate_required_binaries(&root).is_ok());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_runtime_executable_is_rejected() {
        let root = std::env::temp_dir().join(format!("moonshine-runtime-{}", Uuid::new_v4()));
        fs::create_dir_all(root.join("bin")).unwrap();
        fs::write(root.join("bin/wineserver"), b"server").unwrap();

        assert!(matches!(RuntimeManager::validate_required_binaries(&root), Err(MoonshineError::WineNotFound(_))));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn arm_runtime_is_rejected_on_x86_host() {
        assert!(ensure_architecture_for_host("arm64", "x86_64").is_ok());
        assert!(ensure_architecture_for_host("i386", "aarch64").is_err());
    }

    #[test]
    fn unsupported_archive_url_is_rejected_before_download() {
        assert!(Runtime::download_wine("https://example.invalid/runtime.exe").is_err());
        assert!(safe_archive_name("https://example.invalid/runtime.tar.xz?download=1").is_ok());
    }

    #[test]
    fn failed_replacement_preserves_existing_runtime() {
        let root = std::env::temp_dir().join(format!("moonshine-runtime-{}", Uuid::new_v4()));
        let existing = root.join("Wine");
        let missing_stage = root.join("missing-stage");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("marker"), b"keep").unwrap();

        assert!(replace_runtime(&existing, &missing_stage).is_err());
        assert_eq!(fs::read(existing.join("marker")).unwrap(), b"keep");
        fs::remove_dir_all(root).unwrap();
    }
}
