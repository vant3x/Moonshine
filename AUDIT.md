# Moonshine Repository Audit

Date: 2026-09-18
Scope: TASK 001 baseline and TASK 002 design input.

## Architecture Overview

- SwiftUI entry point: `Moonshine/MoonshineApp.swift`.
- UI state and orchestration: `Moonshine/ViewModels/AppViewModel.swift` (`@MainActor`).
- Swift/Rust bridge: `crates/moonshine-ffi/src/lib.rs`, generated outputs under `Moonshine/Bridge/Generated/`.
- Domain crate: `crates/moonshine-core/`, with modules for configuration, prefixes, runtime/downloads, installers, PE parsing, and Wine execution.
- Native build: `scripts/build-rust.sh` builds `moonshine-ffi` for arm64; Xcode links the checked-in `Moonshine/Bridge/libmoonshine_ffi.a` through `LIBRARY_SEARCH_PATHS` and `-lmoonshine_ffi`.

## Feature Matrix

| Area | Current implementation | Status |
|---|---|---|
| SwiftUI shell | SwiftUI app, views, main-actor view model | Implemented |
| Prefix creation/listing/config | Rust `Prefix`, `BottleConfig`, JSON persistence | Implemented, weak error propagation |
| Wine backend discovery | WineHQ, GPTK, CrossOver, Whisky and bundled paths | Implemented, environment-dependent |
| Wine/GPTK execution | Environment construction, wineboot, foreground and detached launch | Implemented, lifecycle limited |
| Steam installer/launcher | Downloads SteamSetup.exe, installs and launches | Implemented, not independently integration-tested |
| D3DMetal/DXVK | Environment variables and DLL overrides | Partial; payloads are not present in this checkout |
| Process lifecycle | PID returned to Swift and SIGTERM helper | Partial and unsafe |
| Persistence | `bottle.json` load/save | Implemented, non-atomic before TASK 002 change |
| Tests | Rust unit tests | 67 passing baseline; no Swift/UI or Wine integration tests |
| CI | None found | Missing |

## Confirmed Bugs

1. `RustPrefix::init_prefix()` returns `true` when the Rust call succeeds even if `wineboot` exits non-zero (`crates/moonshine-ffi/src/lib.rs`, `init_prefix`; `crates/moonshine-core/src/wine.rs`, `init_prefix`).
2. Detached Wine children are forgotten, and Swift tracks only a PID. The PID may refer to a wrapper process and is not ownership-checked before SIGTERM (`crates/moonshine-core/src/wine.rs`, `launch_program`; `Moonshine/ViewModels/AppViewModel.swift`, process tracking).
3. Prefix deletion/reinitialization has no running-process coordination (`RustPrefix::delete_prefix`/`reinit_prefix`; `AppViewModel.deletePrefix`).
4. `Runtime::download_wine` removes the existing installation before validating the replacement (`crates/moonshine-core/src/runtime.rs`).
5. Downloaded Wine, SteamSetup and winetricks content has no checksum/signature verification (`crates/moonshine-core/src/downloader.rs`, `installer.rs`).
6. Prefix loading accepts an arbitrary `id` string and joins it to the base directory; path validation was absent (`crates/moonshine-core/src/prefix.rs`).
7. `BottleConfig::save` wrote directly to `bottle.json`, so interruption could leave malformed JSON (`crates/moonshine-core/src/config.rs`).
8. Xcode links the Rust static library but has no build phase that regenerates/copies it; a clean build requires `scripts/build-rust.sh` first. The build also reports a duplicate `-lmoonshine_ffi` warning in this workspace.

## Confirmed Concurrency/UI Issues

- `AppViewModel` performs filesystem traversal, backend detection and subprocess-backed detection from main-actor methods during setup.
- Several long operations use `Task.detached`; global flags are not per-prefix and deletion/launch do not share a common operation coordinator.
- Wine installation redirects the process-wide stderr descriptor from a detached task.
- Swift bridge vectors and generated reference APIs are tightly coupled to generated `swift-bridge` internals.
- Save failures are discarded in Swift and most Rust FFI failures become `false`, `0`, empty vectors, or formatted strings.

