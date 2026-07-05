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

### 4. Fix 1: Bloquear wineboot antes de Steam/winetricks (CRÍTICO)

**Problema:** `wineboot -u` (inicialización del prefix) corría en background sin esperar. El usuario podía clickear "Install Steam" antes de que wineboot terminara, causando `ShellExecuteEx failed`.

**Solución:**
- Agregado `@Published var isInitializing` y `@Published var initStatus` en `AppViewModel`
- `createPrefix()` ahora muestra estado de inicialización y espera a que wineboot termine
- `installSteamInBackground()` y `runWinetricksInBackground()` verifican `isInitializing` antes de ejecutar
- `PrefixDetailView` muestra spinner de inicialización y deshabilita botones mientras wineboot corre

### 5. Fix 2: Wine path configurable (CrossOver/GPTK custom)

**Problema:** `WineRunner::detect()` solo buscaba rutas hardcodeadas. No soportaba CrossOver o Wine personalizado.

**Solución:**
- Agregado `wine_path: Option<String>` a `BottleConfig` en `config.rs`
- `WineRunner::detect()` ahora busca también en `/Applications/CrossOver.app/Contents/Frameworks/*/bin/wine64`
- Nuevo método `WineRunner::detect_for_config(config)` que usa `wine_path` si está configurado
- `install_steam()`, `run_winetricks()`, `run_program()`, `init_prefix()` usan `detect_for_config`
- FFI expone `get_wine_path()` y `set_wine_path()` a Swift
- SettingsView tiene nueva sección "Custom Wine Path" para configurar la ruta

### 6. Fix 3: Winetricks - verificar dependencias

**Problema:** `run_winetricks()` fallaba silenciosamente si `cabextract` no estaba instalado.

**Solución:**
- Nueva función `check_winetricks_deps()` que verifica que `cabextract` existe
- `run_winetricks()` llama `check_winetricks_deps()` al inicio
- Error claro: "cabextract not found. Install with: brew install cabextract"

### 7. Fix 4: SwiftUI threading

**Problema:** `Publishing changes from within view updates` — se modificaban `@Published` properties dentro de `onChange`.

**Solución:**
- `PrefixDetailView.onAppear` y `onChange` usan `DispatchQueue.main.async` para cargar estado

### 8. Fix 5: FFI error handling

**Problema:** `.expect("Could not get base dir")` causaba panic silencioso en la app.

**Solución:**
- `new_prefix()` usa `unwrap_or_default()` en vez de `expect()` para `get_base_dir()`
- `list_all_prefixes()` usa `unwrap_or_default()` en vez de `expect()`

### 9. Archivos creados/modificados

| Archivo | Acción |
|---------|--------|
| `crates/moonshine-core/src/wine.rs` | Modificado (3 bugs fix + CrossOver detection + detect_for_config) |
| `crates/moonshine-core/src/config.rs` | Modificado (wine_path field) |
| `crates/moonshine-core/src/installer.rs` | Modificado (check_winetricks_deps + detect_for_config) |
| `crates/moonshine-ffi/build.rs` | Creado |
| `crates/moonshine-ffi/Cargo.toml` | Modificado (build-dependencies) |
| `crates/moonshine-ffi/src/lib.rs` | Modificado (FFI error handling + wine_path + set_wine_path) |
| `Moonshine/Bridge/Moonshine-Bridging-Header.h` | Creado |
| `Moonshine/Bridge/Generated/**` | Generado automáticamente |
| `Moonshine/ViewModels/AppViewModel.swift` | Modificado (isInitializing + wine path + threading) |
| `Moonshine/Views/PrefixDetailView.swift` | Modificado (initialization UI + threading) |
| `Moonshine/Views/SettingsView.swift` | Modificado (Custom Wine Path section) |
| `scripts/build-rust.sh` | Modificado (nuevos nombres) |
| `scripts/install-gptk.sh` | Modificado (paths) |
| `README.md` | Modificado |
| `XCODE_SETUP.md` | Modificado |

### 10. Fix 6: Auto-fallback a backend WoW64 para Steam (CRÍTICO)

**Problema:** `install_steam()` intentaba ejecutar `SteamSetup.exe` (32-bit) con GPTK, que no soporta WoW64. El instalador siempre fallaba silenciosamente.

**Solución:**
- Nuevo método `WineRunner::find_backend_with_wo64()` — busca cualquier backend con soporte 32-bit (CrossOver > WineHQ > Whisky > WineStable)
- `install_steam()` ahora verifica `has_wo64()` ANTES de intentar ejecutar
- Si el backend actual es GPTK, auto-cambia al mejor backend WoW64 disponible
- Si no hay ningún backend WoW64, retorna error claro con instrucciones: `brew install --cask wine-stable`
- Se agregó `WINE` env var y wine bin al PATH en `build_env()` para compatibilidad con winetricks

### 11. Fix 7: Winetricks — PATH, WINEARCH y errores accionables

**Problema:** winetricks fallaba porque:
- `wineserver` no estaba en PATH (wine bin directory no se agregaba)
- `WINEARCH` no se forzaba a `wow64` para verbos 32-bit (vcrun, dotnet, etc.)
- Los errores no decían qué instalar

**Solución:**
- `run_winetricks()` ahora agrega wine bin directory a PATH
- Si el backend tiene WoW64, fuerza `WINEARCH=wow64` para verbos 32-bit
- Errores ahora incluyen instrucciones: `brew install cabextract`, `brew install --cask wine-stable`
- Logging detallado de variables de entorno para debugging

### 12. Fix 8: Matar wineserver después de wineboot fallido

**Problema:** Si `wineboot` fallaba (ej: bug msvcrt de WineHQ 11.0), el `wineserver` quedaba como proceso zombie, bloqueando comandos wine posteriores.

**Solución:**
- Después de un wineboot fallido, se ejecuta `wineserver -k -w` para matar procesos stale
- Esto aplica a TODOS los backends, no solo WineHQ

### 13. Fix 9: Detectar bug msvcrt de Wine 11.0 (CRÍTICO)

**Problema:** WineHQ 11.0 en macOS ARM64 tiene un bug crítico en `msvcrt.dll`:
- `_invalid_parameter` → `Unhandled exception 0xc0000417` → `could not load kernel32.dll`
- TODOS los comandos wine crashean: wineboot, winetricks, SteamSetup.exe
- El prefix se crea con GPTK pero wineboot de WineHQ falla siempre

**Solución:**
- `WineRunner::has_known_msvcrt_bug()` — detecta Wine 11.0 por versión
- `init_prefix()` — detecta bug ANTES de intentar wineboot, salta directo a GPTK fallback
- `install_steam()` — retorna error claro con instrucciones si Wine 11.0 detectado
- `run_winetricks()` — retorna error claro con instrucciones si Wine 11.0 detectado
- `detect_best()` — log warning si Wine 11.0 está instalado
- FFI: `has_wine_msvcrt_bug()` expuesto a Swift
- Swift: banner de warning en SettingsView + PrefixDetailView deshabilita botones

**Error que ve el usuario:**
```
Wine 11.0 has a critical bug (msvcrt.dll crash) on macOS.
All wine commands fail with: Unhandled exception 0xc0000417

Solutions:
1. brew uninstall --cask wine-stable && brew install --cask wine-stable
2. Install CrossOver: https://www.codeweavers.com
```

### 14. Pendiente: Xcode Project

Crear `Moonshine.xcodeproj` en Xcode (manual):
- File → New → Project → macOS → App
- Product Name: `Moonshine`
- Guardar en la raíz del proyecto
- Agregar archivos Swift, linkear `libmoonshine_ffi.a`, configurar bridging header
