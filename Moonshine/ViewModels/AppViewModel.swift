import Combine
import Foundation
import SwiftUI
import Darwin

struct PrefixData: Identifiable {
    let id: String
    let name: String
    let windowsVersion: String
    let graphicsBackend: String
    let syncMode: String
    let metalFx: Bool
    let dxvkHud: Bool
    let executables: [String]
    let wineBackend: String
}

@MainActor
class AppViewModel: ObservableObject {
    @Published var prefixes: [PrefixData] = []
    @Published var selectedPrefixId: String?
    @Published var showNewPrefixSheet = false
    @Published var wineDetected = false
    @Published var wineVersion: String?
    @Published var gptkInstalled = false
    @Published var baseDir = ""
    @Published var isDownloading = false
    @Published var downloadStatus = ""
    @Published var isInstalling = false
    @Published var installStatus = ""
    @Published var isInitializing = false
    @Published var initStatus = ""
    @Published var customWinePath = ""
    @Published var backendName: String = "None"
    @Published var backendWo64: Bool = false
    @Published var availableBackends: [(name: String, hasWo64: Bool, version: String)] = []
    @Published var hasWineMsvcrtBug = false

    let defaultWineURL = "https://github.com/Gcenx/game-porting-toolkit/releases/download/Game-Porting-Toolkit-3.0-3/game-porting-toolkit-3.0-3.tar.xz"

    init() {
        setup()
    }

    func setup() {
        baseDir = get_base_dir().toString()
        detectRuntime()
        detectBackends()
        loadPrefixes()
    }

    func detectRuntime() {
        wineDetected = detect_wine() != nil || is_wine_installed()
        wineVersion = wine_version()?.toString()
        gptkInstalled = checkGptkInstalled()
        hasWineMsvcrtBug = has_wine_msvcrt_bug()
        if hasWineMsvcrtBug {
            print("[Moonshine] WARNING: Wine 11.0 msvcrt bug detected! All wine commands will crash.")
        }
    }

    func detectBackends() {
        let backends = detect_all_wine_backends()
        availableBackends = []
        for i in 0..<backends.len() {
            if let backend = backends.get(index: UInt(i)) {
                let name = backend.get_backend_name().toString()
                let hasWo64 = backend.has_wo64_support()
                let version = backend.get_version()?.toString() ?? "unknown"
                availableBackends.append((name: name, hasWo64: hasWo64, version: version))
            }
        }

        // Get best backend info
        if let best = get_best_wine_backend() {
            backendName = best.get_backend_name().toString()
            backendWo64 = best.has_wo64_support()
            let version = best.get_version()?.toString() ?? "unknown"
            print("[Moonshine] Best backend: \(backendName) (WoW64: \(backendWo64), version: \(version))")
        } else {
            backendName = "None"
            backendWo64 = false
        }
    }

    func installWine(url: String) {
        guard !isDownloading else { return }
        isDownloading = true
        downloadStatus = "Downloading Wine..."
        Task.detached {
            // Capture stderr output from Rust
            let pipe = Pipe()
            let oldStderr = dup(STDERR_FILENO)
            dup2(pipe.fileHandleForWriting.fileDescriptor, STDERR_FILENO)
            
            let success = install_wine(url)
            
            // Restore stderr
            dup2(oldStderr, STDERR_FILENO)
            close(oldStderr)
            
            // Read captured stderr
            pipe.fileHandleForWriting.closeFile()
            let errorData = pipe.fileHandleForReading.readDataToEndOfFile()
            let errorOutput = String(data: errorData, encoding: .utf8) ?? ""
            
            await MainActor.run {
                self.isDownloading = false
                if success {
                    self.downloadStatus = "Wine installed successfully!"
                    self.detectRuntime()
                    self.detectBackends()
                } else {
                    // Extract meaningful error from Rust stderr
                    let lines = errorOutput.components(separatedBy: "\n")
                    let errorLine = lines.last { $0.contains("[Moonshine]") } ?? "Unknown error"
                    self.downloadStatus = "Download failed: \(errorLine)"
                    print("[Moonshine-Swift] Error output:\n\(errorOutput)")
                }
            }
        }
    }

