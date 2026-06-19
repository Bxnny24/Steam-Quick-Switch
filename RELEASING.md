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

Just push a version tag — the workflow derives the app version from the tag, so
you do **not** need to edit any version files:

```
git tag v0.1.2
git push origin v0.1.2
```

The **Release** workflow writes the version (from the tag) into
`tauri.conf.json`, `package.json` and `Cargo.toml`, builds the Windows installer
(NSIS + MSI), signs the update artifacts, and publishes a GitHub Release with the
installers and `latest.json`.

> The tag (e.g. `v0.1.2`) must be a **higher** version than what users have
> installed, otherwise the updater sees no newer version and does nothing.

Installed apps fetch `latest.json` on startup and update themselves when a newer
version is available.
