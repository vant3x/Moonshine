use crate::downloader;
use crate::error::{MoonshineError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

impl Runtime {
    pub fn download_wine(url: &str) -> Result<PathBuf> {
        let wine_dir = downloader::get_wine_dir()?;
        tracing::debug!(path = %wine_dir.display(), "Wine directory");

        let archives_dir = wine_dir.parent().unwrap().join("Archives");
        fs::create_dir_all(&archives_dir)?;

        let archive_name = url.split('/').last().unwrap_or("wine.tar.xz");
        let archive_path = archives_dir.join(archive_name);

        tracing::info!(url = %url, dest = %archive_path.display(), "Downloading Wine");
        downloader::download_file(url, &archive_path)?;

        tracing::debug!("Download complete, extracting...");
        if wine_dir.exists() {
            tracing::debug!("Removing old wine dir");
            fs::remove_dir_all(&wine_dir)?;
        }

        let temp_dir = archives_dir.join("tmp_extract");
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

        // Handle archives with a single top-level directory (e.g. game-porting-toolkit-3.0-3/)
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
                    // Create wine dir and copy the wine subtree
                    fs::create_dir_all(&wine_dir)?;
                    let wine_src = inner.join("Contents/Resources/wine");
                    // Copy all contents from wine_src to wine_dir
                    copy_dir_all(&wine_src, &wine_dir)?;
                } else {
                    // Try flat structure inside .app
                    tracing::debug!("No wine found in .app, trying flat copy");
                    fs::rename(&inner, &wine_dir)?;
                }
            } else {
                tracing::debug!(path = %inner.display(), "Moving directory");
                fs::rename(&inner, &wine_dir)?;
            }
        } else {
            tracing::debug!("Multiple entries, moving entire temp dir");
            fs::rename(&temp_dir, &wine_dir)?;
        }

        fs::remove_dir_all(&temp_dir).ok();
        fs::remove_file(&archive_path)?;

        // Remove macOS quarantine attribute so Gatekeeper doesn't block Wine binaries
        tracing::debug!("Removing quarantine attributes...");
        let _ = Command::new("/usr/bin/xattr")
            .args(["-dr", "com.apple.quarantine"])
            .arg(&wine_dir)
            .output();

        // Create `wine` wrapper if missing (GPTK only ships wine64)
        let wine_bin = wine_dir.join("bin/wine64");
        let wine_wrapper = wine_dir.join("bin/wine");
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

        // Also check wineserver
        let wineserver_bin = wine_dir.join("bin/wineserver");
        if wineserver_bin.exists() {
            tracing::debug!(path = %wineserver_bin.display(), "wineserver found");
        }

        tracing::debug!(path = %wine_bin.display(), "Checking wine binary");
        if wine_bin.exists() {
            tracing::info!("Wine installed successfully!");
            Ok(wine_bin)
        } else {
            tracing::error!("Wine binary NOT found after extraction");
            Err(MoonshineError::WineNotFound(wine_bin))
        }
    }

    pub fn is_wine_installed() -> bool {
        Self::wine_binary_path().map_or(false, |p| p.exists())
    }

    pub fn wine_binary_path() -> Result<PathBuf> {
        let home = crate::prefix::get_real_home();
        Ok(home.join("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64"))
    }
}
