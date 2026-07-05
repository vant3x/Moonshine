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
    @State private var isWorking = false
    @State private var workStatus = ""
    @State private var showDeleteConfirm = false
    @State private var showReinitConfirm = false
    @State private var steamInstalled = false

    private var hasSteamInstalled: Bool {
        steamInstalled
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                VStack(alignment: .leading) {
                    Text(prefix.name)
                        .font(.largeTitle)
                        .fontWeight(.bold)
                    Text("ID: \(prefix.id)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                Spacer()
            }
            .padding()

            Divider()

            Form {
                Section("Wine Backend") {
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

                Section("Configuration") {
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

                Section("Install Software") {
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
                        Button(action: { viewModel.installSteamInBackground(id: prefix.id) }) {
                            Label("Install Steam", systemImage: "arrow.down.circle")
                        }
                        .disabled(viewModel.isInstalling || viewModel.isInitializing)

                        // Show Launch Steam if steam.exe exists
                        if hasSteamInstalled {
                            Button(action: { viewModel.launchSteam(prefixId: prefix.id) }) {
                                Label("Launch Steam", systemImage: "play.fill")
                            }
                            .buttonStyle(.borderedProminent)
                            .tint(.green)
                        }
                    }

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
                        // Show last operation result (especially errors)
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

                Section("Installed Programs") {
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
                
                Section("Actions") {
                    Button(action: installCustomExe) {
                        Label("Run Custom EXE / Installer...", systemImage: "square.and.arrow.down")
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
            }
            .padding()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
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
        // Check if Steam is installed in this prefix
        steamInstalled = prefix.executables.contains { $0.contains("steam.exe") }
    }

    private func installCustomExe() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        if let exeType = UTType(filenameExtension: "exe") {
            panel.allowedContentTypes = [exeType]
        }
        
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
