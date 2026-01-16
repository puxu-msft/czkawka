#!/bin/bash
set -euo pipefail

# Cross-compile Czkawka for Windows
# This script is designed to run inside the Docker container

SRC_DIR="/src"
OUTPUT_DIR="/output"
BUILD_TYPE="${BUILD_TYPE:-release}"  # release or debug

echo "=== Czkawka Windows Cross-Compilation ==="
echo "Build type: $BUILD_TYPE"
echo "Source: $SRC_DIR"
echo "Output: $OUTPUT_DIR"

cd "$SRC_DIR"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Set cargo target directory to a location that can be cached
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/src/target}"

if [ "$BUILD_TYPE" = "release" ]; then
    echo ""
    echo "=== Building czkawka_cli (Release) ==="
    cargo build --release --target x86_64-pc-windows-gnu --bin czkawka_cli
    cp "$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/release/czkawka_cli.exe" "$OUTPUT_DIR/"
    
    echo ""
    echo "=== Building krokiet (Release) ==="
    cargo build --release --target x86_64-pc-windows-gnu --bin krokiet
    cp "$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/release/krokiet.exe" "$OUTPUT_DIR/"
else
    echo ""
    echo "=== Building czkawka_cli (Debug) ==="
    cargo build --target x86_64-pc-windows-gnu --bin czkawka_cli
    cp "$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/debug/czkawka_cli.exe" "$OUTPUT_DIR/"
    
    echo ""
    echo "=== Building krokiet (Debug) ==="
    cargo build --target x86_64-pc-windows-gnu --bin krokiet
    cp "$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/debug/krokiet.exe" "$OUTPUT_DIR/"
fi

echo ""
echo "=== Build Complete ==="
echo "Output files:"
ls -lh "$OUTPUT_DIR"/*.exe
