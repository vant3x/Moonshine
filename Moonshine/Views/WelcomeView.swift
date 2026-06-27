import SwiftUI

struct WelcomeView: View {
    @EnvironmentObject var viewModel: AppViewModel

    var body: some View {
        VStack(spacing: 20) {
            Image(systemName: "gamecontroller")
                .font(.system(size: 64))
                .foregroundColor(.accentColor)

            Text("Welcome to Moonshine")
                .font(.largeTitle)
                .fontWeight(.bold)

            Text("Run Windows games on your Mac with Apple Silicon")
                .font(.title3)
                .foregroundColor(.secondary)

            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    Image(systemName: viewModel.wineDetected ? "checkmark.circle.fill" : "xmark.circle.fill")
                        .foregroundColor(viewModel.wineDetected ? .green : .red)
                    Text(viewModel.wineDetected ? "Wine detected: \(viewModel.wineVersion ?? "unknown")" : "Wine not detected")
                }

                HStack {
                    Image(systemName: viewModel.gptkInstalled ? "checkmark.circle.fill" : "xmark.circle.fill")
                        .foregroundColor(viewModel.gptkInstalled ? .green : .red)
                    Text(viewModel.gptkInstalled ? "GPTK installed" : "GPTK not installed")
                }
            }
            .padding()
            .background(RoundedRectangle(cornerRadius: 8).fill(Color.secondary.opacity(0.1)))

            if viewModel.isDownloading {
                ProgressView(viewModel.downloadStatus)
                    .padding()
            }

            if !viewModel.wineDetected && !viewModel.isDownloading {
                Button("Download Wine (ARM64)") {
                    viewModel.installWine(url: viewModel.defaultWineURL)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
            }

            if viewModel.wineDetected && viewModel.prefixes.isEmpty {
                Button("Create your first prefix") {
                    viewModel.showNewPrefixSheet = true
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
            }

            if !viewModel.downloadStatus.isEmpty && !viewModel.isDownloading {
                Text(viewModel.downloadStatus)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding()
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
