#!/bin/bash

# Test CI locally using the same Docker image and commands as GitHub Actions
# Usage: ./build/test-ci.sh

set -e

cd "$(dirname "$0")/.."

echo "Building CI Docker image..."
docker build -t blocce-ci:latest -f build/Dockerfile build/

echo ""
echo "Running CI checks..."
docker run --rm \
    -v "$(pwd):/workspace" \
    -e CARGO_TERM_COLOR=always \
    blocce-ci:latest
