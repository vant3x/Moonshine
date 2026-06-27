#!/bin/bash
set -e

echo "=== Moonshine GPTK Setup ==="
echo "This script installs Apple's Game Porting Toolkit (ARM64 Wine)"
echo ""

INSTALL_DIR="$HOME/Library/Application Support/Moonshine/Libraries"
WINE_DIR="${INSTALL_DIR}/Wine"

echo "Options:"
echo ""
echo "1. Download prebuilt GPTK (recommended):"
echo "   URL: https://github.com/Gcenx/game-porting-toolkit/releases/tag/Game-Porting-Toolkit-3.0-3"
echo "   Archive: game-porting-toolkit-3.0-3.tar.xz"
echo "   Extract to: ${WINE_DIR}"
echo "   Expected: ${WINE_DIR}/bin/wine64"
echo ""
echo "2. Homebrew (Gcenx tap, still x86_64 only):"
echo "   brew tap gcenx/homebrew-apple"
echo "   brew install gcenx/apple/game-porting-toolkit"
echo ""
echo "3. Apple's official DMG (Apple Developer account required):"
echo "   https://developer.apple.com/games/game-porting-toolkit/"
echo ""

if [ -f "$WINE_DIR/bin/wine64" ]; then
    echo "Wine found at: ${WINE_DIR}/bin/wine64"
    file "$WINE_DIR/bin/wine64"
else
    echo "Wine not found at: ${WINE_DIR}/bin/wine64"
    echo "Launch Moonshine and use the 'Download Wine (ARM64)' button."
fi

echo ""
echo "Done!"
