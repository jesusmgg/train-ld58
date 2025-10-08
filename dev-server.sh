#!/bin/sh

# Default to debug build
BUILD_TYPE="debug"

# Check for --release flag
if [ "$1" = "--release" ]; then
    BUILD_TYPE="release"
fi

echo "Starting live-server with $BUILD_TYPE build..."

live-server \
--port=42069 \
--cors \
--verbose \
--no-browser \
--ignore=deps,incremental,examples,build \
--watch=clean_line.wasm \
target/wasm32-unknown-unknown/$BUILD_TYPE
