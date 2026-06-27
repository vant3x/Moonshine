import SwiftUI

@main
struct MoonshineApp: App {
    @StateObject private var viewModel = AppViewModel()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(viewModel)
        }
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("New Prefix") {
                    viewModel.showNewPrefixSheet = true
                }
                .keyboardShortcut("n", modifiers: .command)
            }
        }

        #if os(macOS)
        Settings {
            SettingsView()
                .environmentObject(viewModel)
        }
        #endif
    }
}
