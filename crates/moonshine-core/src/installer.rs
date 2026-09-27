use crate::downloader;
use crate::error::{MoonshineError, Result};
use crate::game::GameProfile;
use crate::prefix::Prefix;
use crate::wine::WineRunner;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const STEAM_SETUP_URL: &str = "https://cdn.akamai.steamstatic.com/client/installer/SteamSetup.exe";

const WINETRICKS_URL: &str =
    "https://raw.githubusercontent.com/Winetricks/winetricks/master/src/winetricks";

pub fn get_installers_dir() -> Result<PathBuf> {
    let home = crate::prefix::get_real_home();
    let dir = home.join("Library/Application Support/Moonshine/Installers");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn download_steam_setup() -> Result<PathBuf> {
    let dir = get_installers_dir()?;
    let dest = dir.join("SteamSetup.exe");
    if dest.exists() {
        tracing::debug!("SteamSetup.exe already cached");
        return Ok(dest);
    }
    tracing::info!("Downloading SteamSetup.exe...");
    downloader::download_file(STEAM_SETUP_URL, &dest)?;
    Ok(dest)
}

pub fn download_winetricks() -> Result<PathBuf> {
    let dir = get_installers_dir()?;
    let dest = dir.join("winetricks");
    if dest.exists() {
        // Check if it's still reasonably fresh (< 7 days)
        if let Ok(meta) = fs::metadata(&dest) {
            if let Ok(modified) = meta.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() < 7 * 86400 {
                        return Ok(dest);
                    }
                }
            }
        }
    }
    tracing::info!("Downloading winetricks...");
    downloader::download_file(WINETRICKS_URL, &dest)?;
    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&dest, fs::Permissions::from_mode(0o755));
    }
    Ok(dest)
}

pub fn install_steam(prefix: &Prefix) -> Result<String> {
    let setup_exe = download_steam_setup()?;
    install_steam_with_setup(prefix, &setup_exe)
}

pub fn install_steam_from_path(prefix: &Prefix, setup_exe: &std::path::Path) -> Result<String> {
    if !setup_exe.is_file()
        || setup_exe.extension().map_or(true, |extension| !extension.eq_ignore_ascii_case("exe"))
    {
        return Err(MoonshineError::InvalidPath(setup_exe.to_path_buf()));
    }
    install_steam_with_setup(prefix, setup_exe)
}

