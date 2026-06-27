import Combine
import Foundation
import SwiftUI

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
            let success = install_wine(url)
            await MainActor.run {
                self.isDownloading = false
                if success {
                    self.downloadStatus = "Wine installed successfully!"
                    self.detectRuntime()
                } else {
                    self.downloadStatus = "Download failed. Check the URL and try again."
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
        let prefix = RustPrefix(name)
        _ = prefix.save()
        loadPrefixes()
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
