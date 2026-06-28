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

    let defaultWineURL = "https://github.com/Gcenx/game-porting-toolkit/releases/download/Game-Porting-Toolkit-3.0-3/game-porting-toolkit-3.0-3.tar.xz"

    init() {
        setup()
    }

    func setup() {
        baseDir = get_base_dir().toString()
        detectRuntime()
        loadPrefixes()
    }

    func detectRuntime() {
        wineDetected = detect_wine() != nil || is_wine_installed()
        wineVersion = wine_version()?.toString()
        gptkInstalled = checkGptkInstalled()
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
                executables: rp.list_executables().map { $0.as_str().toString() }
            )
        }
    }

    func createPrefix(name: String, windowsVersion: String, graphicsBackend: String) {
        // RustPrefix IS a RustPrefixRefMut, so setters are available directly
        let prefix = RustPrefix(name)

        prefix.set_windows_version(windowsVersion == "win10" ? .Win10 : .Win11)
        prefix.set_graphics_backend(graphicsBackend == "d3dmetal" ? .D3DMetal : .DXVK)

        _ = prefix.save()
        loadPrefixes()
    }

    /// Returns a mutable reference to the prefix with the given id,
    /// using index-based getMut to get a RustPrefixRefMut (which has setters).
    private func findPrefixMut(id: String) -> RustPrefixRefMut? {
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
        dxvkHud: Bool
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
        _ = rp.save()
        loadPrefixes()
    }

    func runProgram(prefixId: String, programPath: String) {
        // run_program is on RustPrefixRef (read-only), vec.get(index:) is enough
        let vec = list_all_prefixes()
        for i in 0..<vec.len() {
            if let rp = vec.get(index: UInt(i)) {
                if rp.get_id().toString() == prefixId {
                    Task.detached {
                        _ = rp.run_program(programPath)
                    }
                    break
                }
            }
        }
    }

    func deletePrefix(id: String) {
        if let rp = list_all_prefixes().first(where: { $0.get_id().toString() == id }) {
            _ = rp.delete_prefix()
            loadPrefixes()
        }
    }

    private func checkGptkInstalled() -> Bool {
        let home = FileManager.default.homeDirectoryForCurrentUser
        let gptkPath = home.appendingPathComponent("Library/Application Support/Moonshine/Libraries/Wine/bin/wine64")
        return FileManager.default.fileExists(atPath: gptkPath.path)
    }
}