fn install_steam_with_setup(prefix: &Prefix, setup_exe: &std::path::Path) -> Result<String> {
    let runner = WineRunner::detect_for_config(&prefix.config)?;

    // SteamSetup.exe is a 32-bit application — requires WoW64 support.
    // If current backend doesn't have WoW64, try to find one that does.
    let effective_runner = if !runner.backend().has_wo64() {
        tracing::info!(
            backend = %runner.backend(),
            "Current backend doesn't support 32-bit. Looking for WoW64 backend..."
        );
        match WineRunner::find_backend_with_wo64() {
            Ok(wo64_runner) => {
                tracing::info!(
                    backend = %wo64_runner.backend(),
                    "Auto-fallback: using backend for Steam installation"
                );
                wo64_runner
            }
            Err(_) => {
                return Err(MoonshineError::Config(
                    "SteamSetup.exe is a 32-bit application and requires WoW64 support.\n\n\
                     Your current Wine backend (GPTK) doesn't support 32-bit apps.\n\n\
                     Install one of these:\n\
                     • WineHQ:   brew install --cask wine-stable\n\
                     • CrossOver: https://www.codeweavers.com\n\n\
                     Then restart Moonshine.".to_string()
                ));
            }
        }
    } else {
        runner
    };

    // Check for Wine 11.0 msvcrt bug — all wine commands crash
    if effective_runner.has_known_msvcrt_bug() {
        return Err(MoonshineError::Config(
            "Wine 11.0 has a critical bug (msvcrt.dll crash) on macOS.\n\n\
             All wine commands fail with: Unhandled exception 0xc0000417\n\n\
             This prevents Steam from installing.\n\n\
             Fix — install Wine 11.10 (has the bug fix):\n\
             brew install --cask wine@devel\n\n\
             Or install CrossOver (best compatibility):\n\
             https://www.codeweavers.com".to_string()
        ));
    }

    if !effective_runner.wine_bin_path().is_file() {
        return Err(MoonshineError::WineNotFound(effective_runner.wine_bin_path().clone()));
    }

    tracing::info!(prefix = %prefix.name, "Running SteamSetup.exe");
    tracing::debug!(wine = %effective_runner.wine_bin_path().display(), "Wine binary");
    tracing::debug!(backend = %effective_runner.backend(), "Backend");
    tracing::debug!(prefix = %prefix.path.display(), "Prefix path");

    // Copy to drive_c to avoid path issues with GPTK wine
    let drive_c = prefix.drive_c();
    let dest_exe = drive_c.join("Temp").join("MoonshineSteamSetup.exe");
    fs::create_dir_all(dest_exe.parent().unwrap())?;
    fs::copy(&setup_exe, &dest_exe)?;

    let mut env = WineRunner::build_env(prefix, &prefix.config, &effective_runner.backend());
    // Steam may use a WoW64 fallback different from the prefix's configured runner.
    env.insert(
        "WINE".to_string(),
        effective_runner.wine_bin_path().to_string_lossy().to_string(),
    );

    // Ensure cabextract/Homebrew tools are in PATH for winetricks deps
    let home = crate::prefix::get_real_home();
    let homebrew_bin = home.join(".homebrew/bin").to_string_lossy().to_string();
    let existing_path = std::env::var("PATH").unwrap_or_default();
    env.insert("PATH".to_string(), format!("/opt/homebrew/bin:/usr/local/bin:{}:{}", homebrew_bin, existing_path));

    // Add wine bin directory to PATH so wineserver is found
    if let Some(bin_dir) = effective_runner.wine_bin_path().parent() {
        let bin_str = bin_dir.to_string_lossy().to_string();
        if !env.get("PATH").map_or(false, |p| p.contains(&bin_str)) {
            let current = env.get("PATH").cloned().unwrap_or_default();
            env.insert("PATH".to_string(), format!("{}:{}", bin_str, current));
        }
    }

    // Try running with wine64 directly first (bypasses start.exe issues)
    let unix_path = dest_exe.to_string_lossy().to_string();
    tracing::debug!("Attempting to run SteamSetup.exe (silent) directly with wine64...");

    // Method 1: Direct wine64 execution with /S flag for silent install
    // /S = silent install, /D = install directory (must be last arg)
    let steam_install_dir = "C:\\Program Files (x86)\\Steam";
    let output = std::process::Command::new(effective_runner.wine_bin_path())
        .arg(&unix_path)
        .arg("/S")
        .arg("/D")
        .arg(steam_install_dir)
        .envs(&env)
        .output()
        .map_err(|e| MoonshineError::Io(e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    write_steam_log(prefix, &format!("direct installer\nstdout:\n{}\nstderr:\n{}\n", stdout, stderr))?;
    tracing::debug!(stdout = %stdout, stderr = %stderr, "SteamSetup direct execution");

    // If direct execution failed, try with start /unix
    if !output.status.success() {
        tracing::debug!("Direct execution failed, trying with start /unix (silent)...");
        let output2 = std::process::Command::new(effective_runner.wine_bin_path())
            .arg("start")
            .arg("/unix")
            .arg(&unix_path)
            .arg("/S")
            .arg("/D")
            .arg(steam_install_dir)
            .envs(&env)
            .output()
            .map_err(|e| MoonshineError::Io(e))?;

        let stdout2 = String::from_utf8_lossy(&output2.stdout).to_string();
        let stderr2 = String::from_utf8_lossy(&output2.stderr).to_string();
        write_steam_log(prefix, &format!("fallback installer\nstdout:\n{}\nstderr:\n{}\n", stdout2, stderr2))?;
        tracing::debug!(stdout = %stdout2, stderr = %stderr2, "SteamSetup start /unix execution");

        // If both methods failed, report the error
        if !output2.status.success() {
            let _ = fs::remove_file(&dest_exe);

            // Check if it's a 32-bit exe issue
            if stderr.contains("failed to start") || stderr.contains("failed to open") ||
               stderr2.contains("failed to start") || stderr2.contains("failed to open") {
                return Err(MoonshineError::Config(format!(
                    "SteamSetup.exe is a 32-bit application. WoW64 support is incomplete.\n\n\
                     Current backend: {}\n\n\
                     Try one of these:\n\
                     • Install WineHQ:  brew install --cask wine-stable\n\
                     • Install CrossOver: https://www.codeweavers.com\n\
                     • Or install Steam manually via the .exe after configuring Wine.",
                    effective_runner.backend()
                )));
            }

            return Err(MoonshineError::WineProcessFailed(output2.status.code()));
        }
    }

    let _ = fs::remove_file(&dest_exe);

    // Kill wineserver after Steam installation to avoid stale processes
    tracing::debug!("Stopping wineserver after Steam install...");
    let wineserver = effective_runner.wine_bin_path().parent()
        .unwrap_or(effective_runner.wine_bin_path())
        .join("wineserver");
    let _ = Command::new(&wineserver)
        .arg("-k")
        .arg("-w")
        .envs(&env)
        .output();

    // Steam installs to drive_c/Program Files (x86)/Steam/ by default
    let steam_exe = prefix.drive_c().join("Program Files (x86)/Steam/steam.exe");
    if steam_exe.exists() {
        tracing::info!(path = %steam_exe.display(), "Steam installed successfully");
        Ok(steam_exe.to_string_lossy().to_string())
    } else {
        let alt = prefix.drive_c().join("Program Files/Steam/steam.exe");
        if alt.exists() {
            tracing::info!(path = %alt.display(), "Steam installed at alternative location");
            Ok(alt.to_string_lossy().to_string())
        } else {
            tracing::warn!("Steam installer ran but steam.exe not found");
            Err(MoonshineError::WineProcessFailed(Some(0)))
        }
    }
}

/// Launch Steam as a detached background process. Returns PID immediately.
/// Handles XInput/controller env vars automatically via build_env.
pub fn launch_steam_detached(prefix: &Prefix) -> Result<u32> {
    let runner = WineRunner::detect_for_config(&prefix.config)?;

    let steam_exe = prefix.find_steam_exe()
        .ok_or_else(|| MoonshineError::Config(
            "Steam not found in this prefix. Install it first.".to_string()
        ))?;

    tracing::info!(path = %steam_exe.display(), "Launching Steam (detached)");
    tracing::debug!(hid_controllers = prefix.config.enable_hid_controllers, "Controller support");

    let mut steam_prefix = prefix.clone();
    steam_prefix.config.wine_path = Some(runner.wine_bin_path().to_string_lossy().to_string());
    runner.launch_system_program_with_args(
        &steam_prefix,
        &steam_exe,
        &["-cef-disable-gpu", "-no-cef-sandbox"],
    )
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct InstalledSteamGame {
    pub app_id: String,
    pub name: String,
    pub install_dir: PathBuf,
    pub executable_candidates: Vec<PathBuf>,
    pub profiles: Vec<GameProfile>,
}

/// Discover games from local Steam manifests only. This does not contact Steam,
/// authenticate, download content, or choose an executable silently.
pub fn discover_steam_games(prefix: &Prefix) -> Result<Vec<InstalledSteamGame>> {
    let steam_exe = prefix
        .find_steam_exe()
        .ok_or_else(|| MoonshineError::Config("Steam is not installed in this prefix.".to_string()))?;
    let steam_root = steam_exe
        .parent()
        .ok_or_else(|| MoonshineError::InvalidPath(steam_exe.clone()))?;
    let mut libraries = vec![steam_root.to_path_buf()];
    let library_file = steam_root.join("steamapps/libraryfolders.vdf");
    if library_file.is_file() {
        let content = fs::read_to_string(&library_file)?;
        for line in content.lines() {
            if let Some(path) = quoted_value(line, "path") {
                let path = PathBuf::from(path.replace("\\\\", "\\"));
                if path.is_dir() && !libraries.contains(&path) {
                    libraries.push(path);
                }
            }
        }
    }

    let mut games = Vec::new();
    for library in libraries {
        let steamapps = library.join("steamapps");
        if !steamapps.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&steamapps)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.starts_with("appmanifest_") || !file_name.ends_with(".acf") {
                continue;
            }
            let content = fs::read_to_string(entry.path())?;
            let app_id = file_name
                .trim_start_matches("appmanifest_")
                .trim_end_matches(".acf")
                .to_string();
            let name = quoted_value(&content, "name").unwrap_or_else(|| format!("Steam App {}", app_id));
            let install_dir_name = match quoted_value(&content, "installdir") {
                Some(value) => value,
                None => continue,
            };
            let install_dir = steamapps.join("common").join(install_dir_name);
            if !install_dir.is_dir() {
                continue;
            }
            let executable_candidates = find_game_executables(&install_dir)?;
            let profiles = executable_candidates
                .iter()
                .filter_map(|path| GameProfile::new(&prefix.id, path).ok())
                .collect();
            games.push(InstalledSteamGame {
                app_id,
                name,
                executable_candidates,
                profiles,
                install_dir,
            });
        }
    }
    games.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(games)
}

fn write_steam_log(prefix: &Prefix, content: &str) -> Result<()> {
    let logs = prefix.path.join("logs");
    fs::create_dir_all(&logs)?;
    fs::write(logs.join("steam-install.log"), content)?;
    Ok(())
}

fn quoted_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let fields: Vec<&str> = line.split('"').collect();
        if fields.len() >= 4 && fields[1].trim() == key {
            return Some(fields[3].replace("\\\\", "\\"));
        }
    }
    None
}

