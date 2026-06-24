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

## Signing key security (important)

The updater public key is **baked into every released binary and cannot be
revoked**. This makes the private signing key the single trust anchor for all
auto-updates — treat it like a production credential:

- **A leak is unrecoverable for existing installs.** Anyone with the private key
  can sign updates that already-installed clients will accept. The only remedy is
  shipping a new build with a *new* keypair, which existing users must install
  manually. There is no online revocation.
- **No password is currently set** (`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` is
  empty). Consider regenerating the keypair *with* a password and storing that
  password as the secret, so the key file alone is not immediately usable if it
  leaks. (A new keypair changes the public key, so it only applies to fresh
  installs.)
- **Keep it out of the repo.** `*.key`/`*.pem` are git-ignored; never commit the
  key. Store it only in GitHub Actions secrets plus an offline backup, and
  minimise who can read the Actions secrets.
- **Restrict who can release.** Only trusted maintainers should be able to push
  `v*` tags. Consider putting the release job behind a GitHub Environment with
  required reviewers so a release cannot be cut (and the key cannot be used)
  without approval.

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
