# Docker Cross-Compilation for Windows

This directory contains Docker configurations for cross-compiling Czkawka from Linux to Windows.

## Available Images

### 1. Krokiet + CLI (Lightweight)

**File:** `Dockerfile.cross-windows`

Compiles `czkawka_cli` and `krokiet` (Slint GUI). These are standalone executables with no external DLL dependencies.

```bash
# Build the Docker image
docker build -t czkawka-cross-windows -f misc/docker/Dockerfile.cross-windows .

# Compile (output goes to ./build-output)
docker run --rm \
    -v $(pwd):/src \
    -v $(pwd)/build-output:/output \
    czkawka-cross-windows

# For debug build
docker run --rm \
    -v $(pwd):/src \
    -v $(pwd)/build-output:/output \
    -e BUILD_TYPE=debug \
    czkawka-cross-windows
```

**Output:**
- `build-output/czkawka_cli.exe` (~31 MB)
- `build-output/krokiet.exe` (~49 MB)

### 2. GTK GUI (Full Package)

**File:** `Dockerfile.cross-windows-gtk`

Compiles `czkawka_gui` (GTK 4) along with all required DLLs, themes, and resources.

```bash
# Build the Docker image
docker build -t czkawka-cross-windows-gtk -f misc/docker/Dockerfile.cross-windows-gtk .

# Compile (output goes to ./build-output/package)
docker run --rm \
    -v $(pwd):/src \
    -v $(pwd)/build-output:/output \
    czkawka-cross-windows-gtk

# For debug build (faster, larger binary)
docker run --rm \
    -v $(pwd):/src \
    -v $(pwd)/build-output:/output \
    -e BUILD_TYPE=debug \
    czkawka-cross-windows-gtk
```

**Output:** `build-output/package/` directory containing:
- `czkawka_gui.exe`
- `czkawka_cli.exe`
- Required DLLs (`*.dll`)
- GTK schemas and themes
- Icon loaders

## Caching Build Artifacts

To speed up subsequent builds, mount the target directory:

```bash
# Create a persistent volume for cargo cache
docker volume create czkawka-cargo-cache

# Use the cache
docker run --rm \
    -v $(pwd):/src \
    -v $(pwd)/build-output:/output \
    -v czkawka-cargo-cache:/src/target \
    czkawka-cross-windows
```

Or mount a local directory:

```bash
docker run --rm \
    -v $(pwd):/src \
    -v $(pwd)/build-output:/output \
    -v $(pwd)/target-docker:/src/target \
    czkawka-cross-windows
```

## Quick One-Liner

```bash
# Build everything at once (CLI + Krokiet)
docker build -t czkawka-cross-windows -f misc/docker/Dockerfile.cross-windows . && \
docker run --rm -v $(pwd):/src -v $(pwd)/build-output:/output czkawka-cross-windows

# Or for GTK version
docker build -t czkawka-cross-windows-gtk -f misc/docker/Dockerfile.cross-windows-gtk . && \
docker run --rm -v $(pwd):/src -v $(pwd)/build-output:/output czkawka-cross-windows-gtk
```

## Notes

- The GTK image is based on `ghcr.io/mglolenstine/gtk4-cross:gtk-4.12` which provides pre-built GTK 4 libraries for Windows cross-compilation
- Krokiet (Slint) produces a single self-contained executable, making it ideal for distribution
- The GTK version requires bundling many DLLs (~100+ MB total package size)
- For video similarity feature, users need to install ffmpeg separately on Windows