    func loadPrefixes() {
        let rawPrefixes = list_all_prefixes()
        prefixes = rawPrefixes.map { rp in
            PrefixData(
                id: rp.get_id().toString(),
                name: rp.get_name().toString(),
                windowsVersion: rp.get_windows_version() == SwiftWindowsVersion.Win10 ? "win10" : "win11",
                graphicsBackend: rp.get_graphics_backend() == SwiftGraphicsBackend.D3DMetal ? "d3dmetal" : "dxvk",
                syncMode: {
                    switch rp.get_sync_mode() {
                    case SwiftSyncMode.Default: return "default"
                    case SwiftSyncMode.ESync: return "esync"
                    case SwiftSyncMode.MSync: return "msync"
                    }
                }(),
                metalFx: rp.get_metal_fx(),
                dxvkHud: rp.get_dxvk_hud(),
                executables: rp.list_executables().map { $0.as_str().toString() },
                wineBackend: {
                    switch rp.get_wine_backend() {
                    case SwiftWineBackend.Auto: return "auto"
                    case SwiftWineBackend.WineHQ: return "winehq"
                    case SwiftWineBackend.GPTK: return "gptk"
                    case SwiftWineBackend.CrossOver: return "crossover"
                    case SwiftWineBackend.Custom: return "custom"
                    }
                }()
            )
        }
    }

    func createPrefix(name: String, windowsVersion: String, graphicsBackend: String) {
        let prefix = RustPrefix(name)
        prefix.set_windows_version(windowsVersion == "win10" ? .Win10 : .Win11)
        prefix.set_graphics_backend(graphicsBackend == "d3dmetal" ? .D3DMetal : .DXVK)
        _ = prefix.save()
        let prefixId = prefix.get_id().toString()
        loadPrefixes()

        // Initialize prefix with wineboot — MUST complete before Steam/winetricks
        Task.detached {
            await MainActor.run {
                self.isInitializing = true
                self.initStatus = "Initializing prefix (wineboot)..."
            }

            let vec = list_all_prefixes()
            for i in 0..<vec.len() {
                if let rp = vec.get(index: UInt(i)) {
                    if rp.get_id().toString() == prefixId {
                        let success = rp.init_prefix()
                        await MainActor.run {
                            if success {
                                self.initStatus = "Prefix initialized successfully"
                            } else {
                                self.initStatus = "Warning: wineboot had errors (prefix may still work)"
                            }
                            self.isInitializing = false
                            self.loadPrefixes()
                        }
                        break
                    }
                }
            }
        }
    }

    /// Returns a mutable reference to the prefix with the given id,
    /// using index-based getMut to get a RustPrefixRefMut (which has setters).
    func findPrefixMut(id: String) -> RustPrefixRefMut? {
        let vec = list_all_prefixes()
        for i in 0..<vec.len() {
            if let rp = vec.getMut(index: UInt(i)) {
                if rp.get_id().toString() == id {
                    return rp
                }
            }
        }
        return nil
    }

    func updatePrefixSettings(
        id: String,
        windowsVersion: String,
        graphicsBackend: String,
        syncMode: String,
        metalFx: Bool,
        dxvkHud: Bool,
        wineBackend: String
    ) {
        guard let rp = findPrefixMut(id: id) else { return }

        rp.set_windows_version(windowsVersion == "win10" ? .Win10 : .Win11)
        rp.set_graphics_backend(graphicsBackend == "d3dmetal" ? .D3DMetal : .DXVK)

        switch syncMode {
        case "esync": rp.set_sync_mode(.ESync)
        case "msync": rp.set_sync_mode(.MSync)
        default:      rp.set_sync_mode(.Default)
        }

        rp.set_metal_fx(metalFx)
        rp.set_dxvk_hud(dxvkHud)

        switch wineBackend {
        case "winehq": rp.set_wine_backend(.WineHQ)
        case "gptk": rp.set_wine_backend(.GPTK)
        case "crossover": rp.set_wine_backend(.CrossOver)
        case "custom": rp.set_wine_backend(.Custom)
        default: rp.set_wine_backend(.Auto)
        }

        _ = rp.save()
        loadPrefixes()
    }

