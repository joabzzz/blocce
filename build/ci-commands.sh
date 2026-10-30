#!/bin/bash

# Shared CI commands used by both GitHub Actions and local Docker testing
# This ensures the CI flow is always identical

set -e

echo "==> Installing rustup components..."
rustup component add rustfmt clippy rust-src rust-analysis

echo ""
echo "==> Running Clippy..."
CARGO_BUILD_JOBS=2 cargo clippy --all-targets -- -D warnings

echo ""
echo "==> Building (Release)..."
cargo build --release

echo ""
echo "==> Building Tests (single-threaded to reduce memory)..."
CARGO_BUILD_JOBS=1 cargo test --all --no-run

echo ""
echo "==> Running Tests..."
cargo test --all

echo ""
echo "==> CI Complete!"
