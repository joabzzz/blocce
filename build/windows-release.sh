#!/bin/bash
set -e

echo "=== Windows x64 Build ==="

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Extract version from Cargo.toml
VERSION=$(grep -E '^version = ' "$REPO_ROOT/Cargo.toml" | head -1 | sed -E 's/version = "(.*)"/\1/')
echo "Building version: $VERSION"

APP_NAME="Blocce"
DIST_DIR="dist"
PACKAGE_DIR="${DIST_DIR}/Blocce-${VERSION}-x64"
ZIP_PATH="${DIST_DIR}/Blocce-${VERSION}-x64.zip"

# Detect platform
if [[ "$(uname)" == "MINGW"* ]] || [[ "$(uname)" == "MSYS"* ]] || [[ "$OS" == "Windows_NT" ]]; then
    IS_WINDOWS=true
else
    IS_WINDOWS=false
fi

# Install Rust if not present (cross-platform)
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    if $IS_WINDOWS; then
        # Windows
        curl --proto '=https' --tlsv1.2 -sSf https://win.rustup.rs/x86_64 -o rustup-init.exe
        ./rustup-init.exe -y
        source "$HOME/.cargo/env"
    else
        # Unix-like
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
fi

# Detect target based on environment
if [[ -n "$DOCKER_BUILD" ]] || ! command -v cl.exe &> /dev/null; then
    # Docker or no MSVC - use GNU target
    WINDOWS_TARGET="x86_64-pc-windows-gnu"
    echo "Using GNU target for cross-compilation..."
else
    # Native Windows with MSVC
    WINDOWS_TARGET="x86_64-pc-windows-msvc"
    echo "Using MSVC target for native build..."
fi

# Add Windows target
echo "Adding Windows x64 target ($WINDOWS_TARGET)..."
rustup target add $WINDOWS_TARGET

# Build for Windows x64
echo "Building for $WINDOWS_TARGET..."
cargo build --release --target $WINDOWS_TARGET

# Create distribution directory
echo "Creating distribution package..."
mkdir -p "$PACKAGE_DIR"

# Copy binary (detect which target was used)
if [[ -f "target/x86_64-pc-windows-msvc/release/blocce.exe" ]]; then
    cp target/x86_64-pc-windows-msvc/release/blocce.exe "$PACKAGE_DIR/"
elif [[ -f "target/x86_64-pc-windows-gnu/release/blocce.exe" ]]; then
    cp target/x86_64-pc-windows-gnu/release/blocce.exe "$PACKAGE_DIR/"
else
    echo "❌ Could not find Windows binary"
    exit 1
fi

# Copy assets if they exist
if [[ -d "assets" ]]; then
    echo "Copying assets..."
    cp -r assets "$PACKAGE_DIR/"
fi

# Create a simple README
cat > "$PACKAGE_DIR/README.txt" << EOF
Blocce - Windows x64 Build

To run the game, double-click blocce.exe

System Requirements:
- Windows 10 or later
- 64-bit processor

For more information, visit: https://github.com/yourusername/blocce
EOF

# Create ZIP archive
echo "Creating ZIP archive..."
cd "$DIST_DIR"
PACKAGE_NAME="Blocce-${VERSION}-x64"
if command -v 7z &> /dev/null; then
    # Use 7-Zip if available (Windows)
    7z a -tzip "${PACKAGE_NAME}.zip" "${PACKAGE_NAME}/"
elif command -v zip &> /dev/null; then
    # Use zip if available (Unix-like)
    zip -r "${PACKAGE_NAME}.zip" "${PACKAGE_NAME}/"
else
    # Fallback to PowerShell (Windows)
    if $IS_WINDOWS; then
        powershell -Command "Compress-Archive -Path '${PACKAGE_NAME}' -DestinationPath '${PACKAGE_NAME}.zip' -Force"
    else
        echo "❌ No zip utility found. Please install zip or 7z."
        exit 1
    fi
fi
cd "$REPO_ROOT"

echo "✅ Windows build and packaging completed successfully!"
echo "Artifacts created:"
ls -lh "$ZIP_PATH" 2>/dev/null || ls -l "$ZIP_PATH"
