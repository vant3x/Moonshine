use crate::error::{MoonshineError, Result};
use std::fs;
use std::path::PathBuf;

pub fn download_file(url: &str, dest: &PathBuf) -> Result<PathBuf> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Moonshine/0.1")
        .build()
        .map_err(|e| MoonshineError::DownloadFailed(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| MoonshineError::DownloadFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(MoonshineError::DownloadFailed(format!(
            "HTTP {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .map_err(|e| MoonshineError::DownloadFailed(e.to_string()))?;

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(dest, &bytes)?;
    Ok(dest.clone())
}

pub fn extract_tar_gz(archive: &PathBuf, dest: &PathBuf) -> Result<()> {
    let file = fs::File::open(archive)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

pub fn extract_tar_xz(archive: &PathBuf, dest: &PathBuf) -> Result<()> {
    let output = std::process::Command::new("tar")
        .arg("-xJf")
        .arg(archive)
        .arg("-C")
        .arg(dest)
        .output()
        .map_err(|e| MoonshineError::DownloadFailed(format!("tar failed: {}", e)))?;

    if !output.status.success() {
        return Err(MoonshineError::DownloadFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    Ok(())
}

pub fn extract_zip(archive: &PathBuf, dest: &PathBuf) -> Result<()> {
    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| MoonshineError::DownloadFailed(e.to_string()))?;
    zip.extract(dest)
        .map_err(|e| MoonshineError::DownloadFailed(e.to_string()))?;
    Ok(())
}

pub fn get_libraries_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        MoonshineError::Config("Could not determine home directory".to_string())
    })?;
    let dir = home.join("Library/Application Support/Moonshine/Libraries");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn get_wine_dir() -> Result<PathBuf> {
    Ok(get_libraries_dir()?.join("Wine"))
}

pub fn get_gptk_dir() -> Result<PathBuf> {
    Ok(get_libraries_dir()?.join("GPTK"))
}
