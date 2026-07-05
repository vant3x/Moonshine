import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var viewModel: AppViewModel
    @State private var customWinePath = ""
    @State private var isFixingWine = false
    @State private var fixWineStatus = ""
    @State private var showFixConfirm = false

    var body: some View {
        Form {
            if viewModel.hasWineMsvcrtBug {
                Section {
                    HStack {
                        Image(systemName: "exclamationmark.triangle.fill")
                            .foregroundColor(.red)
                        Text("Wine 11.0 msvcrt bug detected!")
                            .fontWeight(.bold)
                            .foregroundColor(.red)
                    }
                    Text("Wine 11.0.1 on macOS has a critical bug that crashes ALL wine commands.")
                        .font(.caption)
                        .foregroundColor(.secondary)

                    if isFixingWine {
                        HStack {
                            ProgressView()
                                .controlSize(.small)
                            Text(fixWineStatus)
                                .font(.caption)
                                .foregroundColor(.orange)
                        }
                    } else {
                        Button(action: { showFixConfirm = true }) {
                            Label("Fix Wine (install wine@devel 11.10)", systemImage: "wrench.fill")
                        }
                        .buttonStyle(.borderedProminent)
                    }

                    Text("This will install wine@devel (11.10) which fixes the msvcrt bug.")
                        .font(.system(.caption, design: .monospaced))
                        .foregroundColor(.secondary)
                    Text("brew install --cask wine@devel")
                        .font(.system(.caption, design: .monospaced))
                        .foregroundColor(.orange)
                }
                .alert("Fix Wine?", isPresented: $showFixConfirm) {
                    Button("Cancel", role: .cancel) { }
                    Button("Fix Wine", role: .destructive) {
                        fixWine()
                    }
                } message: {
                    Text("This will install Wine 11.10 (wine@devel) which fixes the crash bug. May take a few minutes.")
                }
            }

            Section("Active Backend") {
                HStack {
                    Text("Backend:")
                    Spacer()
                    Text(viewModel.backendName)
                        .foregroundColor(.green)
                        .fontWeight(.medium)
                }

                HStack {
                    Text("WoW64 Support:")
                    Spacer()
                    Text(viewModel.backendWo64 ? "Yes (32-bit apps work)" : "No (32-bit apps will fail)")
                        .foregroundColor(viewModel.backendWo64 ? .green : .orange)
                }

                HStack {
                    Text("Wine Version:")
                    Spacer()
                    Text(viewModel.wineVersion ?? "Unknown")
                        .foregroundColor(.secondary)
                }

                if !viewModel.backendWo64 {
                    Text("Tip: Install CrossOver or WineHQ (brew install --cask wine-stable) for full 32-bit support")
                        .font(.caption)
                        .foregroundColor(.orange)
                }
            }

            Section("Available Backends") {
                if viewModel.availableBackends.isEmpty {
                    Text("No Wine backends detected")
                        .foregroundColor(.red)
                } else {
                    ForEach(viewModel.availableBackends, id: \.name) { backend in
                        HStack {
                            VStack(alignment: .leading) {
                                Text(backend.name)
                                    .fontWeight(.medium)
                                Text(backend.version)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            Spacer()
                            if backend.hasWo64 {
                                Text("WoW64")
                                    .font(.caption)
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 2)
                                    .background(Color.green.opacity(0.2))
                                    .cornerRadius(4)
                            } else {
                                Text("64-bit only")
                                    .font(.caption)
                                    .padding(.horizontal, 8)
                                    .padding(.vertical, 2)
                                    .background(Color.orange.opacity(0.2))
                                    .cornerRadius(4)
                            }
                        }
                    }
                }
            }

            Section("Custom Wine Path (optional)") {
                TextField("e.g. /Applications/CrossOver.app/Contents/Frameworks/.../bin/wine64", text: $customWinePath)
                    .textFieldStyle(.roundedBorder)
                    .onAppear {
                        customWinePath = viewModel.customWinePath
                    }

                HStack {
                    Button("Save") {
                        viewModel.setCustomWinePath(customWinePath)
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(customWinePath == viewModel.customWinePath)

                    if !customWinePath.isEmpty {
                        Button("Clear") {
                            customWinePath = ""
                            viewModel.setCustomWinePath("")
                        }
                        .buttonStyle(.bordered)
                    }
                }

                Text("Leave empty to use auto-detection (best available backend)")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Section("Paths") {
                HStack {
                    Text("Prefixes:")
                    Spacer()
                    Text(viewModel.baseDir)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding()
        .frame(width: 550, height: 500)
        .onAppear {
            viewModel.detectBackends()
        }
    }

    private func fixWine() {
        guard !isFixingWine else { return }
        isFixingWine = true
        fixWineStatus = "Installing wine@devel (11.10)..."

        Task.detached {
            // Install wine@devel which has the msvcrt fix
            let install = Process()
            install.executableURL = URL(fileURLWithPath: "/opt/homebrew/bin/brew")
            install.arguments = ["install", "--cask", "wine@devel"]
            let pipe = Pipe()
            install.standardOutput = pipe
            install.standardError = pipe
            try? install.run()
            install.waitUntilExit()

            let output = String(data: pipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
            print("[Moonshine] wine@devel install output: \(output)")

            await MainActor.run {
                isFixingWine = false
                viewModel.detectRuntime()
                viewModel.detectBackends()

                if viewModel.hasWineMsvcrtBug {
                    fixWineStatus = "Wine still has the bug. Try: brew info --cask wine@devel"
                } else {
                    fixWineStatus = "Wine fixed! \(viewModel.wineVersion ?? "")"
                }
            }
        }
    }
}