fn find_game_executables(root: &std::path::Path) -> Result<Vec<PathBuf>> {
    let mut executables = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(|entry| entry.ok())
    {
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .map_or(false, |extension| extension.eq_ignore_ascii_case("exe"))
        {
            executables.push(entry.path().to_path_buf());
        }
    }
    executables.sort();
    Ok(executables)
}

pub fn check_winetricks_deps() -> Result<()> {
    // Check multiple possible locations for cabextract
    let home = crate::prefix::get_real_home();
    let candidates = [
        "/opt/homebrew/bin/cabextract".to_string(),
        "/usr/local/bin/cabextract".to_string(),
        home.join(".homebrew/bin/cabextract").to_string_lossy().to_string(),
    ];

    for candidate in &candidates {
        if std::path::Path::new(candidate).exists() {
            return Ok(());
        }
    }

    // Fallback to `which`
    let output = std::process::Command::new("which")
        .arg("cabextract")
        .output()
        .map_err(|e| MoonshineError::Io(e))?;
    if output.status.success() {
        return Ok(());
    }

    Err(MoonshineError::Config(
        "cabextract not found. Install with: brew install cabextract".to_string(),
    ))
}

pub fn run_winetricks(prefix: &Prefix, verb: &str) -> Result<String> {
    check_winetricks_deps()?;
    let runner = WineRunner::detect_for_config(&prefix.config)?;

    // Check for Wine 11.0 msvcrt bug — all wine commands crash
    if runner.has_known_msvcrt_bug() {
        return Err(MoonshineError::Config(
            "Wine 11.0 has a critical bug (msvcrt.dll crash) on macOS.\n\n\
             All wine commands fail with: Unhandled exception 0xc0000417\n\n\
             This prevents winetricks from running.\n\n\
             Fix — install Wine 11.10 (has the bug fix):\n\
             brew install --cask wine@devel\n\n\
             Or install CrossOver:\n\
             https://www.codeweavers.com".to_string()
        ));
    }

    let winetricks = download_winetricks()?;

    tracing::info!(verb = %verb, prefix = %prefix.name, "Running winetricks");
    tracing::debug!(backend = %runner.backend(), "Wine backend");
    tracing::debug!(has_wo64 = runner.backend().has_wo64(), "WoW64 support");

    // Create a patched winetricks wrapper that fixes the syswow64 regedit path issue
    let patched_winetricks = create_patched_winetricks(&winetricks, prefix)?;

    let mut env = WineRunner::build_env(prefix, &prefix.config, &runner.backend());

    // Ensure cabextract and other tools are in PATH
    let home = crate::prefix::get_real_home();
    let homebrew_bin = home.join(".homebrew/bin").to_string_lossy().to_string();
    let existing_path = std::env::var("PATH").unwrap_or_default();
    env.insert("PATH".to_string(), format!("/opt/homebrew/bin:/usr/local/bin:{}:{}", homebrew_bin, existing_path));

    // Add wine bin directory to PATH so wineserver and other wine tools are found
    if let Some(bin_dir) = runner.wine_bin_path().parent() {
        let bin_str = bin_dir.to_string_lossy().to_string();
        let current = env.get("PATH").cloned().unwrap_or_default();
        if !current.contains(&bin_str) {
            env.insert("PATH".to_string(), format!("{}:{}", bin_str, current));
        }
    }

    env.insert("CABEXTRACT".to_string(), "/opt/homebrew/bin/cabextract".to_string());
    env.insert("WINE".to_string(), runner.wine_bin_path().to_string_lossy().to_string());
    env.insert("WINEPREFIX".to_string(), prefix.path.to_string_lossy().to_string());

    // For winetricks, use wow64 arch if backend supports it (needed for 32-bit verbs like vcrun, dotnet)
    if runner.backend().has_wo64() {
        env.insert("WINEARCH".to_string(), "wow64".to_string());
    }

    tracing::debug!(wine = %env.get("WINE").unwrap_or(&String::new()), "winetricks env WINE");
    tracing::debug!(wineprefix = %env.get("WINEPREFIX").unwrap_or(&String::new()), "winetricks env WINEPREFIX");
    tracing::debug!(winearch = %env.get("WINEARCH").unwrap_or(&String::new()), "winetricks env WINEARCH");

    let output = std::process::Command::new("/bin/bash")
        .arg(&patched_winetricks)
        .arg(verb)
        .envs(&env)
        .output()
        .map_err(|e| MoonshineError::Io(e))?;

    // Clean up patched winetricks
    let _ = fs::remove_file(&patched_winetricks);

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    tracing::debug!(stdout = %stdout, stderr = %stderr, "winetricks completed");

    if !output.status.success() {
        // Provide actionable error messages
        if stderr.contains("command not found") || stderr.contains("No such file") {
            return Err(MoonshineError::Config(
                "winetricks failed: missing dependency.\n\n\
                 Install required tools:\n\
                 • brew install cabextract\n\
                 • brew install wget".to_string()
            ));
        }
        if stderr.contains("wine") && stderr.contains("not found") {
            return Err(MoonshineError::Config(
                "winetricks failed: wine binary not found.\n\n\
                 Make sure Wine is installed:\n\
                 • brew install --cask wine-stable".to_string()
            ));
        }
        return Err(MoonshineError::WineProcessFailed(output.status.code()));
    }

    Ok(stdout)
}

