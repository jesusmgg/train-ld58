#!/bin/bash
set -e

# Check if target is provided
if [ -z "$1" ]; then
    echo "Usage: ./release.sh <target>"
    echo "Targets: wasm, windows, linux-musl, linux-gnu"
    echo "Example: ./release.sh wasm"
    exit 1
fi

TARGET=$1

# Store project root
PROJECT_ROOT="$(pwd)"

echo "Building release for target: $TARGET..."

# Build based on target
case "$TARGET" in
    wasm)
        cargo build --release --target wasm32-unknown-unknown
        RELEASE_DIR="target/wasm32-unknown-unknown/release"
        ;;
    windows)
        cargo build --release --target x86_64-pc-windows-gnu
        RELEASE_DIR="target/x86_64-pc-windows-gnu/release"
        ;;
    linux-musl)
        cargo build --release --target x86_64-unknown-linux-musl
        RELEASE_DIR="target/x86_64-unknown-linux-musl/release"
        ;;
    linux-gnu)
        cargo build --release --target x86_64-unknown-linux-gnu
        RELEASE_DIR="target/x86_64-unknown-linux-gnu/release"
        ;;
    *)
        echo "Unknown target: $TARGET"
        echo "Valid targets: wasm, windows, linux-musl, linux-gnu"
        exit 1
        ;;
esac

# Extract version from Cargo.toml
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "Version: $VERSION"

# Paths
BUILD_DIR="build/$TARGET"
ZIP_NAME="clean-line-${VERSION}.zip"

# Create build directory if it doesn't exist
mkdir -p "$BUILD_DIR"

# Create temporary directory for zipping
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "Preparing files for packaging..."

# Copy files based on target
case "$TARGET" in
    wasm)
        # For wasm, put files directly in temp dir (zipbomb for hosting sites)
        cp "$RELEASE_DIR/clean_line.wasm" "$TEMP_DIR/"
        cp "$RELEASE_DIR/index.html" "$TEMP_DIR/"
        cp "$RELEASE_DIR/mq_js_bundle.js" "$TEMP_DIR/"
        cp -r "$RELEASE_DIR/assets" "$TEMP_DIR/"
        ;;
    windows|linux-musl|linux-gnu)
        # For native builds, create a directory inside zip
        PACKAGE_DIR="$TEMP_DIR/clean-line-${VERSION}"
        mkdir -p "$PACKAGE_DIR"

        if [ "$TARGET" = "windows" ]; then
            cp "$RELEASE_DIR/clean_line.exe" "$PACKAGE_DIR/"
        else
            cp "$RELEASE_DIR/clean_line" "$PACKAGE_DIR/"
        fi
        cp -r assets "$PACKAGE_DIR/"
        ;;
esac

# Create zip file
echo "Creating $ZIP_NAME..."
cd "$TEMP_DIR"
if [ "$TARGET" = "wasm" ]; then
    zip -r "$ZIP_NAME" ./*
else
    zip -r "$ZIP_NAME" "clean-line-${VERSION}"
fi
mv "$ZIP_NAME" "$PROJECT_ROOT/$BUILD_DIR/"

echo "Release package created at $BUILD_DIR/$ZIP_NAME"
