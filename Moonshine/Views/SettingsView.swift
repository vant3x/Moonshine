import SwiftUI

struct SettingsView: View {
    @EnvironmentObject var viewModel: AppViewModel

    var body: some View {
        Form {
            Section("Runtime") {
                HStack {
                    Text("Wine:")
                    Spacer()
                    Text(viewModel.wineDetected ? viewModel.wineVersion ?? "Detected" : "Not found")
                        .foregroundColor(viewModel.wineDetected ? .green : .red)
                }

                HStack {
                    Text("GPTK:")
                    Spacer()
                    Text(viewModel.gptkInstalled ? "Installed" : "Not installed")
                        .foregroundColor(viewModel.gptkInstalled ? .green : .red)
                }
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
        .frame(width: 450, height: 300)
    }
}
