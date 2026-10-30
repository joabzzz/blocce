#!/bin/bash
set -e

echo "=== macOS Build (No Code Signing) ==="
echo "Platform: $(uname -m)"
echo "OS: $(sw_vers -productName) $(sw_vers -productVersion)"

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Extract version from Cargo.toml
VERSION=$(grep -E '^version = ' "$REPO_ROOT/Cargo.toml" | head -1 | sed -E 's/version = "(.*)"/\1/')
echo "Building version: $VERSION"

# Check if we're actually on macOS
if [[ "$(uname)" != "Darwin" ]]; then
    echo "❌ This script must run on macOS"
    exit 1
fi

# Install Rust if not present
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Add macOS targets
echo "Adding macOS targets..."
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build for both architectures
echo "Building for x86_64-apple-darwin..."
cargo build --release --target x86_64-apple-darwin

echo "Building for aarch64-apple-darwin..."
cargo build --release --target aarch64-apple-darwin

# Create universal binary
echo "Creating universal binary..."
mkdir -p target/universal/release
lipo -create \
    target/x86_64-apple-darwin/release/blocce \
    target/aarch64-apple-darwin/release/blocce \
    -output target/universal/release/blocce

# App bundle setup
APP_NAME="Blocce"
BINARY_PATH="target/universal/release/blocce"
APP_BUNDLE="target/universal/release/${APP_NAME}.app"
DMG_PATH="dist/${APP_NAME}-${VERSION}.dmg"

# Create .app bundle structure
echo "Creating .app bundle..."
mkdir -p "${APP_BUNDLE}/Contents/MacOS"
mkdir -p "${APP_BUNDLE}/Contents/Resources"

# Copy binary
cp "$BINARY_PATH" "${APP_BUNDLE}/Contents/MacOS/${APP_NAME}"

# Copy project assets directly into the bundle
if [[ -d "assets" ]]; then
    echo "Copying project assets into bundle..."
    ditto "assets" "${APP_BUNDLE}/Contents/Resources/assets"
fi

# Copy icon if it exists
if [[ -f "assets/icon.png" ]]; then
    echo "Converting icon to icns..."
    mkdir -p /tmp/Blocce.iconset
    sips -z 16 16     assets/icon.png --out /tmp/Blocce.iconset/icon_16x16.png 2>/dev/null || cp assets/icon.png /tmp/Blocce.iconset/icon_16x16.png
    sips -z 32 32     assets/icon.png --out /tmp/Blocce.iconset/icon_16x16@2x.png 2>/dev/null || cp assets/icon.png /tmp/Blocce.iconset/icon_16x16@2x.png
    sips -z 32 32     assets/icon.png --out /tmp/Blocce.iconset/icon_32x32.png 2>/dev/null || cp assets/icon.png /tmp/Blocce.iconset/icon_32x32.png
    sips -z 64 64     assets/icon.png --out /tmp/Blocce.iconset/icon_32x32@2x.png 2>/dev/null || cp assets/icon.png /tmp/Blocce.iconset/icon_32x32@2x.png
    sips -z 128 128   assets/icon.png --out /tmp/Blocce.iconset/icon_128x128.png 2>/dev/null || cp assets/icon.png /tmp/Blocce.iconset/icon_128x128.png
    sips -z 256 256   assets/icon.png --out /tmp/Blocce.iconset/icon_128x128@2x.png 2>/dev/null || cp assets/icon.png /tmp/Blocce.iconset/icon_128x128@2x.png
    sips -z 256 256   assets/icon.png --out /tmp/Blocce.iconset/icon_256x256.png 2>/dev/null || cp assets/icon.png /tmp/Blocce.iconset/icon_256x256.png
    sips -z 512 512   assets/icon.png --out /tmp/Blocce.iconset/icon_256x256@2x.png 2>/dev/null || cp assets/icon.png /tmp/Blocce.iconset/icon_256x256@2x.png
    sips -z 512 512   assets/icon.png --out /tmp/Blocce.iconset/icon_512x512.png 2>/dev/null || cp assets/icon.png /tmp/Blocce.iconset/icon_512x512.png
    sips -z 1024 1024 assets/icon.png --out /tmp/Blocce.iconset/icon_512x512@2x.png 2>/dev/null || cp assets/icon.png /tmp/Blocce.iconset/icon_512x512@2x.png
    iconutil -c icns /tmp/Blocce.iconset -o "${APP_BUNDLE}/Contents/Resources/Blocce.icns" 2>/dev/null || echo "Warning: Could not create icns file"
    rm -rf /tmp/Blocce.iconset
fi

# Create Info.plist
cat > "${APP_BUNDLE}/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.blocce.game</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>Blocce.icns</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "⚠️  Skipping code signing (not configured)"

# Create DMG
echo "Creating DMG..."
mkdir -p dist

# Remove old DMG if exists
rm -f "$DMG_PATH"

# Ensure no stale volume is mounted
if mount | awk '{print $3}' | grep -qx "/Volumes/${APP_NAME}"; then
    echo "Detaching previous /Volumes/${APP_NAME} mount..."
    hdiutil detach "/Volumes/${APP_NAME}" || hdiutil detach -force "/Volumes/${APP_NAME}" || true
fi

# Create DMG with app bundle using a temporary mount point
DMG_TMP_DIR=$(mktemp -d /tmp/blocce-dmg.XXXXXX)
MOUNT_POINT="$DMG_TMP_DIR/mount"
SPARSE_IMAGE="$DMG_TMP_DIR/${APP_NAME}.sparseimage"

cleanup_dmg() {
    if mount | awk '{print $3}' | grep -qx "$MOUNT_POINT"; then
        hdiutil detach "$MOUNT_POINT" -quiet || hdiutil detach "$MOUNT_POINT" -force -quiet || true
    fi
    rm -rf "$DMG_TMP_DIR"
}
trap cleanup_dmg EXIT

mkdir -p "$MOUNT_POINT"

# Determine minimum DMG size (add buffer for filesystem metadata)
APP_SIZE_KB=$(du -sk "${APP_BUNDLE}" | awk '{print $1}')
DMG_SIZE_MB=$(( (APP_SIZE_KB / 1024) + 20 ))
if [[ $DMG_SIZE_MB -lt 100 ]]; then
    DMG_SIZE_MB=100
fi

echo "Creating sparse image (${DMG_SIZE_MB}MB)..."
hdiutil create -size "${DMG_SIZE_MB}m" -fs HFS+ -volname "$APP_NAME" "$SPARSE_IMAGE"

echo "Attaching sparse image..."
hdiutil attach "$SPARSE_IMAGE" -mountpoint "$MOUNT_POINT" -nobrowse -quiet

echo "Copying app bundle into image..."
cp -R "${APP_BUNDLE}" "$MOUNT_POINT/"
sync

echo "Detaching sparse image..."
hdiutil detach "$MOUNT_POINT" -quiet || hdiutil detach "$MOUNT_POINT" -force -quiet || true

echo "Converting sparse image to compressed DMG..."
CONVERTED_IMAGE="$DMG_TMP_DIR/${APP_NAME}"
hdiutil convert "$SPARSE_IMAGE" -format UDZO -o "$CONVERTED_IMAGE" -quiet
mv "${CONVERTED_IMAGE}.dmg" "$DMG_PATH"

trap - EXIT
rm -rf "$DMG_TMP_DIR"

echo "✅ macOS build and packaging completed successfully!"
echo "Artifacts created:"
ls -lh "$DMG_PATH"
echo ""
echo "To test locally: open \"$DMG_PATH\""
