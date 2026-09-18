import SwiftUI
import UniformTypeIdentifiers

struct PrefixDetailView: View {
    @EnvironmentObject var viewModel: AppViewModel
    let prefix: PrefixData

    @State private var windowsVersion = "win10"
    @State private var graphicsBackend = "d3dmetal"
    @State private var syncMode = "default"
    @State private var metalFx = false
    @State private var dxvkHud = false
    @State private var wineBackend = "auto"
    @State private var enableHidControllers = true
    @State private var reduceWineDebug = true
    @State private var isWorking = false
    @State private var workStatus = ""
    @State private var showDeleteConfirm = false
    @State private var showReinitConfirm = false
    @State private var steamInstalled = false

    private var hasSteamInstalled: Bool {
        steamInstalled
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {

                // MARK: - Header
                VStack(alignment: .leading, spacing: 4) {
                    Text(prefix.name)
                        .font(.title)
                        .fontWeight(.bold)
                    Text("ID: \(prefix.id)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding(.horizontal)

                // MARK: - Wine Backend
                GroupBox("Wine Backend") {
                    VStack(alignment: .leading, spacing: 8) {
                        Picker("Backend", selection: $wineBackend) {
                            Text("Auto (best available)").tag("auto")
                            Text("WineHQ - 32-bit support (Steam, winetricks)").tag("winehq")
                            Text("GPTK - 64-bit gaming (best performance)").tag("gptk")
                            Text("CrossOver - Commercial (full WoW64)").tag("crossover")
                            Text("Custom path").tag("custom")
                        }

                        if wineBackend == "auto" {
                            HStack {
                                Text("Active:")
                                Text(viewModel.backendName)
                                    .foregroundColor(viewModel.backendWo64 ? .green : .orange)
                                    .fontWeight(.medium)
                                if viewModel.backendWo64 {
                                    Text("(WoW64)")
                                        .font(.caption)
                                        .foregroundColor(.green)
                                }
                            }
                            .font(.caption)
                        }

                        if wineBackend == "gptk" {
                            Text("GPTK: Best performance for 64-bit games. Steam is 32-bit, may need WineHQ fallback.")
                                .font(.caption)
                                .foregroundColor(.orange)
                        } else if wineBackend == "winehq" {
                            Text("WineHQ: Full 32-bit support. Steam + most Windows apps work. Slightly slower than GPTK.")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        } else {
                            Text("Use WineHQ for Steam/winetricks (32-bit). Use GPTK for gaming (best performance).")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }

                // MARK: - Configuration
                GroupBox("Configuration") {
                    VStack(alignment: .leading, spacing: 8) {
                        Picker("Windows Version", selection: $windowsVersion) {
                            Text("Windows 10").tag("win10")
                            Text("Windows 11").tag("win11")
                        }

                        Picker("Graphics Backend", selection: $graphicsBackend) {
                            Text("D3DMetal (Recommended)").tag("d3dmetal")
                            Text("DXVK").tag("dxvk")
                        }

                        Picker("Sync Mode", selection: $syncMode) {
                            Text("Default").tag("default")
                            Text("ESync").tag("esync")
                            Text("MSync").tag("msync")
                        }

                        Toggle("MetalFX", isOn: $metalFx)
                        Toggle("DXVK HUD", isOn: $dxvkHud)

                        Button("Save Configuration") {
                            viewModel.updatePrefixSettings(
                                id: prefix.id,
                                windowsVersion: windowsVersion,
                                graphicsBackend: graphicsBackend,
                                syncMode: syncMode,
                                metalFx: metalFx,
                                dxvkHud: dxvkHud,
                                wineBackend: wineBackend
                            )
                        }
                        .buttonStyle(.borderedProminent)
                        .padding(.top, 4)
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }

                // MARK: - Controllers & Input
                GroupBox("Controllers & Input") {
                    VStack(alignment: .leading, spacing: 8) {
                        Toggle(isOn: $enableHidControllers) {
                            VStack(alignment: .leading, spacing: 2) {
                                Label("HID Controller Support", systemImage: "gamecontroller.fill")
                                Text("Enables GameSir Nova, Xbox, PS5, DualSense and other USB/Bluetooth gamepads via XInput.")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }

                        Toggle(isOn: $reduceWineDebug) {
                            VStack(alignment: .leading, spacing: 2) {
                                Label("Reduce Wine Debug Logs", systemImage: "speedometer")
                                Text("Sets WINEDEBUG=-all,+err,+warn for better performance. Disable only when debugging crashes.")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }

                        Button("Save Controller Settings") {
                            viewModel.updateControllerSettings(
                                id: prefix.id,
                                enableHid: enableHidControllers,
                                reduceWineDebug: reduceWineDebug
                            )
                        }
                        .buttonStyle(.bordered)

                        if enableHidControllers {
                            VStack(alignment: .leading, spacing: 4) {
                                Text("Controller Setup Tips:")
                                    .font(.caption)
                                    .fontWeight(.semibold)
                                    .foregroundColor(.secondary)
                                Text("1. Connect GameSir Nova in PC/XInput mode (not Nintendo mode)")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                                Text("2. Install xinput via winetricks (Install Dependencies below)")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                                Text("3. Launch Steam or game — controller should be detected automatically")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            .padding(.top, 4)
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }

                // MARK: - Install Software
                GroupBox("Install Software") {
                    VStack(alignment: .leading, spacing: 8) {
                        if viewModel.hasWineMsvcrtBug {
                            HStack {
                                Image(systemName: "exclamationmark.triangle.fill")
                                    .foregroundColor(.red)
                                Text("Wine 11.0 bug detected. Install may fail. Check Settings to fix.")
                                    .font(.caption)
                                    .foregroundColor(.red)
                            }
                        }

                        HStack {
                            if !hasSteamInstalled {
                                Button(action: { viewModel.installSteamInBackground(id: prefix.id) }) {
                                    Label("Install Steam", systemImage: "arrow.down.circle")
                                }
                                .disabled(viewModel.isInstalling || viewModel.isInitializing)
                            } else if viewModel.isSteamRunning {
                                Button(action: { viewModel.stopSteam() }) {
                                    Label("Stop Steam (PID \(viewModel.activeSteamPid ?? 0))", systemImage: "stop.fill")
                                }
                                .buttonStyle(.borderedProminent)
                                .tint(.red)
                            } else {
                                Button(action: { viewModel.installSteamInBackground(id: prefix.id) }) {
                                    Label("Reinstall Steam", systemImage: "arrow.counterclockwise")
                                }
                                .disabled(viewModel.isInstalling || viewModel.isInitializing)

                                Button(action: { viewModel.launchSteam(prefixId: prefix.id) }) {
                                    Label("Launch Steam", systemImage: "play.fill")
                                }
                                .buttonStyle(.borderedProminent)
                                .tint(.green)
                            }
                        }

                        Menu("Quick Presets") {
                            Button("🎮 Steam (vcrun2019 + corefonts)") {
                                viewModel.runWinetricksPreset(id: prefix.id, preset: "steam")
                            }
                            Button("🎮 Epic Games (vcrun2019 + corefonts + dotnet48)") {
                                viewModel.runWinetricksPreset(id: prefix.id, preset: "epic")
                            }
                            Button("🎮 GOG Galaxy (vcrun2019 + corefonts)") {
                                viewModel.runWinetricksPreset(id: prefix.id, preset: "gog")
                            }
                            Divider()
                            Button("🎯 Gaming Basic (vcrun2019 + d3dcompiler + xinput)") {
                                viewModel.runWinetricksPreset(id: prefix.id, preset: "gaming-basic")
                            }
                            Button("🎯 Gaming Full (vcrun2019 + d3dcompiler + dxvk + xact + xinput)") {
                                viewModel.runWinetricksPreset(id: prefix.id, preset: "gaming-full")
                            }
                        }
                        .disabled(viewModel.isInstalling || viewModel.isInitializing)

                        Menu("Install Dependencies (winetricks)") {
                            Button("Visual C++ 2019") { viewModel.runWinetricksInBackground(id: prefix.id, verb: "vcrun2019") }
                            Button("Visual C++ 2022") { viewModel.runWinetricksInBackground(id: prefix.id, verb: "vcrun2022") }
                            Button(".NET Framework 4.8") { viewModel.runWinetricksInBackground(id: prefix.id, verb: "dotnet48") }
                            Button("DXVK (DirectX to Vulkan)") { viewModel.runWinetricksInBackground(id: prefix.id, verb: "dxvk") }
                            Button("Direct3D Compiler 4.7") { viewModel.runWinetricksInBackground(id: prefix.id, verb: "d3dcompiler_47") }
                            Button("XInput (gamepad)") { viewModel.runWinetricksInBackground(id: prefix.id, verb: "xinput") }
                            Button("Core Fonts") { viewModel.runWinetricksInBackground(id: prefix.id, verb: "corefonts") }
                        }
                        .disabled(viewModel.isInstalling || viewModel.isInitializing)

                        if viewModel.isInstalling {
                            HStack {
                                ProgressView()
                                    .controlSize(.small)
                                Text(viewModel.installStatus)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        } else if !viewModel.installStatus.isEmpty {
                            let isError = viewModel.installStatus.contains("failed") || viewModel.installStatus.contains("Failed") || viewModel.installStatus.contains("error") || viewModel.installStatus.contains("bug")
                            HStack(alignment: .top, spacing: 6) {
                                Image(systemName: isError ? "exclamationmark.circle.fill" : "checkmark.circle.fill")
                                    .foregroundColor(isError ? .red : .green)
                                    .font(.caption)
                                Text(viewModel.installStatus)
                                    .font(.caption)
                                    .foregroundColor(isError ? .red : .green)
                                Spacer()
                                Button(action: { viewModel.installStatus = "" }) {
                                    Image(systemName: "xmark.circle.fill")
                                        .foregroundColor(.secondary)
                                        .font(.caption)
                                }
                                .buttonStyle(.plain)
                            }
                        }

                        if viewModel.isInitializing {
                            HStack {
                                ProgressView()
                                    .controlSize(.small)
                                Text(viewModel.initStatus)
                                    .font(.caption)
                                    .foregroundColor(.orange)
                            }
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }

                // MARK: - Installed Programs
                GroupBox("Installed Programs") {
                    VStack(alignment: .leading, spacing: 4) {
                        if prefix.executables.isEmpty {
                            Text("No programs found")
                                .foregroundColor(.secondary)
                        } else {
                            // Steam first (most important)
                            ForEach(prefix.executables.filter { $0.lowercased().contains("steam.exe") }, id: \.self) { exe in
                                HStack {
                                    Image(systemName: "gamecontroller")
                                        .foregroundColor(.blue)
                                    Text("Steam")
                                        .fontWeight(.medium)
                                    Spacer()
                                    Button(action: { viewModel.launchSteam(prefixId: prefix.id) }) {
                                        Label("Launch", systemImage: "play.fill")
                                    }
                                    .buttonStyle(.borderedProminent)
                                    .tint(.green)
                                    .controlSize(.small)
                                }
                            }

                            // Other programs
                            let otherExes = prefix.executables.filter { !$0.lowercased().contains("steam.exe") }
                            if !otherExes.isEmpty {
                                ForEach(otherExes, id: \.self) { exe in
                                    HStack {
                                        Image(systemName: "app")
                                        Text(URL(fileURLWithPath: exe).lastPathComponent)
                                        Spacer()
                                        Button("Run") {
                                            viewModel.runProgram(prefixId: prefix.id, programPath: exe)
                                        }
                                        .buttonStyle(.bordered)
                                        .controlSize(.small)
                                    }
                                }
                            }
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }

                // MARK: - Actions
                GroupBox("Actions") {
                    VStack(alignment: .leading, spacing: 8) {
                        Button(action: installCustomExe) {
                            Label("Run Installer (.exe / .msi)...", systemImage: "square.and.arrow.down")
                        }

                        Button(action: openPrefixFolder) {
                            Label("Open Prefix Folder", systemImage: "folder")
                        }

                        Button(action: { showReinitConfirm = true }) {
                            Label("Reinitialize Prefix (fix corrupted)", systemImage: "arrow.clockwise")
                                .foregroundColor(.orange)
                        }

                        Button(action: { showDeleteConfirm = true }) {
                            Label("Delete Prefix", systemImage: "trash")
                                .foregroundColor(.red)
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }

                // Bottom padding for scroll
                Color.clear.frame(height: 16)
            }
            .padding(.horizontal)
            .padding(.top)
        }
        .overlay {
            if isWorking {
                VStack {
                    ProgressView()
                        .controlSize(.large)
                    Text(workStatus)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .padding()
                .background(.ultraThinMaterial)
                .cornerRadius(8)
            }
        }
        .alert("Delete Prefix", isPresented: $showDeleteConfirm) {
            Button("Cancel", role: .cancel) { }
            Button("Delete", role: .destructive) {
                viewModel.deletePrefix(id: prefix.id)
            }
        } message: {
            Text("Are you sure you want to delete '\(prefix.name)'? This cannot be undone.")
        }
        .alert("Reinitialize Prefix", isPresented: $showReinitConfirm) {
            Button("Cancel", role: .cancel) { }
            Button("Reinitialize") {
                reinitPrefix()
            }
        } message: {
            Text("This will delete and recreate the Wine prefix for '\(prefix.name)'. Installed programs will be lost, but you can reinstall them.")
        }
        .onAppear {
            DispatchQueue.main.async {
                loadState()
            }
        }
        .onChange(of: prefix.id) { _ in
            DispatchQueue.main.async {
                loadState()
            }
        }
    }

    private func loadState() {
        windowsVersion = prefix.windowsVersion
        graphicsBackend = prefix.graphicsBackend
        syncMode = prefix.syncMode
        metalFx = prefix.metalFx
        dxvkHud = prefix.dxvkHud
        wineBackend = prefix.wineBackend
        enableHidControllers = prefix.enableHidControllers
        reduceWineDebug = prefix.reduceWineDebug
        // Check if Steam is installed in this prefix
        steamInstalled = prefix.executables.contains { $0.contains("steam.exe") }
    }

    private func installCustomExe() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        // Accept both .exe and .msi installers (Epic Games uses .msi)
        var types: [UTType] = []
        if let exeType = UTType(filenameExtension: "exe") {
            types.append(exeType)
        }
        if let msiType = UTType(filenameExtension: "msi") {
            types.append(msiType)
        }
        panel.allowedContentTypes = types

        if panel.runModal() == .OK {
            if let url = panel.url {
                viewModel.runProgram(prefixId: prefix.id, programPath: url.path)
            }
        }
    }

    private func openPrefixFolder() {
        if let rp = viewModel.findPrefixMut(id: prefix.id) {
            let path = rp.get_path().toString()
            NSWorkspace.shared.selectFile(nil, inFileViewerRootedAtPath: path)
        }
    }

    private func reinitPrefix() {
        isWorking = true
        workStatus = "Reinitializing prefix..."
        Task.detached {
            let vec = list_all_prefixes()
            for i in 0..<vec.len() {
                if let rp = vec.getMut(index: UInt(i)) {
                    if rp.get_id().toString() == prefix.id {
                        let result = rp.reinit_prefix().toString()
                        await MainActor.run {
                            isWorking = false
                            workStatus = result
                            viewModel.loadPrefixes()
                        }
                        break
                    }
                }
            }
        }
    }
}
