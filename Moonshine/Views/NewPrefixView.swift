import SwiftUI

struct NewPrefixView: View {
    @EnvironmentObject var viewModel: AppViewModel
    @Environment(\.dismiss) var dismiss

    @State private var name = ""
    @State private var windowsVersion = "win10"
    @State private var graphicsBackend = "d3dmetal"
    @State private var isCreating = false

    var body: some View {
        VStack(spacing: 20) {
            Text("Create New Prefix")
                .font(.title2)
                .fontWeight(.bold)

            Form {
                TextField("Name", text: $name)
                    .textFieldStyle(.roundedBorder)

                Picker("Windows Version", selection: $windowsVersion) {
                    Text("Windows 10").tag("win10")
                    Text("Windows 11").tag("win11")
                }

                Picker("Graphics Backend", selection: $graphicsBackend) {
                    Text("D3DMetal (Recommended)").tag("d3dmetal")
                    Text("DXVK").tag("dxvk")
                }
            }
            .formStyle(.grouped)

            HStack {
                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.cancelAction)

                Button("Create") {
                    isCreating = true
                    let prefixName = name
                    let winVer = windowsVersion
                    let gfxBackend = graphicsBackend
                    dismiss()
                    DispatchQueue.main.async {
                        viewModel.createPrefix(
                            name: prefixName,                 
                            windowsVersion: winVer,
                            graphicsBackend: gfxBackend
                        )
                    }
                }
                .buttonStyle(.borderedProminent)
                .disabled(name.isEmpty || isCreating)
                .keyboardShortcut(.defaultAction)
            }
        }
        .padding()
        .frame(width: 400, height: 300)
    }
}
