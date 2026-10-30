# Version Format

Your releases now include version numbers from `Cargo.toml` in the artifact names.

## Version Format Support

### Standard Versions (Supported)
```toml
version = "0.1.0"       → Blocce-0.1.0-x64.zip
version = "1.2.3"       → Blocce-1.2.3-x64.zip
```

### Pre-release Versions (Supported)
```toml
version = "0.1.1-alpha"     → Blocce-0.1.1-alpha-x64.zip
version = "0.1.1-beta.1"    → Blocce-0.1.1-beta.1-x64.zip
version = "0.1.1-rc.2"      → Blocce-0.1.1-rc.2-x64.zip
version = "1.0.0-alpha"     → Blocce-1.0.0-alpha-x64.zip
```

**Important:** Use hyphens (`-`), NOT dots (`.`) for pre-release identifiers.

### Invalid Formats
```toml
version = "0.1.1.alpha"     ❌ Won't work (uses dot instead of hyphen)
```

## Artifact Names

### Local Builds (via `just release`)
- Windows: `dist/Blocce-{version}-x64.zip`
- macOS: `dist/Blocce-{version}.dmg`

### GitHub Actions
- Staging: Artifacts named with version
- Main: Release tag format `v{version}-build-{run_number}`
  - Example: `v0.1.0-build-42`
  - Files: `Blocce-0.1.0-x64.zip`, `Blocce-0.1.0.dmg`

## Testing

```bash
# Current version
just release
# Creates: dist/Blocce-0.1.0-x64.zip

# Update Cargo.toml to test
version = "0.2.0-alpha"
just release
# Creates: dist/Blocce-0.2.0-alpha-x64.zip
```
