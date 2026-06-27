# Moonshine

> Run Windows games on Apple Silicon Macs — natively.

Moonshine is a modern Wine prefix manager for macOS, built with Rust and SwiftUI. It uses Apple's Game Porting Toolkit (GPTK) to translate DirectX games to Metal, so you can play Windows games on your Mac with great performance.

## Features

- Create and manage Wine prefixes (bottles) with a native macOS interface
- Run Windows games via D3DMetal or DXVK
- Automatic Wine/GPTK runtime management *(coming soon)*
- One-click Steam installation *(coming soon)*
- Per-bottle configuration (Windows version, sync mode, DLL overrides)
- Native Apple Silicon performance (arm64)

## Requirements

- macOS 14+ (Sonoma or later)
- Apple Silicon (M1/M2/M3/M4)
- Rust 1.75+ (to build from source)

## Quick Start

```bash
# Build Rust crates
./scripts/build-rust.sh

# Open in Xcode and run
open Moonshine.xcodeproj
```

## Architecture

```
SwiftUI App → swift-bridge FFI → Rust Core → Wine/GPTK → Metal → GPU
```

## Project Structure

```
moonshine/
├── crates/moonshine-core/   # Rust core (prefix mgmt, wine, config)
├── crates/moonshine-ffi/    # Swift-Rust bridge
├── Moonshine/               # SwiftUI app
└── scripts/                 # Build scripts
```

## License

MIT
