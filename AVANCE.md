# Avance del Proyecto Moonshine

## Resumen de cambios realizados

### 1. Renombre completo: VantePlay → Moonshine

Se renombró **todo** el proyecto:

| Antes | Después |
|-------|---------|
| `VantePlay/` (directorio Swift) | `Moonshine/` |
| `VantePlayApp.swift` | `MoonshineApp.swift` |
| `VantePlay-Bridging-Header.h` | `Moonshine-Bridging-Header.h` |
| `crates/vanteplay-core/` | `crates/moonshine-core/` |
| `crates/vanteplay-ffi/` | `crates/moonshine-ffi/` |
| `VantePlayError` (enum Rust) | `MoonshineError` |
| `libvanteplay_ffi.a` | `libmoonshine_ffi.a` |
| `vanteplay_core` (crate) | `moonshine_core` |
| `vanteplay-ffi` (crate) | `moonshine-ffi` |

### 2. Bugs corregidos en Rust (`crates/moonshine-core/src/wine.rs`)

#### Bug 1: WINEDLLOVERRIDES se sobrescribía en cada iteración
- **Problema:** `HashMap::insert` reemplazaba el valor, solo el último override se conservaba
- **Fix:** Se acumulan todos los overrides con `";"` antes de insertar

#### Bug 2: `init_prefix` usaba comando incorrecto
- **Problema:** Usaba `wine boot -u` en lugar de `wine wineboot -u`
- **Fix:** `boot` → `wineboot`

#### Bug 3: Registry key path incorrecto en `set_windows_version`
- **Problema:** `HKEY_CURRENT_USER\\Software\\Wine\\Version` incluía "Version" en el path de la key
- **Fix:** Se cambió a `HKEY_CURRENT_USER\\Software\\Wine` (key correcta)

### 3. Generación del Bridge Swift-Rust

- Se creó `crates/moonshine-ffi/build.rs` con `swift_bridge_build::parse_bridges`
- Se agregó `swift-bridge-build` como build-dependency
- Ahora `cargo build` genera los archivos bridge en `Moonshine/Bridge/Generated/`

### 4. Archivos creados/modificados

| Archivo | Acción |
|---------|--------|
| `crates/moonshine-core/src/wine.rs` | Modificado (3 bugs fix) |
| `crates/moonshine-ffi/build.rs` | Creado |
| `crates/moonshine-ffi/Cargo.toml` | Modificado (build-dependencies) |
| `Moonshine/Bridge/Moonshine-Bridging-Header.h` | Creado |
| `Moonshine/Bridge/Generated/**` | Generado automáticamente |
| `scripts/build-rust.sh` | Modificado (nuevos nombres) |
| `scripts/install-gptk.sh` | Modificado (paths) |
| `README.md` | Modificado |
| `XCODE_SETUP.md` | Modificado |

### 5. Pendiente: Xcode Project

Crear `Moonshine.xcodeproj` en Xcode (manual):
- File → New → Project → macOS → App
- Product Name: `Moonshine`
- Guardar en la raíz del proyecto
- Agregar archivos Swift, linkear `libmoonshine_ffi.a`, configurar bridging header
