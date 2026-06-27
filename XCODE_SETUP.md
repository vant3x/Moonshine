# Moonshine — Xcode Setup Guide

## Prerequisitos

- Xcode 15+ instalado
- Rust toolchain (ya instalado: `rustc 1.96.0`)
- macOS 14+ (Sonoma o posterior)

## 1. Compilar Rust

```bash
cd /Users/alevante/vante-dev/vanteplay
./scripts/build-rust.sh
```

Esto genera `libmoonshine_ffi.a` en `target/aarch64-apple-darwin/release/`.

## 2. Crear Proyecto Xcode

1. Abrir Xcode
2. **File → New → Project**
3. Seleccionar **macOS → App**
4. Configurar:
   - Product Name: `Moonshine`
   - Organization Identifier: `com.vante`
   - Interface: **SwiftUI**
   - Language: **Swift**
   - Storage: **None**
5. Guardar en `/Users/alevante/vante-dev/vanteplay/` (carpeta raíz del proyecto)
6. **NO** crear Git repo (ya existe el workspace)

## 3. Agregar Archivos Swift

En el Finder, arrastrar estos archivos al proyecto Xcode (asegurar que estén en el target `Moonshine`):

```
Moonshine/
├── MoonshineApp.swift
├── Views/
│   ├── ContentView.swift
│   ├── WelcomeView.swift
│   ├── PrefixDetailView.swift
│   ├── NewPrefixView.swift
│   └── SettingsView.swift
└── ViewModels/
    └── AppViewModel.swift
```

> **Importante:** Seleccionar "Copy items if needed" y agregar al target `Moonshine`.

## 4. Linkar Static Library (Rust)

### 4a. Copiar el `.a` al proyecto

```bash
cp /Users/alevante/vante-dev/vanteplay/target/aarch64-apple-darwin/release/libmoonshine_ffi.a \
   /Users/alevante/vante-dev/vanteplay/Moonshine/Bridge/
```

### 4b. Agregar a Xcode

1. En Xcode, ir a **Project Navigator** → `Moonshine` (raíz)
2. Seleccionar target **Moonshine**
3. Ir a **Build Phases** → pestaña **Link Binary With Libraries**
4. Click **+** → **Add Other...** → **Add Files...**
5. Navegar a `Moonshine/Bridge/libmoonshine_ffi.a`
6. Click **Open**

### 4c. Configurar Library Search Paths

1. Seleccionar target **Moonshine**
2. Ir a **Build Settings**
3. Buscar **Library Search Paths**
4. Agregar: `$(PROJECT_DIR)/Moonshine/Bridge`

### 4d. Configurar Header Search Paths

1. En **Build Settings**, buscar **Header Search Paths**
2. Agregar: `$(PROJECT_DIR)/Moonshine/Bridge`

## 5. Generar Bridging Header

### 5a. Compilar con swift-bridge para generar el header

```bash
cd /Users/alevante/vante-dev/vanteplay

# Instalar swift-bridge CLI si no está
cargo install swift-bridge-build

# Generar el bridge
cargo run -p moonshine-ffi
```

> Si `cargo run` no genera los archivos, usar el build script manual:

```bash
cd crates/moonshine-ffi
cargo build --release
```

### 5b. Crear Bridging Header manualmente

Crear archivo `Moonshine/Bridge/Moonshine-Bridging-Header.h`:

```c
#ifndef Moonshine_Bridging_Header_h
#define Moonshine_Bridging_Header_h

#import "Generated/libmoonshine_ffi/libmoonshine_ffi.h"
#import "Generated/SwiftBridgeCore.h"

#endif
```

### 5c. Configurar en Xcode

1. Seleccionar target **Moonshine**
2. Ir a **Build Settings**
3. Buscar **Objective-C Bridging Header**
4. Establecer valor: `Moonshine/Bridge/Moonshine-Bridging-Header.h`

## 6. Configurar Build Settings

En **Build Settings** del target `Moonshine`:

| Setting | Valor |
|---------|-------|
| **Architecture** | `arm64` |
| **Valid Architectures** | `arm64` |
| **Build Active Architecture Only** | `YES` (Debug) / `NO` (Release) |
| **Swift Language Version** | `Swift 5` o `Swift 6` |
| **Mac Deployment Target** | `14.0` |

## 7. Clean Build Folder

```bash
# En Xcode: Shift+Cmd+K
# O desde terminal:
xcodebuild clean -project Moonshine.xcodeproj -scheme Moonshine
```

## 8. Build y Run

1. Seleccionar scheme **Moonshine** en la barra superior
2. Seleccionar destino: **My Mac (arm64)**
3. **Cmd+R** para build y run

## 9. Verificar que Funciona

Al ejecutar, la app debería:
- Mostrar "Welcome to Moonshine"
- Indicar si Wine está detectado (rojo si no está)
- Permitir crear prefixes
- Listar prefixes existentes

## Troubleshooting

### Error: "No such module" al importar tipos Rust

- Verificar que el bridging header está correctamente configurado
- Verificar que `libmoonshine_ffi.a` está en Link Binary With Libraries
- Verificar Library Search Paths incluye `$(PROJECT_DIR)/Moonshine/Bridge`

### Error: "Undefined symbols" al linkar

- Verificar que el `.a` fue compilado para `aarch64-apple-darwin`
- Verificar que el target es arm64 (no x86_64)

### Error: "Missing required module"

- Verificar que el módulo generado por swift-bridge está en Header Search Paths
- Verificar que el bridging header importa el `.h` correcto

### La app no detecta Wine

- Instalar Wine: `brew install wine`
- O instalar GPTK y ejecutar `./scripts/install-gptk.sh`

## Estructura Final del Proyecto

```
moonshine/
├── Moonshine.xcodeproj/          ← Xcode project
├── Moonshine/                    ← App source
│   ├── MoonshineApp.swift
│   ├── Views/
│   │   ├── ContentView.swift
│   │   ├── WelcomeView.swift
│   │   ├── PrefixDetailView.swift
│   │   ├── NewPrefixView.swift
│   │   └── SettingsView.swift
│   ├── ViewModels/
│   │   └── AppViewModel.swift
│   └── Bridge/
│       ├── libmoonshine_ffi.a    ← Static library from Rust
│       ├── Moonshine-Bridging-Header.h
│       └── Generated/            ← swift-bridge output
│           ├── libmoonshine_ffi/
│           ├── SwiftBridgeCore.h
│           └── SwiftBridgeCore.swift
├── crates/                       ← Rust source
│   ├── moonshine-core/
│   └── moonshine-ffi/
├── scripts/
├── Libraries/
└── Cargo.toml
```
