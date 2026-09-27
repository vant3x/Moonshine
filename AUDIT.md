# Moonshine Repository Audit

Date: 2026-09-19
Scope: TASK 001, including the current local state after the recent Rust, FFI and SwiftUI changes. No product code was changed during this audit.

## Executive Summary

Moonshine is a macOS SwiftUI application backed by two Rust crates. The current implementation is a functional prefix manager with Wine backend discovery, runtime download/validation, Steam installation/discovery, graphics diagnostics and supervised process logging. It is not yet a complete game launcher UX as described by TASK 008.

The repository does not demonstrate a real Windows game launch or rendering result. No Windows executable, Steam installation, game fixture, graphics benchmark, Swift test target or CI workflow is present in this checkout.

## Architecture

### SwiftUI entry point

- `Moonshine/MoonshineApp.swift:3-27` defines `MoonshineApp`, creates the `@StateObject` `AppViewModel`, installs `ContentView`, the New Prefix command and macOS Settings.
- `Moonshine/Views/ContentView.swift:4-39` provides the current NavigationSplitView shell: a prefix list on the left and a prefix detail or welcome screen on the right.
- `Moonshine/ViewModels/AppViewModel.swift:39-760` is the main-actor orchestration object. It owns prefix snapshots, runtime detection, installation status, initialization status and process polling.

### Rust crates and modules

Workspace members are declared in `Cargo.toml:1-3`:

- `crates/moonshine-core`: library crate; no binary target. Modules exported in `crates/moonshine-core/src/lib.rs:1-11`:
  - `config.rs`: persisted bottle configuration and validation.
  - `downloader.rs`: downloads, checksums and safe archive extraction.
  - `error.rs`: domain error enum.
  - `game.rs`: game profile association.
  - `graphics.rs`: D3DMetal/DXVK diagnostics and validation.
  - `installer.rs`: Steam, winetricks and local Steam manifest discovery.
  - `pe_parser.rs`: Windows executable parsing.
  - `process.rs`: detached process supervision and launch logs.
  - `prefix.rs`: prefix lifecycle, persistence, backups and executable scanning.
  - `runtime.rs`: runtime discovery, validation, download and replacement.
  - `wine.rs`: backend detection, environment construction and Wine execution.
- `crates/moonshine-ffi`: static library crate, configured in `crates/moonshine-ffi/Cargo.toml:8-10`; it exposes Rust types and functions to Swift.
- No Rust binary targets were found. `Package.swift` is absent; Swift is built by Xcode rather than Swift Package Manager.

### Swift/Rust FFI

- The bridge declaration is `crates/moonshine-ffi/src/lib.rs:30-136`.
- `RustPrefix` implements the prefix-facing API at `crates/moonshine-ffi/src/lib.rs:139-520`.
- Global functions for detection, runtime state, process state and termination are at `crates/moonshine-ffi/src/lib.rs:532-700`.
- `crates/moonshine-ffi/build.rs:1-19` generates bridge files into `Moonshine/Bridge/Generated/` during Cargo builds.
- The checked-in Swift header and wrapper are `Moonshine/Bridge/Generated/libmoonshine_ffi/libmoonshine_ffi.h` and `.swift`; they are generated artifacts and can drift if the bridge is changed without rebuilding.
- The contract is currently lossy for many operations: `Bool`, `0`, empty vectors and formatted `String` values are used instead of structured operation results. This is confirmed by the signatures at `crates/moonshine-ffi/src/lib.rs:89-107` and implementations such as `delete_prefix`, `launch_program` and `install_steam`.

## Feature Matrix

