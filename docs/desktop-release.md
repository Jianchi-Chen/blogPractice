# Desktop Release

The Tauri application is a delivery shell around the shared Vue/Axum product.
Its Rust code owns the system tray and signed updater only.

## GitHub Configuration

Create this repository variable under Actions:

- `VITE_API_BASE_URL`: public HTTPS Axum origin, without `/api`.

Create these repository secrets:

- `TAURI_SIGNING_PRIVATE_KEY`: updater signing private key.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: private-key password, when configured.

The matching public key remains in `frontend/src-tauri/tauri.conf.json`. Never
commit the private key; `*.key` and `*.key.pub` are ignored.

## Release Flow

Push a semantic version tag such as `v1.2.3`, or manually run `Tauri Release`
with the same tag format. The workflow:

1. Validates the release tag and production API URL.
2. Runs backend contract tests, frontend tests, and Tauri tests.
3. Builds signed updater artifacts for Windows, macOS x64/ARM64, and Linux.
4. Uploads all matrix artifacts to a draft GitHub Release.
5. Publishes the release only after every platform succeeds.

Tags with a prerelease suffix, such as `v1.2.3-beta.1`, remain GitHub
prereleases and are not exposed as the stable updater release.

The draft phase prevents installed clients from observing a partial
`latest.json`. The application checks for updates after the frontend event
listener is ready and also exposes a tray action. Concurrent checks are ignored.

## Key Rotation

If the private key is lost or exposed, generate a new updater key pair, replace
the GitHub secrets and the public key in `tauri.conf.json`, then ship through a
trusted transition plan. Clients signed against the old public key cannot accept
artifacts signed only by an unrelated new key.
