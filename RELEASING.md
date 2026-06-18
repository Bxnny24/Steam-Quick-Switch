# Releasing

Releases are built, signed, and published automatically by GitHub Actions
(`.github/workflows/release.yml`) whenever a `v*` tag is pushed.

## One-time setup

Add a repository secret under **Settings → Secrets and variables → Actions**:

- `TAURI_SIGNING_PRIVATE_KEY` — the full contents of the local, git-ignored
  `src-tauri/sqs-updater.key` file. Get it with:

  ```powershell
  Get-Content src-tauri/sqs-updater.key -Raw
  ```

- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — leave **empty** (the key has no password).

> Keep `sqs-updater.key` secret and backed up. If it is lost, already-installed
> apps can no longer verify and receive signed updates.

## Cutting a release

1. Bump the version in all three files so they match:
   - `src-tauri/tauri.conf.json` → `version`
   - `package.json` → `version`
   - `src-tauri/Cargo.toml` → `[package] version`
2. Commit the bump:
   ```
   git commit -am "chore: release vX.Y.Z"
   ```
3. Tag and push:
   ```
   git tag vX.Y.Z
   git push origin main --tags
   ```
4. The **Release** workflow builds the Windows installer (NSIS + MSI), signs the
   update artifacts, and publishes a GitHub Release containing the installers and
   `latest.json`.

Installed apps fetch `latest.json` on startup and update themselves when a newer
version is available.