| Area | Current state | Assessment |
| --- | --- | --- |
| SwiftUI shell | Prefix list, welcome screen, detail screen, settings and New Prefix sheet | Implemented |
| Prefix persistence | `bottle.json`, validation, atomic save, reload, backup/restore | Implemented and Rust-tested |
| Runtime manager | Discovery and JSON state exist; verified download path exists through FFI | Core implemented; no dedicated Swift runtime screen |
| Wine backends | GPTK, WineHQ, CrossOver, Whisky, Wine Stable and custom-path detection | Implemented; host-dependent |
| Graphics | D3DMetal/DXVK prerequisites and diagnostics; payloads are absent from checkout | Partial; no rendering test |
| Steam | Download/local installer, installation log, launch, local manifest discovery | Implemented; not real-environment tested |
| Game library | Discovered Steam games are embedded as JSON in `PrefixData` | Partial; no library screen or durable game model in Swift |
| Process lifecycle | Supervisor-owned child and launch logs; PID polling and SIGTERM exposed | Partial; ownership and process groups missing |
| Diagnostics/logs | Per-launch log path and graphics message are surfaced in detail view | Partial; no diagnostics/logs screen or log browser |
| Tests | 80 core tests + 9 FFI tests pass; no Swift/UI or Wine integration tests | Rust baseline only |
| CI | No `.github/workflows` or equivalent found | Missing |

## Confirmed Bugs and Risks

### Confirmed issues

1. **`wineboot -u` status is ignored.** `WineRunner::init_prefix` runs the update at `crates/moonshine-core/src/wine.rs:853-884` but returns the first `wineboot` output, so the update can fail while initialization reports success. The FFI method correctly checks the returned status, but that status is the wrong command's status (`crates/moonshine-ffi/src/lib.rs:374-397`).
2. **Prefix mutation is not coordinated with running processes.** `RustPrefix::reinit_prefix` removes the prefix directory directly (`crates/moonshine-ffi/src/lib.rs:334-373`), and `AppViewModel.deletePrefix` ignores the boolean result (`Moonshine/ViewModels/AppViewModel.swift:363-379`). A live Wine process can still use the directory.
3. **PID termination is not ownership-safe.** `kill_process` sends `SIGTERM` to any non-zero PID (`crates/moonshine-ffi/src/lib.rs:674-689`). There is no process-group validation, owner token, or persisted process identity.
4. **Process records are never removed.** `crates/moonshine-core/src/process.rs:45-105` retains completed records in the global registry. Records also do not survive an app restart.
5. **Process-wide stderr redirection is concurrent and unsafe.** `AppViewModel.installWine` changes the global `STDERR_FILENO` from a detached task (`Moonshine/ViewModels/AppViewModel.swift:115-160`), which can capture or disrupt unrelated Rust logs.
6. **Main-actor setup performs blocking work.** `AppViewModel.setup` calls runtime detection, backend detection and prefix loading synchronously (`Moonshine/ViewModels/AppViewModel.swift:70-111`). These operations include filesystem traversal and subprocess-backed version detection.
7. **Custom Wine path is applied to every prefix.** `setCustomWinePath` iterates through every prefix (`Moonshine/ViewModels/AppViewModel.swift:542-558`) even though Wine selection is otherwise stored in each prefix's configuration.
8. **Xcode has no Rust build phase.** `Moonshine.xcodeproj/project.pbxproj:42-63` has only Sources, Frameworks and Resources phases. The static library is linked through `LIBRARY_SEARCH_PATHS` and `-lmoonshine_ffi` (`project.pbxproj:205-252`) while `Moonshine/Bridge/libmoonshine_ffi.a` is ignored by `.gitignore:22`. A clean checkout is therefore not a reproducible one-command Xcode build.
9. **Strict lint baseline is failing.** `cargo clippy --workspace --all-targets --all-features -- -D warnings` reports multiple errors, including derivable defaults, needless borrows, `map_or` simplifications, redundant closures and items after a test module. This is a confirmed quality-gate failure, not a runtime failure.

### Unverified risks

