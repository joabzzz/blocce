# Blocce Build & Test Commands (Docker-based)

# Show all available commands
default:
    @just --list

# Run dev CI checks in Docker (same as GitHub Actions)
ci:
    @echo "==> Running CI checks in Docker..."
    docker-compose -f build/docker-compose.yml run --rm -T ci
    @echo "✅ CI checks passed!"

# Build releases for all platforms in Docker
release:
    @echo "==> Building Windows release in Docker..."
    docker-compose -f build/docker-compose.yml run --rm -T windows
    @echo "==> Note: macOS builds require macOS hardware (use GitHub Actions)"
    @echo "✅ Release builds complete!"

# Run all checks and tests
test-all: ci
    @echo "==> Running tests in Docker..."
    docker-compose -f build/docker-compose.yml run --rm -T ci sh -c "cargo test"
    @echo "✅ All tests passed!"

# Clean Docker images and build artifacts
clean:
    @echo "==> Cleaning Docker images and artifacts..."
    docker-compose -f build/docker-compose.yml down -v
    rm -rf dist/ target/
    @echo "✅ Clean complete!"

# Rebuild Docker images (use when Dockerfiles change)
rebuild:
    @echo "==> Rebuilding Docker images..."
    docker-compose -f build/docker-compose.yml build --no-cache

# Quick local dev run (no Docker)
dev:
    cargo run

# Format code (no Docker needed)
fmt:
    cargo fmt