/// Dependency presets for common game launchers.
/// Each preset installs multiple winetricks verbs in sequence so the user
/// doesn't have to install them one by one.
pub fn run_winetricks_preset(prefix: &Prefix, preset: &str) -> Result<String> {
    let (verbs, label) = match preset {
        "steam" => (vec!["vcrun2019", "corefonts"], "Steam"),
        "epic" => (vec!["vcrun2019", "corefonts", "dotnet48"], "Epic Games"),
        "gog" => (vec!["vcrun2019", "corefonts"], "GOG Galaxy"),
        "gaming-basic" => (vec!["vcrun2019", "d3dcompiler_47", "xinput"], "Gaming (basic)"),
        "gaming-full" => (vec!["vcrun2019", "d3dcompiler_47", "dxvk", "xact", "xinput"], "Gaming (full)"),
        _ => return Err(MoonshineError::Config(format!("Unknown preset: {}", preset))),
    };

    tracing::info!(preset = %label, verbs = ?verbs, prefix = %prefix.name, "Running winetricks preset");

    let mut results = Vec::new();
    for verb in &verbs {
        tracing::info!(verb = %verb, "Preset step: installing {}", verb);
        match run_winetricks(prefix, verb) {
            Ok(_output) => {
                results.push(format!("✓ {} — installed", verb));
            }
            Err(e) => {
                // Don't abort the whole preset on one failure — report it and continue
                tracing::warn!(verb = %verb, error = %e, "Preset step failed");
                results.push(format!("✗ {} — failed: {}", verb, e));
            }
        }
    }

    Ok(results.join("\n"))
}

