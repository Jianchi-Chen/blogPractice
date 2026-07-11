# Deployment And Migration

## Target Architecture

Browser and Tauri builds use the same Vue frontend and the same Axum HTTP API.
The server SQLite database is authoritative. The Tauri Rust layer contains only
desktop integration for the tray and signed updater.

## Required Configuration

Set `VITE_API_BASE_URL` to the public Axum server origin:

```env
VITE_API_BASE_URL=https://api.example.com
```

Do not append `/api`. Local frontend development defaults to
`http://127.0.0.1:3000`; formal Web and Tauri workflows require an HTTPS GitHub
Actions repository variable named `VITE_API_BASE_URL`.

## Rollout Order

1. Back up the server database.
2. Deploy the backend compatibility release and let SQLx apply migration
   `0007_remote_api_profiles_and_favorites.sql`.
3. Verify `/health`, authentication, search, favorites, profile, avatar, and
   comment-like contract tests against the deployed service.
4. Build/deploy Web and Tauri clients with the same API origin.
5. Confirm CORS/network policy allows the Web origin and installed desktop app.
6. After old Web assets have expired and access logs show no legacy-path calls,
   remove `backend/src/routes/legacy.rs` in a separate change.

Migration 0007 adds user signatures, avatar data/content type, and the
server-side favorites table. It does not import records from a desktop database.

## Existing Desktop Installations

- Existing local `app.db` files are deliberately left untouched.
- The new desktop application does not open, migrate, overwrite, or delete that
  database. It becomes an unused local artifact.
- Tokens issued by the removed embedded Tauri backend use a different signing
  key. Startup validation will reject them and the user must log in again.
- Local-only favorites, signatures, or avatars are not silently uploaded. The
  server starts as the authoritative state after login.

Do not add cleanup code for legacy databases. Any future import or deletion must
be a separate, explicit migration with user consent and rollback coverage.

## Rollback

Rolling back the client only changes which frontend is used; it must not remove
server migration data or local desktop files. Before rolling back the backend,
confirm the old version tolerates migration 0007, or restore the database backup
as a coordinated server operation.
