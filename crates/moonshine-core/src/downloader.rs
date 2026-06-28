use crate::error::{MoonshineError, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn download_file(url: &str, dest: &PathBuf) -> Result<PathBuf> {
    eprintln!("[Moonshine] Downloading: {} -> {}", url, dest.display());

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    // Use system curl — handles TLS, redirects (302), and certificates correctly
    let output = Command::new("/usr/bin/curl")
        .arg("-L")               // follow redirects
        .arg("-f")               // fail on HTTP errors
        .arg("--connect-timeout")
        .arg("30")
        .arg("--max-time")
        .arg("0")                // no overall timeout for large files
        .arg("--retry")
        .arg("3")
        .arg("--retry-delay")
        .arg("2")
        .arg("-o")
        .arg(dest)
        .arg(url)
        .output()
        .map_err(|e| MoonshineError::DownloadFailed(format!("Failed to run curl: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        eprintln!("[Moonshine] curl stderr: {}", stderr);
        eprintln!("[Moonshine] curl stdout: {}", stdout);
        return Err(MoonshineError::DownloadFailed(format!(
            "curl failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    let file_size = fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    eprintln!("[Moonshine] Downloaded {} bytes to {}", file_size, dest.display());
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
