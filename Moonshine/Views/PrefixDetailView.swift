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
                            dxvkHud: dxvkHud
                        )
                    }
                    .buttonStyle(.borderedProminent)
                    .padding(.top, 4)
                }

                Section("Installed Programs") {
                    if prefix.executables.isEmpty {
                        Text("No programs found")
                            .foregroundColor(.secondary)
                    } else {
                        ForEach(prefix.executables, id: \.self) { exe in
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
                
                Section("Actions") {
                    Button(action: installCustomExe) {
                        Label("Run Custom EXE / Installer...", systemImage: "square.and.arrow.down")
                    }
                    
                    Button(action: deletePrefix) {
                        Label("Delete Prefix", systemImage: "trash")
                            .foregroundColor(.red)
                    }
                }
            }
            .padding()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .onAppear {
            loadState()
        }
        .onChange(of: prefix.id) { _ in
            loadState()
        }
    }

    private func loadState() {
        windowsVersion = prefix.windowsVersion
        graphicsBackend = prefix.graphicsBackend
        syncMode = prefix.syncMode
        metalFx = prefix.metalFx
        dxvkHud = prefix.dxvkHud
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

    private func deletePrefix() {
        viewModel.deletePrefix(id: prefix.id)
    }
}