## Runtime and Graphics Integration

- Wine backend detection and environment setup are in `crates/moonshine-core/src/wine.rs`.
- GPTK setup is best-effort and creates synthetic `syswow64` links when needed.
- D3DMetal/DXVK integration currently configures environment/overrides; `Libraries/d3dmetal/` and `Libraries/dxvk/` contain no payload files in this checkout.
- Quarantine removal and several symlink operations ignore their result.

## Missing Features

- Explicit process ownership/handles and coordinated shutdown before prefix mutation.
- Structured FFI operation results with error codes/messages rather than sentinel values.
- Atomic runtime replacement with post-extraction validation.
- Download integrity verification and update metadata.
- Swift unit/UI tests and Wine-process integration tests.
- CI workflow and reproducible clean-checkout native build.
- Steam integration is intentionally not expanded in TASK 002.

## Technical Risks (Unverified)

- Wine process PID reuse could terminate an unrelated process if a tracked PID exits and is reused.
- Captured process output and global stderr redirection may interleave under concurrent operations.
- Generated bridge files may drift from the Rust bridge declaration.
- Backend detection behavior varies substantially across host installations and macOS sandbox permissions.
- The current Xcode build may be relying on a checked-in prebuilt archive rather than rebuilding Rust.

## Baseline

- `cargo test --workspace`: passed, 67 tests total (58 core, 9 FFI).
- `cargo fmt --all -- --check`: failed because existing formatting differences are present in `installer.rs`, `wine.rs`, and related code; no formatting changes were applied for the audit.
- `xcodebuild -project Moonshine.xcodeproj -scheme Moonshine -configuration Debug -destination 'platform=macOS' build`: reached a successful native build in this workspace; emitted a duplicate `-lmoonshine_ffi` linker warning and an empty supported-platform diagnostic.
- No `.github` CI workflow was found.

## Prioritized Implementation Plan

1. Establish safe domain contracts: validate prefix identifiers and executable paths, add explicit process request/result models, propagate non-zero wineboot status, and make config writes atomic.
2. Replace FFI sentinel results with an explicit operation-result bridge while preserving a compatibility layer for the current Swift UI.
3. Move long-running orchestration behind an actor/service and remove process-wide stderr redirection.
4. Add process ownership and prefix operation coordination before allowing delete/reinitialize.
5. Make runtime replacement transactional and add checksum/signature verification for remote artifacts.
6. Add Swift tests, Wine integration tests with fixture runners, CI, and a clean Xcode/Rust build pipeline.
7. Only then expand Steam/game library integration.

## TASK 002 Changes Applied

- Added validated prefix-id and program-path handling in the core domain.
- Added `ProgramRequest` and `ProcessLaunchResult` models for process requests/results.
- Made `bottle.json` writes atomic.
- Made `wineboot` report failure through the existing FFI boolean instead of false success.
- Added focused unit tests for these contracts.

Remaining debt is listed above; Steam integration was not changed.

## TASK 003 Runtime Manager

- Supported runtime types are represented by `RuntimeType`: GPTK, WineHQ, CrossOver, Whisky, Wine Stable and Custom.
- `RuntimeManager::discover()` validates detected Wine installations, required `wine64`/`wineserver` binaries, architecture and macOS compatibility.
- Runtime installation uses a generated local archive name, optional SHA-256 verification, safe tar/zip extraction, staging, validation and rollback-preserving replacement.
- `runtime.json` stores runtime type, version, architecture, host macOS version, final binary paths, checksum and installation time.
- Existing per-prefix `WineBackendConfig`/`wine_path` selection remains the selection mechanism; `RuntimeManager::selected_path()` makes that contract explicit.
- `runtime_state_json()` exposes discovered runtime state to SwiftUI, while `install_wine_verified()` allows the UI/FFI caller to provide a published checksum.
- No third-party runtime is bundled or redistributed by these changes. The default GPTK URL remains an external download selected by the user/application.
- Tests cover valid/missing binaries, checksum mismatch, unsupported architecture, rejected archive paths, failed download input and interrupted replacement rollback.