- Wine child trees may outlive the tracked supervisor or wrapper PID; this requires a real Wine installation to verify.
- PID reuse could make an old UI PID target an unrelated process.
- Backend behavior, sandbox/file access and graphics compatibility vary by installed runtime and game.
- D3DMetal and DXVK payload availability is not demonstrated: `Libraries/d3dmetal/` and `Libraries/dxvk/` contain no usable payload in this checkout.
- Generated bridge files may become stale after Rust API changes.

## TASK 008 UX Assessment

The current UI is a prefix administration surface, not yet the requested launcher workflow.

1. **Game Library:** Missing as a first-class screen. Steam games are loaded into `PrefixData.steamGames` as raw JSON (`Moonshine/ViewModels/AppViewModel.swift:6-33, 186-199`).
2. **Runtime Manager:** Runtime state is fetched into a string (`AppViewModel.swift:48, 92`) but there is no runtime management view for selection, installation progress, validation or removal.
3. **Prefix Manager:** Partially implemented through the split list, New Prefix sheet and PrefixDetailView.
4. **Add Game Wizard:** Missing. The current flow opens an arbitrary `.exe`/`.msi` panel and immediately launches it (`PrefixDetailView.swift:426-447`); it does not persist a game, validate a profile or guide the user.
5. **Game Settings:** Partially present as prefix settings, not per-game settings.
6. **Launch Progress:** Partially present as text and polling in `AppViewModel.monitorProcess` (`AppViewModel.swift:265-296`); no reusable progress model or dedicated screen.
7. **Diagnostics / Logs:** Partially present as one graphics message and launch-log path; no searchable log view or diagnostic report.

The most important TASK 008 architectural gap is state ownership. Swift stores snapshots and status strings separately from Rust, while views also keep local copies such as `steamInstalled` and configuration fields (`PrefixDetailView.swift:5-21`). This makes stale UI and conflicting state likely as more screens are added.

## Build and Test Baseline

Executed on 2026-09-19:

- `cargo test --workspace`: **passed, 89 tests** (80 `moonshine-core`, 9 `moonshine-ffi`).
- `cargo fmt --all -- --check`: **failed**; formatting differences remain in Rust sources, including `process.rs`, `wine.rs` and related modules.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: **failed** on existing lint violations.
- `xcodebuild -project Moonshine.xcodeproj -scheme Moonshine -configuration Debug -destination 'platform=macOS' build`: **passed** for the current machine and arm64 target. It emitted `ignoring duplicate libraries: '-lmoonshine_ffi'` and an empty supported-platform diagnostic.
- No Swift unit/UI tests, Wine integration tests, installed Steam client test or real Windows game launch test were available.

## Prioritized Plan

1. Fix the execution contract first: return the `wineboot -u` result, make prefix delete/reinitialize reject active processes, and replace unrestricted PID killing with owned process groups.
2. Introduce one structured FFI operation result containing status, error code/message, progress and log path. Preserve compatibility wrappers only while Swift migrates.
3. Move blocking detection and filesystem work behind an actor/service. Make per-prefix operation state explicit rather than global flags and status strings.
4. Make Xcode invoke `scripts/build-rust.sh` or otherwise build/link Rust reproducibly; remove duplicate linker configuration.
5. Build TASK 008 around durable Swift models: `GameLibrary`, `RuntimeState`, `PrefixState`, `LaunchOperation` and `DiagnosticReport`, each with loading/success/error states.
6. Implement the functional screens in this order: Runtime Manager, Prefix Manager cleanup, Add Game Wizard, Game Library, Launch Progress, Diagnostics/Logs, then per-game settings.
7. Add Swift tests for state transitions and navigation actions, UI tests for destructive confirmations and keyboard commands, and Wine integration fixtures before claiming launch support.
8. Add CI gates for tests, formatting, clippy and a clean bridge/Xcode build.

## Launch Claim

The repository contains code paths intended to launch Steam and arbitrary Windows executables, but this audit does **not** claim that Moonshine can launch a Windows game successfully. That behavior remains unverified on the available machine and with the current checkout.
