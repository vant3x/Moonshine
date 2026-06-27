import SwiftUI

struct PrefixDetailView: View {
    let prefix: PrefixData

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
                    Picker("Windows Version", selection: .constant(prefix.windowsVersion)) {
                        Text("Windows 10").tag("win10")
                        Text("Windows 11").tag("win11")
                    }

                    Picker("Graphics Backend", selection: .constant(prefix.graphicsBackend)) {
                        Text("D3DMetal (Recommended)").tag("d3dmetal")
                        Text("DXVK").tag("dxvk")
                    }

                    Picker("Sync Mode", selection: .constant(prefix.syncMode)) {
                        Text("Default").tag("default")
                        Text("ESync").tag("esync")
                        Text("MSync").tag("msync")
                    }

                    Toggle("MetalFX", isOn: .constant(prefix.metalFx))
                    Toggle("DXVK HUD", isOn: .constant(prefix.dxvkHud))
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
                            }
                        }
                    }
                }
            }
            .padding()
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}
