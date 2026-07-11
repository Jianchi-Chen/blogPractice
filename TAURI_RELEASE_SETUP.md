# Tauri Release Setup Guide

## Overview
This document describes how to set up GitHub Actions for building and releasing Tauri applications across multiple platforms.

## GitHub Secrets Required

The workflow requires two secrets to be configured in your GitHub repository:

### 1. TAURI_SIGNING_PRIVATE_KEY
The private key used for signing Tauri updater artifacts. This should be the base64-encoded content of your private key.

**To generate a new key pair:**
```bash
# Using Tauri CLI (recommended)
npm run tauri signer generate -- --write-keys ~/.tauri/myapp.key

# The above will create:
# - ~/.tauri/myapp.key (private key)
# - ~/.tauri/myapp.key.pub (public key)
```

**To add to GitHub secrets:**
1. Go to your repository Settings > Secrets and variables > Actions
2. Click "New repository secret"
3. Name: `TAURI_SIGNING_PRIVATE_KEY`
4. Value: Copy the entire content of your private key file

### 2. TAURI_SIGNING_PRIVATE_KEY_PASSWORD (Optional)
If your private key is password-protected, add the password here. Otherwise, this can be left empty.

## Workflow Configuration

The workflow is configured in `.github/workflows/tauri-release.yml` and:

- Builds for multiple platforms: Windows, macOS (x64 and ARM64), Linux
- Automatically updates version numbers from git tags
- Creates GitHub releases with installers
- Generates updater artifacts when signing keys are provided

## Build Artifacts

### Windows
- `.msi` installer
- `.msi.zip` (updater artifact)

### macOS
- `.dmg` disk image
- `.app.tar.gz` (updater artifact)

### Linux
- `.deb` package
- `.rpm` package
- `.AppImage` portable application
- `.AppImage.tar.gz` (updater artifact)

## Triggering a Release

1. Create and push a version tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. The workflow will automatically start and build for all platforms

3. Or manually trigger via GitHub Actions UI:
   - Go to Actions > Tauri Release > Run workflow

## Important Notes

- **DO NOT commit signing keys** to the repository
- Keys should be stored outside the project directory (e.g., `~/.tauri/`)
- Local `.tauri/` directory and `*.key` files are excluded by .gitignore
- If secrets are not configured, the build will still work but without signed updater artifacts
- The workflow uses `createUpdaterArtifacts: true` in tauri.conf.json

## Troubleshooting

### Build fails with signing errors
- Ensure `TAURI_SIGNING_PRIVATE_KEY` secret is configured
- Check if the key format is correct (should be Ed25519)
- Verify the public key in `tauri.conf.json` matches your private key

### Artifacts not generated
- Check the workflow logs in GitHub Actions
- Ensure all platform dependencies are installed (workflow handles this automatically)
- Verify `bundle.active: true` in tauri.conf.json
