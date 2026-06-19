# Steam Quick Switch

A lightweight Windows **tray app** to switch between your signed-in Steam
accounts in one click. It lives entirely in the system tray — there is no main
window. The tray icon shows the avatar of the currently active account, and a
click opens a menu of all your profiles.

## Features

- No window — left- or right-click the tray icon for a menu of all your profiles
- Each profile shows its Steam avatar; one click switches to it
- The tray icon itself is the active account's avatar
- Avatars and names read from Steam's local cache (no API key, works offline)
- Settings right in the tray menu: language (English/German) and start with Windows
- Custom nickname per account, set from the tray menu (a small input popup)
- Automatic, cryptographically signed updates

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
