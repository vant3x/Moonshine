use crate::downloader;
use crate::error::{MoonshineError, Result};
use std::fs;
use std::path::PathBuf;

pub struct Runtime;

impl Runtime {
    pub fn download_wine(url: &str) -> Result<PathBuf> {
        let wine_dir = downloader::get_wine_dir()?;

        let archives_dir = wine_dir.parent().unwrap().join("Archives");
        fs::create_dir_all(&archives_dir)?;

        let archive_name = url.split('/').last().unwrap_or("wine.tar.xz");
        let archive_path = archives_dir.join(archive_name);

        println!("Downloading Wine from {}", url);
        downloader::download_file(url, &archive_path)?;

        if wine_dir.exists() {
            fs::remove_dir_all(&wine_dir)?;
        }

        let temp_dir = archives_dir.join("tmp_extract");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir)?;
        }
        fs::create_dir_all(&temp_dir)?;

        println!("Extracting...");
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

        if entries.len() == 1 && entries[0].file_type().map_or(false, |t| t.is_dir()) {
            let inner = entries[0].path();
            fs::rename(&inner, &wine_dir)?;
        } else {
            fs::rename(&temp_dir, &wine_dir)?;
        }

        fs::remove_dir_all(&temp_dir).ok();
        fs::remove_file(&archive_path)?;

        let wine_bin = wine_dir.join("bin/wine64");
        if wine_bin.exists() {
            Ok(wine_bin)
        } else {
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
