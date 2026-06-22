# Steam Quick Switch — Development

A Windows system-tray app to switch between signed-in Steam accounts. Built with
**Tauri v2** (Rust) plus a minimal React/TypeScript shell. The app is **tray-only** —
there is no application window; the entire UI is a native tray menu built in Rust.

## How account switching works

The app closes Steam, points `AutoLoginUser` (Windows registry) and
`loginusers.vdf` at the chosen account, then relaunches Steam, which auto-logs
in. **No admin rights required.**

Constraints:

- Windows only.
- Only accounts with **"Remember password"** (stored in `loginusers.vdf`) can be
  switched without re-entering credentials.
- Switching closes and reopens Steam.
- Steam Guard / 2FA may still prompt.

## Project layout

- `src-tauri/src/` — Rust backend (the whole app):
  - `steam/` — registry lookup, `loginusers.vdf` parsing, avatars, switching
  - `tray.rs` — native tray menu (accounts + settings) and the dynamic icon
  - `settings.rs` — `settings.json` via the store plugin (`language`, `nameMode`)
  - `i18n.rs` — English/German menu labels
  - `lib.rs` — app entry, plugin registration, background update check
- `src/` — minimal React shell; no window is shown, it only satisfies the build
- `.github/workflows/release.yml` — signed release build (tauri-action)

## Develop

```bash
npm install
npm run tauri dev     # runs the app (tray only — no window appears)
npm run tauri build   # builds the installer
cargo test --manifest-path src-tauri/Cargo.toml
```

## Settings & data

Stored in `%AppData%\Roaming\SteamQuickSwitch\settings.json`
(`language`, `nameMode`).

## Releasing

See [RELEASING.md](RELEASING.md).

## License

[MIT](LICENSE)
