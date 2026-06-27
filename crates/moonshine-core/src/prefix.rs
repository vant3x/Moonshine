use crate::config::BottleConfig;
use crate::error::{Result, MoonshineError};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Prefix {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub config: BottleConfig,
}

impl Prefix {
    pub fn new(name: &str, base_dir: &PathBuf) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let prefix_dir = base_dir.join(&id);

        if prefix_dir.exists() {
            return Err(MoonshineError::PrefixAlreadyExists(name.to_string()));
        }

        fs::create_dir_all(&prefix_dir)?;
        fs::create_dir_all(prefix_dir.join("drive_c"))?;
        fs::create_dir_all(prefix_dir.join("drive_c/Program Files"))?;
        fs::create_dir_all(prefix_dir.join("drive_c/users"))?;

        let mut config = BottleConfig::default();
        config.name = name.to_string();
        config.save(&prefix_dir)?;

        Ok(Self {
            id,
            name: name.to_string(),
            path: prefix_dir,
            config,
        })
    }

    pub fn load(id: &str, base_dir: &PathBuf) -> Result<Self> {
        let prefix_dir = base_dir.join(id);

        if !prefix_dir.exists() {
            return Err(MoonshineError::PrefixNotFound(id.to_string()));
        }

        let config = BottleConfig::load(&prefix_dir)?;

        Ok(Self {
            id: id.to_string(),
            name: config.name.clone(),
            path: prefix_dir,
            config,
        })
    }

    pub fn list_all(base_dir: &PathBuf) -> Result<Vec<Self>> {
        let mut prefixes = Vec::new();

        if !base_dir.exists() {
            return Ok(prefixes);
        }

        for entry in fs::read_dir(base_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                if let Ok(prefix) = Self::load(&dir_name, base_dir) {
                    prefixes.push(prefix);
                }
            }
        }

        prefixes.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(prefixes)
    }

    pub fn delete(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_dir_all(&self.path)?;
        }
        Ok(())
    }

    pub fn update_config(&mut self, config: BottleConfig) -> Result<()> {
        self.config = config;
        self.config.save(&self.path)?;
        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        self.config.save(&self.path)
    }

    pub fn drive_c(&self) -> PathBuf {
        self.path.join("drive_c")
    }

    pub fn programs_dir(&self) -> PathBuf {
        self.drive_c().join("Program Files")
    }

    pub fn list_executables(&self) -> Result<Vec<PathBuf>> {
        let mut exes = Vec::new();
        let programs_dir = self.programs_dir();

        if !programs_dir.exists() {
            return Ok(exes);
        }

        for entry in walkdir::WalkDir::new(&programs_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();
            if path.extension().map_or(false, |ext| ext == "exe") {
                exes.push(path);
            }
        }

        Ok(exes)
    }
}

pub fn get_base_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        MoonshineError::Config("Could not determine home directory".to_string())
    })?;
    let base = home.join("Library/Application Support/Moonshine/Prefixes");
    fs::create_dir_all(&base)?;
    Ok(base)
}
