#!/bin/sh

live-server \
--port=42069 \
--cors \
--verbose \
--no-browser \
--ignore=deps,incremental,examples,build \
--watch=clean_line.wasm \
target/wasm32-unknown-unknown/debug