    func runProgram(prefixId: String, programPath: String) {
        Task.detached {
            let vec = list_all_prefixes()
            for i in 0..<vec.len() {
                if let rp = vec.get(index: UInt(i)) {
                    if rp.get_id().toString() == prefixId {
                        _ = rp.run_program(programPath)
                        break
                    }
                }
            }
        }
    }

    func deletePrefix(id: String) {
        print("[Moonshine] Attempting to delete prefix: \(id)")
        let vec = list_all_prefixes()
        for i in 0..<vec.len() {
            if let rp = vec.get(index: UInt(i)) {
                if rp.get_id().toString() == id {
                    print("[Moonshine] Found prefix, deleting...")
                    let result = rp.delete_prefix()
                    print("[Moonshine] Delete result: \(result)")
                    loadPrefixes()
                    return
                }
            }
        }
        print("[Moonshine] Prefix not found with id: \(id)")
    }

    func installSteamInBackground(id: String) {
        guard !isInstalling && !isInitializing else { return }
        isInstalling = true
        installStatus = "Downloading SteamSetup.exe..."
        let prefixId = id
        Task.detached {
            let vec = list_all_prefixes()
            var targetRp: RustPrefixRef?
            for i in 0..<vec.len() {
                if let rp = vec.get(index: UInt(i)) {
                    if rp.get_id().toString() == prefixId {
                        targetRp = rp
                        break
                    }
                }
            }
            guard let rp = targetRp else {
                await MainActor.run {
                    self.isInstalling = false
                    self.installStatus = "Prefix not found"
                }
                return
            }
            let result = rp.install_steam().toString()
            await MainActor.run {
                self.isInstalling = false
                if result.contains("failed") || result.contains("Failed") {
                    self.installStatus = result
                } else {
                    self.installStatus = "Steam installed successfully! Click 'Launch Steam' to start."
                    self.loadPrefixes()
                }
            }
        }
    }

    func launchSteam(prefixId: String) {
        Task.detached {
            let vec = list_all_prefixes()
            for i in 0..<vec.len() {
                if let rp = vec.get(index: UInt(i)) {
                    if rp.get_id().toString() == prefixId {
                        let success = rp.launch_steam()
                        await MainActor.run {
                            if !success {
                                self.installStatus = "Failed to launch Steam. Try running it manually."
                            }
                        }
                        break
                    }
                }
            }
        }
    }

    func runWinetricksInBackground(id: String, verb: String) {
        guard !isInstalling && !isInitializing else { return }
        isInstalling = true
        installStatus = "Running winetricks \(verb)..."
        let prefixId = id
        let btVerb = verb
        Task.detached {
            let vec = list_all_prefixes()
            var targetRp: RustPrefixRef?
            for i in 0..<vec.len() {
                if let rp = vec.get(index: UInt(i)) {
                    if rp.get_id().toString() == prefixId {
                        targetRp = rp
                        break
                    }
                }
            }
            guard let rp = targetRp else {
                await MainActor.run {
                    self.isInstalling = false
                    self.installStatus = "Prefix not found"
                }
                return
            }
            let result = rp.run_winetricks(btVerb).toString()
            await MainActor.run {
                self.isInstalling = false
                if result.contains("failed") || result.contains("Failed") {
                    self.installStatus = "\(btVerb) failed"
                } else {
                    self.installStatus = "\(btVerb) installed successfully"
                }
            }
        }
    }

    private func checkGptkInstalled() -> Bool {
        let home = FileManager.default.homeDirectoryForCurrentUser
        let gptkPath = home.appendingPathComponent("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64")
        return FileManager.default.fileExists(atPath: gptkPath.path)
    }

    func setCustomWinePath(_ path: String) {
        customWinePath = path
        // Apply to all prefixes
        let vec = list_all_prefixes()
        for i in 0..<vec.len() {
            if let rp = vec.getMut(index: UInt(i)) {
                rp.set_wine_path(path)
                _ = rp.save()
            }
        }
        loadPrefixes()
        detectRuntime()
    }
}
