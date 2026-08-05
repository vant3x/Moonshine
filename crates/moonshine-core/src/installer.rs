use crate::downloader;
use crate::error::{MoonshineError, Result};
use crate::prefix::Prefix;
use crate::wine::WineRunner;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const STEAM_SETUP_URL: &str = "https://cdn.akamai.steamstatic.com/client/installer/SteamSetup.exe";

const WINETRICKS_URL: &str =
    "https://raw.githubusercontent.com/Winetricks/winetricks/master/src/winetricks";

pub fn get_installers_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        MoonshineError::Config("Could not determine home directory".to_string())
    })?;
    let dir = home.join("Library/Application Support/Moonshine/Installers");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn download_steam_setup() -> Result<PathBuf> {
    let dir = get_installers_dir()?;
    let dest = dir.join("SteamSetup.exe");
    if dest.exists() {
        eprintln!("[Moonshine] SteamSetup.exe already cached");
        return Ok(dest);
    }
    eprintln!("[Moonshine] Downloading SteamSetup.exe...");
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
    eprintln!("[Moonshine] Downloading winetricks...");
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
    let runner = WineRunner::detect_for_config(&prefix.config)?;

    // SteamSetup.exe is a 32-bit application — requires WoW64 support.
    // If current backend doesn't have WoW64, try to find one that does.
    let effective_runner = if !runner.backend().has_wo64() {
        eprintln!(
            "[Moonshine] Current backend ({}) doesn't support 32-bit. Looking for WoW64 backend...",
            runner.backend()
        );
        match WineRunner::find_backend_with_wo64() {
            Ok(wo64_runner) => {
                eprintln!(
                    "[Moonshine] Auto-fallback: using {} for Steam installation",
                    wo64_runner.backend()
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

    let setup_exe = download_steam_setup()?;

    eprintln!("[Moonshine] Running SteamSetup.exe in prefix: {}", prefix.name);
    eprintln!("[Moonshine] Wine binary: {}", effective_runner.wine_bin_path().display());
    eprintln!("[Moonshine] Backend: {}", effective_runner.backend());
    eprintln!("[Moonshine] Prefix path: {}", prefix.path.display());

    // Copy to drive_c to avoid path issues with GPTK wine
    let drive_c = prefix.drive_c();
    let dest_exe = drive_c.join("Temp").join("MoonshineSteamSetup.exe");
    fs::create_dir_all(dest_exe.parent().unwrap())?;
    fs::copy(&setup_exe, &dest_exe)?;

    let mut env = WineRunner::build_env(prefix, &prefix.config, &effective_runner.backend());

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
    eprintln!("[Moonshine] Attempting to run SteamSetup.exe directly with wine64...");

    // Method 1: Direct wine64 execution (most reliable for GPTK)
    let output = std::process::Command::new(effective_runner.wine_bin_path())
        .arg(&unix_path)
        .envs(&env)
        .output()
        .map_err(|e| MoonshineError::Io(e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    eprintln!("[Moonshine] SteamSetup direct stdout: {}", stdout);
    eprintln!("[Moonshine] SteamSetup direct stderr: {}", stderr);

    // If direct execution failed, try with start /unix
    if !output.status.success() {
        eprintln!("[Moonshine] Direct execution failed, trying with start /unix...");
        let output2 = std::process::Command::new(effective_runner.wine_bin_path())
            .arg("start")
            .arg("/unix")
            .arg(&unix_path)
            .envs(&env)
            .output()
            .map_err(|e| MoonshineError::Io(e))?;

        let stdout2 = String::from_utf8_lossy(&output2.stdout).to_string();
        let stderr2 = String::from_utf8_lossy(&output2.stderr).to_string();
        eprintln!("[Moonshine] SteamSetup start /unix stdout: {}", stdout2);
        eprintln!("[Moonshine] SteamSetup start /unix stderr: {}", stderr2);

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
    eprintln!("[Moonshine] Stopping wineserver after Steam install...");
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
        eprintln!("[Moonshine] Steam installed successfully at: {}", steam_exe.display());
        Ok(steam_exe.to_string_lossy().to_string())
    } else {
        let alt = prefix.drive_c().join("Program Files/Steam/steam.exe");
        if alt.exists() {
            eprintln!("[Moonshine] Steam installed at: {}", alt.display());
            Ok(alt.to_string_lossy().to_string())
        } else {
            eprintln!("[Moonshine] Steam installer ran but steam.exe not found");
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

    eprintln!("[Moonshine] Launching Steam (detached): {}", steam_exe.display());
    eprintln!("[Moonshine] Controller support: {}", prefix.config.enable_hid_controllers);

    runner.launch_program(prefix, &steam_exe)
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

    eprintln!("[Moonshine] Running winetricks {} in prefix: {}", verb, prefix.name);
    eprintln!("[Moonshine] Wine backend: {}", runner.backend());
    eprintln!("[Moonshine] WoW64 support: {}", runner.backend().has_wo64());

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

    eprintln!("[Moonshine] winetricks env WINE={}", env.get("WINE").unwrap_or(&String::new()));
    eprintln!("[Moonshine] winetricks env WINEPREFIX={}", env.get("WINEPREFIX").unwrap_or(&String::new()));
    eprintln!("[Moonshine] winetricks env WINEARCH={}", env.get("WINEARCH").unwrap_or(&String::new()));

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
    eprintln!("[Moonshine] winetricks stdout: {}", stdout);
    eprintln!("[Moonshine] winetricks stderr: {}", stderr);

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

    eprintln!("[Moonshine] Created patched winetricks at: {}", patched_path.display());
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