/// Create a patched copy of winetricks that fixes the syswow64 regedit path issue.
/// GPTK's wine64 doesn't populate syswow64 with 32-bit executables, so winetricks
/// fails when trying to use C:\windows\syswow64\regedit.exe.
/// This patches the script to use C:\windows\regedit.exe (64-bit) instead.
fn create_patched_winetricks(original: &std::path::Path, _prefix: &Prefix) -> Result<std::path::PathBuf> {
    let installers_dir = get_installers_dir()?;
    let patched_path = installers_dir.join("winetricks_patched");

    // Read original winetricks
    let content = fs::read_to_string(original)
        .map_err(|e| MoonshineError::Io(e))?;

    // Patch: Replace syswow64\regedit.exe with the 64-bit regedit from windows directory
    // winetricks uses: "$WINE" C:\\windows\\syswow64\\regedit.exe
    // We change it to: "$WINE" C:\\windows\\regedit.exe
    let patched = content
        .replace(
            "C:\\\\windows\\\\syswow64\\\\regedit.exe",
            "C:\\\\windows\\\\regedit.exe"
        )
        .replace(
            "C:\\windows\\syswow64\\regedit.exe",
            "C:\\windows\\regedit.exe"
        )
        // Also fix rundll32.exe path if used
        .replace(
            "C:\\\\windows\\\\syswow64\\\\rundll32.exe",
            "C:\\\\windows\\\\rundll32.exe"
        )
        .replace(
            "C:\\windows\\syswow64\\rundll32.exe",
            "C:\\windows\\rundll32.exe"
        );

    fs::write(&patched_path, patched)
        .map_err(|e| MoonshineError::Io(e))?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&patched_path, fs::Permissions::from_mode(0o755));
    }

    tracing::debug!(path = %patched_path.display(), "Created patched winetricks");
    Ok(patched_path)
}

