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
        eprintln!("[Moonshine] Wine dir: {}", wine_dir.display());

        let archives_dir = wine_dir.parent().unwrap().join("Archives");
        fs::create_dir_all(&archives_dir)?;

        let archive_name = url.split('/').last().unwrap_or("wine.tar.xz");
        let archive_path = archives_dir.join(archive_name);

        eprintln!("[Moonshine] Downloading Wine from {} to {}", url, archive_path.display());
        downloader::download_file(url, &archive_path)?;

        eprintln!("[Moonshine] Download complete, extracting...");
        if wine_dir.exists() {
            eprintln!("[Moonshine] Removing old wine dir");
            fs::remove_dir_all(&wine_dir)?;
        }

        let temp_dir = archives_dir.join("tmp_extract");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;

        eprintln!("[Moonshine] Extracting archive: {}", archive_name);
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

        eprintln!("[Moonshine] Extracted {} entries", entries.len());

        if entries.len() == 1 && entries[0].file_type().map_or(false, |t| t.is_dir()) {
            let inner = entries[0].path();
            let inner_name = inner.file_name().unwrap().to_string_lossy();
            eprintln!("[Moonshine] Single directory found: {}", inner_name);

            // Check if it's a .app bundle (e.g. "Game Porting Toolkit.app")
            if inner_name.ends_with(".app") {
                eprintln!("[Moonshine] Detected .app bundle, looking for wine inside...");
                // GPTK .app structure: Contents/Resources/wine/bin/wine64
                let app_wine = inner.join("Contents/Resources/wine/bin/wine64");
                if app_wine.exists() {
                    eprintln!("[Moonshine] Found wine at: {}", app_wine.display());
                    // Create wine dir and copy the wine subtree
                    fs::create_dir_all(&wine_dir)?;
                    let wine_src = inner.join("Contents/Resources/wine");
                    // Copy all contents from wine_src to wine_dir
                    copy_dir_all(&wine_src, &wine_dir)?;
                } else {
                    // Try flat structure inside .app
                    eprintln!("[Moonshine] No wine found in .app, trying flat copy");
                    fs::rename(&inner, &wine_dir)?;
                }
            } else {
                eprintln!("[Moonshine] Moving directory: {}", inner.display());
                fs::rename(&inner, &wine_dir)?;
            }
        } else {
            eprintln!("[Moonshine] Multiple entries, moving entire temp dir");
            fs::rename(&temp_dir, &wine_dir)?;
        }

        fs::remove_dir_all(&temp_dir).ok();
        fs::remove_file(&archive_path)?;

        // Remove macOS quarantine attribute so Gatekeeper doesn't block Wine binaries
        eprintln!("[Moonshine] Removing quarantine attributes...");
        let _ = Command::new("/usr/bin/xattr")
            .args(["-dr", "com.apple.quarantine"])
            .arg(&wine_dir)
            .output();

        let wine_bin = wine_dir.join("bin/wine64");
        eprintln!("[Moonshine] Checking wine binary: {}", wine_bin.display());
        if wine_bin.exists() {
            eprintln!("[Moonshine] Wine installed successfully!");
            Ok(wine_bin)
        } else {
            eprintln!("[Moonshine] Wine binary NOT found after extraction");
            Err(MoonshineError::WineNotFound(wine_bin))
        }
    }

    pub fn is_wine_installed() -> bool {
        Self::wine_binary_path().map_or(false, |p| p.exists())
    }

    pub fn wine_binary_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            MoonshineError::Config("Could not determine home directory".to_string())
        })?;
        Ok(home.join("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64"))
    }
}
