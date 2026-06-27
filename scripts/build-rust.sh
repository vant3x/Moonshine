#!/bin/bash
set -e

echo "=== Building Moonshine Rust crates ==="

cd "$(dirname "$0")/.."

TARGET="aarch64-apple-darwin"
BUILD_DIR="target/${TARGET}/release"

echo "Building moonshine-ffi for ${TARGET}..."
cargo build --release --target ${TARGET} -p moonshine-ffi

STATIC_LIB="${BUILD_DIR}/libmoonshine_ffi.a"
OUTPUT_DIR="Moonshine/Bridge"

if [ -f "$STATIC_LIB" ]; then
    echo "Copying static library to ${OUTPUT_DIR}/"
    cp "$STATIC_LIB" "${OUTPUT_DIR}/libmoonshine_ffi.a"
    echo "Done! Static library: ${OUTPUT_DIR}/libmoonshine_ffi.a"
else
    echo "ERROR: Static library not found at ${STATIC_LIB}"
    exit 1
fi