pub fn list_available_verbs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("steam", "Install Steam client"),
        ("vcrun2019", "Visual C++ 2019 Redistributable"),
        ("vcrun2022", "Visual C++ 2022 Redistributable"),
        ("dotnet48", ".NET Framework 4.8"),
        ("dotnet40", ".NET Framework 4.0"),
        ("dxvk", "DXVK (DirectX to Vulkan)"),
        ("d3dcompiler_47", "Direct3D Compiler 4.7"),
        ("xact", "XACT Audio Engine"),
        ("xinput", "XInput (gamepad support)"),
        ("corefonts", "Microsoft Core Fonts"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_games_from_local_steam_manifest() {
        let root = std::env::temp_dir().join(format!("moonshine-steam-{}", uuid::Uuid::new_v4()));
        let prefix = Prefix::new("Steam Test", &root).unwrap();
        let steam_root = prefix.drive_c().join("Program Files (x86)/Steam");
        let game_dir = steam_root.join("steamapps/common/Test Game");
        fs::create_dir_all(&game_dir).unwrap();
        fs::write(steam_root.join("steam.exe"), b"steam").unwrap();
        fs::write(
            steam_root.join("steamapps/appmanifest_123.acf"),
            "\"AppState\"\n{\n\t\"name\"\t\"Test Game\"\n\t\"installdir\"\t\"Test Game\"\n}",
        )
        .unwrap();
        fs::write(game_dir.join("TestGame.exe"), b"game").unwrap();

        let games = discover_steam_games(&prefix).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].app_id, "123");
        assert_eq!(games[0].name, "Test Game");
        assert_eq!(games[0].executable_candidates.len(), 1);
        assert_eq!(games[0].profiles.len(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn discovery_requires_an_installed_client() {
        let root = std::env::temp_dir().join(format!("moonshine-steam-{}", uuid::Uuid::new_v4()));
        let prefix = Prefix::new("No Steam", &root).unwrap();
        assert!(discover_steam_games(&prefix).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
