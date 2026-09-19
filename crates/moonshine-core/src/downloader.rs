use crate::error::{MoonshineError, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

pub fn download_file(url: &str, dest: &PathBuf) -> Result<PathBuf> {
    tracing::info!(url = %url, dest = %dest.display(), "Downloading file");

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
        tracing::error!(stderr = %stderr, stdout = %stdout, "curl failed");
        return Err(MoonshineError::DownloadFailed(format!(
            "curl failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        )));
    }

    let file_size = fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    tracing::debug!(bytes = file_size, dest = %dest.display(), "Download complete");
    Ok(dest.clone())
}

pub fn extract_tar_gz(archive: &PathBuf, dest: &PathBuf) -> Result<()> {
    let file = fs::File::open(archive)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    unpack_tar_safely(&mut archive, dest)?;
    Ok(())
}

pub fn extract_tar_xz(archive: &PathBuf, dest: &PathBuf) -> Result<()> {
    let file = fs::File::open(archive)?;
    let decoder = xz2::read::XzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    unpack_tar_safely(&mut archive, dest)?;
    Ok(())
}

fn safe_archive_path(path: &Path) -> Result<()> {
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        })
    {
        return Err(MoonshineError::UnsafeArchiveEntry(path.display().to_string()));
    }
    Ok(())
}

fn unpack_tar_safely<R: std::io::Read>(archive: &mut tar::Archive<R>, dest: &Path) -> Result<()> {
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        safe_archive_path(&path)?;
        if let Some(link) = entry.link_name()? {
            safe_archive_path(&link)?;
        }
        entry.unpack_in(dest)?;
    }
    Ok(())
}

pub fn extract_zip(archive: &PathBuf, dest: &PathBuf) -> Result<()> {
    let file = fs::File::open(archive)?;
    let mut zip = zip::ZipArchive::new(file)
        .map_err(|e| MoonshineError::DownloadFailed(e.to_string()))?;
    for index in 0..zip.len() {
        let mut entry = zip
            .by_index(index)
            .map_err(|e| MoonshineError::DownloadFailed(e.to_string()))?;
        let path = entry
            .enclosed_name()
            .ok_or_else(|| MoonshineError::UnsafeArchiveEntry(entry.name().to_string()))?
            .to_path_buf();
        safe_archive_path(&path)?;
        let output_path = dest.join(path);
        if entry.is_dir() {
            fs::create_dir_all(output_path)?;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = fs::File::create(output_path)?;
        std::io::copy(&mut entry, &mut output)?;
    }
    Ok(())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn verify_sha256(path: &Path, expected: &str) -> Result<()> {
    let actual = sha256_file(path)?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(MoonshineError::ChecksumMismatch {
            expected: expected.to_ascii_lowercase(),
            actual,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_verification_rejects_invalid_checksum() {
        let path = std::env::temp_dir().join(format!("moonshine-checksum-{}", uuid::Uuid::new_v4()));
        fs::write(&path, b"runtime").unwrap();

        assert!(matches!(verify_sha256(&path, "00"), Err(MoonshineError::ChecksumMismatch { .. })));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn archive_path_traversal_is_rejected() {
        assert!(safe_archive_path(Path::new("../outside")).is_err());
        assert!(safe_archive_path(Path::new("bin/wine64")).is_ok());
    }
}

pub fn get_libraries_dir() -> Result<PathBuf> {
    let home = crate::prefix::get_real_home();
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
