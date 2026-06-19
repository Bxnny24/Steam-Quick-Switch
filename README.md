# Steam Quick Switch

A lightweight Windows **tray app** to switch between your signed-in Steam
accounts in one click. The tray icon shows the avatar of the account that is
currently active.

## Features

- Lives in the system tray — the tray icon is the current account's avatar
- One-click switching between all remembered Steam accounts
- Avatars and names read from Steam's local cache (no API key, works offline)
- English and German (auto-detected, switchable)
- Custom nicknames per account (right-click an account to rename it)
- Settings: language and start with Windows
- Automatic, cryptographically signed updates
- Clean, modern light/dark UI

## How it works

To switch accounts the app closes Steam, points `AutoLoginUser` (in the Windows
registry) and `loginusers.vdf` at the chosen account, and relaunches Steam,
which then auto-logs in. **No admin rights required.**

### Requirements & limitations

- Windows only.
- Only accounts with **"Remember password"** (stored in `loginusers.vdf`) can be
  switched without re-entering credentials.
- Switching closes and reopens Steam.
- Steam Guard / 2FA may still prompt — that cannot be bypassed.

## Install

Download the latest installer from the
[Releases](https://github.com/Bxnny24/Steam-Quick-Switch/releases) page and run
it. Windows SmartScreen may warn because the app is not code-signed yet — choose
**More info → Run anyway**. After the first install the app updates itself
automatically.

## Settings & data

Settings are stored in
`%AppData%\Roaming\com.bxnny24.steamquickswitch\settings.json`.

## Development

```bash
npm install
npm run tauri dev     # run the app
npm run tauri build   # build the installer
```

Tech stack: **Tauri v2** (Rust) + **React** + **TypeScript**.

## Releasing

See [RELEASING.md](RELEASING.md).

## License

[MIT](LICENSE)
