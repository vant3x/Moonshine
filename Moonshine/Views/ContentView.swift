import SwiftUI

struct ContentView: View {
    @EnvironmentObject var viewModel: AppViewModel

    var body: some View {
        NavigationSplitView {
            List(selection: $viewModel.selectedPrefixId) {
                Section("Prefixes") {
                    ForEach(viewModel.prefixes) { prefix in
                        NavigationLink(value: prefix.id) {
                            Label(prefix.name, systemImage: "applescript")
                        }
                    }
                }
            }
            .navigationTitle("Moonshine")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button(action: { viewModel.showNewPrefixSheet = true }) {
                        Image(systemName: "plus")
                    }
                }
            }
        } detail: {
            if let selectedId = viewModel.selectedPrefixId,
               let prefix = viewModel.prefixes.first(where: { $0.id == selectedId }) {
                PrefixDetailView(prefix: prefix)
            } else {
                WelcomeView()
            }
        }
        .sheet(isPresented: $viewModel.showNewPrefixSheet) {
            NewPrefixView()
                .environmentObject(viewModel)
        }
    }
}
