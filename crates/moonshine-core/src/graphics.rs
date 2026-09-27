use crate::config::{BottleConfig, GraphicsBackend};
use crate::downloader;
use crate::error::{MoonshineError, Result};
use crate::wine::WineBackend;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphicsDiagnostics {
    pub backend: GraphicsBackend,
    pub usable: bool,
    pub required_files: Vec<String>,
    pub environment: Vec<(String, String)>,
    pub message: String,
}

pub fn diagnose(config: &BottleConfig, wine_backend: &WineBackend, wine_path: &Path) -> GraphicsDiagnostics {
    match config.graphics_backend {
        GraphicsBackend::D3DMetal => {
            let usable = *wine_backend == WineBackend::GPTK && wine_path.is_file();
            GraphicsDiagnostics {
                backend: GraphicsBackend::D3DMetal,
                usable,
                required_files: vec![wine_path.display().to_string()],
                environment: vec![("MTL_HUD_ENABLED".to_string(), "0".to_string())],
                message: if usable {
                    "D3DMetal is provided by the selected GPTK runtime.".to_string()
                } else {
                    "D3DMetal requires a valid GPTK runtime; this backend does not bundle D3DMetal.".to_string()
                },
            }
        }
        GraphicsBackend::DXVK => {
            let directory = downloader::get_libraries_dir()
                .map(|path| path.join("dxvk"))
                .unwrap_or_else(|_| std::path::PathBuf::from("Libraries/dxvk"));
            diagnose_dxvk_directory(&directory)
        }
    }
}

pub fn diagnose_dxvk_directory(directory: &Path) -> GraphicsDiagnostics {
    let required = [directory.join("d3d11.dll"), directory.join("dxgi.dll")];
    let usable = required.iter().all(|path| path.is_file());
    GraphicsDiagnostics {
        backend: GraphicsBackend::DXVK,
        usable,
        required_files: required.iter().map(|path| path.display().to_string()).collect(),
        environment: vec![
            ("WINEDLLOVERRIDES".to_string(), "d3d11=native;d3d10core=native;dxgi=native".to_string()),
            ("DXVK_HUD".to_string(), "disabled unless enabled per prefix".to_string()),
        ],
        message: if usable {
            "DXVK DLLs are present and can be selected.".to_string()
        } else {
            "DXVK is unavailable: install the required DLL payloads; Moonshine does not redistribute them.".to_string()
        },
    }
}

pub fn validate(config: &BottleConfig, wine_backend: &WineBackend, wine_path: &Path) -> Result<GraphicsDiagnostics> {
    let diagnostics = diagnose(config, wine_backend, wine_path);
    if diagnostics.usable {
        Ok(diagnostics)
    } else {
        Err(MoonshineError::GraphicsBackendUnavailable(diagnostics.message.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn dxvk_requires_both_translation_dlls() {
        let directory = std::env::temp_dir().join(format!("moonshine-dxvk-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&directory).unwrap();
        assert!(!diagnose_dxvk_directory(&directory).usable);
        fs::write(directory.join("d3d11.dll"), b"dll").unwrap();
        fs::write(directory.join("dxgi.dll"), b"dll").unwrap();
        assert!(diagnose_dxvk_directory(&directory).usable);
        fs::remove_dir_all(directory).unwrap();
    }
}