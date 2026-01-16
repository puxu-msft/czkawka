#!/bin/bash
set -euo pipefail

# Cross-compile Czkawka GTK for Windows
# This script is designed to run inside the GTK4 cross Docker container

source "$HOME/.cargo/env"

SRC_DIR="/src"
OUTPUT_DIR="/output"
BUILD_TYPE="${BUILD_TYPE:-release}"

echo "=== Czkawka GTK Windows Cross-Compilation ==="
echo "Build type: $BUILD_TYPE"
echo "Source: $SRC_DIR"
echo "Output: $OUTPUT_DIR"

cd "$SRC_DIR"

# Create output directory structure
mkdir -p "$OUTPUT_DIR/package"

# Set PKG_CONFIG_PATH for cross-compilation
export PKG_CONFIG_PATH=/usr/lib64/pkgconfig:/usr/share/pkgconfig:$MINGW_PREFIX/lib/pkgconfig/:/usr/x86_64-w64-mingw32/lib/pkgconfig/
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/src/target}"

if [ "$BUILD_TYPE" = "release" ]; then
    echo ""
    echo "=== Building czkawka_gui and czkawka_cli (Release) ==="
    cargo build --target=x86_64-pc-windows-gnu --release --locked
    
    cp "$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/release/czkawka_gui.exe" "$OUTPUT_DIR/package/"
    cp "$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/release/czkawka_cli.exe" "$OUTPUT_DIR/package/"
else
    echo ""
    echo "=== Building czkawka_gui and czkawka_cli (Debug/FastCI) ==="
    cargo build --target=x86_64-pc-windows-gnu --locked --profile fastci
    
    cp "$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/fastci/czkawka_gui.exe" "$OUTPUT_DIR/package/"
    cp "$CARGO_TARGET_DIR/x86_64-pc-windows-gnu/fastci/czkawka_cli.exe" "$OUTPUT_DIR/package/"
fi

echo ""
echo "=== Packaging DLLs and dependencies ==="

# Copy required DLLs
cp -t "$OUTPUT_DIR/package" $(pds -vv -f "$OUTPUT_DIR/package"/*.exe) || true

# Add gdbus which is recommended on Windows
cp "$MINGW_PREFIX/bin/gdbus.exe" "$OUTPUT_DIR/package/" || true

# Handle the glib schema compilation
glib-compile-schemas "$MINGW_PREFIX/share/glib-2.0/schemas/"
mkdir -p "$OUTPUT_DIR/package/share/glib-2.0/schemas/"
cp -T "$MINGW_PREFIX/share/glib-2.0/schemas/gschemas.compiled" "$OUTPUT_DIR/package/share/glib-2.0/schemas/gschemas.compiled"

# Pixbuf stuff for SVG icons
mkdir -p "$OUTPUT_DIR/package/lib/gdk-pixbuf-2.0"
cp -rT "$MINGW_PREFIX/lib/gdk-pixbuf-2.0" "$OUTPUT_DIR/package/lib/gdk-pixbuf-2.0"
cp -f -t "$OUTPUT_DIR/package" $(pds -vv -f "$MINGW_PREFIX/lib/gdk-pixbuf-2.0/2.10.0/loaders/"*) || true

# Strip binaries to reduce size
find "$OUTPUT_DIR/package" -iname "*.dll" -or -iname "*.exe" -type f -exec mingw-strip {} + || true

# Download and add GTK theme
cd "$OUTPUT_DIR/package"
mkdir -p share
cd share
wget2 -q https://github.com/qarmin/czkawka/files/10832192/gtk4_theme.zip || true
if [ -f gtk4_theme.zip ]; then
    unzip -q gtk4_theme.zip
    rm gtk4_theme.zip
fi
cd ..

# Download OpenGL libraries
wget2 -q https://github.com/qarmin/Automated-Fuzzer/releases/download/test/libGL.zip || true
if [ -f libGL.zip ]; then
    unzip -q libGL.zip
    rm libGL.zip
fi

cd "$OUTPUT_DIR"

echo ""
echo "=== Build Complete ==="
echo "Output files:"
ls -lh "$OUTPUT_DIR/package"/*.exe
echo ""
echo "Total package size:"
du -sh "$OUTPUT_DIR/package"
