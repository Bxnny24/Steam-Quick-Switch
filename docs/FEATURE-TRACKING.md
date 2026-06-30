# Steam Quick Switch — Feature Tracking Sheet

**Single canonical spreadsheet** tracking every feature in the app: its user story,
expected behaviour, and status across the audit → test → fix → re-test loop.

- **App type:** Windows-only, tray-only Tauri 2 app (Rust backend, empty React frontend).
- **Source of truth for features:** `src-tauri/src/**` (the React `src/App.tsx` renders `null`).
- **Last updated:** 2026-06-30

## Status legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Done / passing |
| ⬜ | Not started |
| 🔄 | In progress |
| ❌ | Failing / defect found |
| ⚠️ | Passes but has a logistical / UX concern |
| — | Not applicable |

## Canonical status table

> Columns map to the four goal phases:
> **Spec** = user story documented · **Test** = behaviour verified against code/build ·
> **Issue** = defect/UX problem found · **Fixed** = remediation applied · **Re-test** = verified post-fix.

| ID | Feature | Severity if broken | Spec | Test | Issue | Fixed | Re-test |
|------|---------|-------------------|:----:|:----:|:-----:|:-----:|:-------:|
| US-01 | Tray-only operation (no window, stays resident) | High | ✅ | ✅ | — | — | ✅ |
| US-02 | List signed-in Steam accounts | High | ✅ | ⚠️ | BUG-1 | ✅ | ✅ |
| US-03 | Show each account's avatar in the menu | Low | ✅ | ✅ | — | — | ✅ |
| US-04 | Indicate & pin the active account | Medium | ✅ | ✅ | BUG-3 | ✅ | ✅ |
| US-05 | Switch account in one click | High | ✅ | ⚠️ | BUG-2 | ✅ | ✅ |
| US-06 | Close only the foreground game on switch | High | ✅ | ✅ | — | — | ✅ |
| US-07 | Tray icon shows active account avatar + tooltip | Low | ✅ | ✅ | BUG-3 | ✅ | ✅ |
| US-08 | Detect account switches made outside the app | Medium | ✅ | ✅ | — | — | ✅ |
| US-09 | Display-name mode (profile vs account name) | Low | ✅ | ✅ | — | — | ✅ |
| US-10 | Language setting (English/German) + OS-locale default | Low | ✅ | ✅ | — | — | ✅ |
| US-11 | Start with Windows (autostart) toggle + default-on | Medium | ✅ | ✅ | — | — | ✅ |
| US-12 | Automatic background updates | Medium | ✅ | ✅ | — | — | ✅ |
| US-13 | Single-instance enforcement | Medium | ✅ | ✅ | — | — | ✅ |
| US-14 | One-time legacy data migration | Low | ✅ | ✅ | BUG-4 | ✅ | ✅ |
| US-15 | Locate the Steam installation | High | ✅ | ✅ | — | — | ✅ |
| US-16 | Empty state when no accounts are found | Medium | ✅ | ✅ | — | — | ✅ |
| US-17 | Quit the app | Medium | ✅ | ✅ | — | — | ✅ |
| US-18 | Reject unsafe `installdir` in appmanifest (traversal guard) | High | ✅ | ✅ | — | — | ✅ |

### Test method
- `cargo test --lib` → 5/5 unit tests pass (US-18 traversal guard directly verified), 1 ignored (needs live Steam).
- `cargo clippy --all-targets` → compiles; 2 style lints only (see BUG-4).
- `npm run build` → frontend bundles clean.
- All other stories verified by tracing the exact code path against the expected behaviour (tray-only app cannot be GUI-driven headlessly).

---

## User stories & expected behaviour

### US-01 — Tray-only operation
**As a** user, **I want** the app to live entirely in the system tray with no window,
**so that** it stays out of my way but is always one click away.

**Expected behaviour**
- On launch no window is created (`tauri.conf.json` `windows: []`; `App.tsx` returns `null`).
- Closing/last-window events never quit the app: `RunEvent::ExitRequested` with `code.is_none()` calls `api.prevent_exit()` (`lib.rs:33`).
- The process only terminates via the **Quit** menu item (`app.exit(0)`).

**Source:** `lib.rs`, `src/App.tsx`, `tray.rs`

---

### US-02 — List signed-in Steam accounts
**As a** user with multiple Steam accounts, **I want** the tray menu to list every
account Steam has remembered, **so that** I can pick one to switch to.

**Expected behaviour**
- Accounts are read from `<steam>/config/loginusers.vdf` (`vdf.rs::parse_login_users`).
- Each account exposes steamID64, account name, persona name, RememberPassword, MostRecent, timestamp (`accounts.rs::Account`).
- Order: active account first, then most-recently-used (descending `Timestamp`) (`accounts.rs:34,55`).

**Source:** `accounts.rs`, `vdf.rs`, `tray.rs::build_menu`

---

### US-03 — Account avatars in the menu
**As a** user, **I want** each account row to show that account's Steam avatar,
**so that** I can recognise accounts visually.

**Expected behaviour**
- Avatar PNG read from `<steam>/config/avatarcache/<steamID64>.png` (`avatar.rs::avatar_path`).
- Rendered at 18px as an anti-aliased rounded square (`round_icon_rgba`, `apply_rounded_mask`).
- If no cached avatar exists, the row shows no icon (graceful: `icon` is `None`).

**Source:** `avatar.rs`, `tray.rs::menu_icon`

---

### US-04 — Indicate & pin the active account
**As a** user, **I want** the currently-active account clearly marked and at the top,
**so that** I always know which account I'm on.

**Expected behaviour**
- "Active" account = the one matching `HKCU\...\Steam\AutoLoginUser` (case-insensitive) (`accounts.rs:39`).
- Active row is pinned to the top (`sort_by_key(|a| !a.is_current)`), labeled `"<name>  •  active"`, and **disabled** (not clickable) (`tray.rs:57,68`).

**Source:** `accounts.rs`, `tray.rs::build_menu`, `registry.rs::auto_login_user`

---

### US-05 — Switch account in one click
**As a** user, **I want** to click an account and have Steam log into it without
retyping my password, **so that** switching is effortless.

**Expected behaviour**
- Clicking a non-active row triggers `switch:<steamID64>` → `switch_to` → `switch_account` (off the main thread) (`tray.rs:199,228`).
- Switch sequence (`switch.rs::switch_account`):
  1. If Steam is running, close the foreground game, then hard-kill `steam.exe` + `steamwebhelper.exe`.
  2. Set `AutoLoginUser` (+ `RememberPassword=1`) in `HKCU` (`registry.rs::set_auto_login_user`).
  3. Relaunch `steam.exe -login <account_name>` (uses Steam's cached token — no admin, no password).
- After switching, the tray is refreshed (`refresh`).
- Only accounts with a cached token / "Remember password" log in silently; Steam Guard / 2FA may still prompt (documented in README).

**Source:** `switch.rs`, `tray.rs::switch_to`, `registry.rs`

---

### US-06 — Close only the foreground game on switch
**As a** user, **I want** only the game I'm playing to be closed on switch,
**so that** background Steam apps (e.g. Wallpaper Engine) keep running.

**Expected behaviour**
- The running game is resolved from `RunningAppID` → matching `appmanifest_<id>.acf` → `installdir` → `steamapps/common/<installdir>` across all library roots (`switch.rs::running_game_dir`, `steam_library_roots`).
- Only processes whose exe path is inside that one game directory are force-killed (`kill_running_game`).
- No graceful shutdown / save wait (documented: user expected to save first).
- If no game is running (`RunningAppID == 0`), nothing is killed beyond Steam itself.

**Source:** `switch.rs`, `registry.rs::running_app_id`

---

### US-07 — Tray icon shows active account avatar + tooltip
**As a** user, **I want** the tray icon to be my current account's avatar with its name
as the tooltip, **so that** I can see who I'm logged in as at a glance.

**Expected behaviour**
- Tray icon = active account's avatar as a 32px rounded square; falls back to the first account if none is "current" (`tray.rs::refresh_icon`).
- Tooltip = the account's display name (per name-mode setting).
- If no avatar is cached, the icon remains the default window icon.

**Source:** `tray.rs::refresh_icon`, `avatar.rs`

---

### US-08 — Detect account switches made outside the app
**As a** user, **I want** the tray to update if I switch accounts via Steam itself
or another tool, **so that** it never shows stale state.

**Expected behaviour**
- A background thread polls `AutoLoginUser` every 3s (`tray.rs::start_account_watcher`, `WATCH_INTERVAL`).
- When the value changes, the tray menu + icon are rebuilt on the main thread.

**Source:** `tray.rs::start_account_watcher`, `current_account_key`

---

### US-09 — Display-name mode (profile vs account name)
**As a** user, **I want** to choose whether accounts show their Steam profile name or
login/account name, **so that** the list matches how I think of my accounts.

**Expected behaviour**
- Setting `nameMode` ∈ {`persona`, `account`}, default `persona` (`settings.rs::name_mode`).
- `persona`: show PersonaName, falling back to AccountName if empty; `account`: always AccountName (`tray.rs::display_name`).
- Toggled from **Settings → Display name**; change persists and rebuilds the menu immediately.

**Source:** `settings.rs`, `tray.rs::display_name`, `i18n.rs`

---

### US-10 — Language setting (English/German)
**As a** user, **I want** the menu in English or German, **so that** I can read it in my language.

**Expected behaviour**
- Setting `language` ∈ {`en`, `de`}; if unset, detected from OS locale (`de*` → German, else English) (`settings.rs::language`, `detect_language`).
- Toggled from **Settings → Language**; persists and rebuilds the menu immediately (`i18n.rs::labels`).
- Only menu chrome is translated; account names are untouched.

**Source:** `settings.rs`, `i18n.rs`, `tray.rs`

---

### US-11 — Start with Windows (autostart)
**As a** user, **I want** the app to optionally launch at login, enabled by default,
**so that** it's always available without me remembering to start it.

**Expected behaviour**
- On first run only, autostart is enabled and a `autostartConfigured` flag is stored so it's never silently re-enabled (`settings.rs::ensure_autostart_default`).
- **Settings → Start with Windows** is a checkbox reflecting the live OS state (`tray.rs:108`) and toggling it enables/disables the launch agent.
- Registered with the `--minimized` argument (`lib.rs:15`).

**Source:** `settings.rs::ensure_autostart_default`, `tray.rs` autostart handler, `lib.rs`

---

### US-12 — Automatic background updates
**As a** user, **I want** the app to update itself, **so that** I always run the latest version.

**Expected behaviour**
- On startup a background task checks the updater endpoint (GitHub `latest.json`) (`lib.rs::try_update`).
- If an update exists, it is downloaded, installed, and the app restarts.
- Update integrity is verified against the configured minisign `pubkey` (`tauri.conf.json`).

**Source:** `lib.rs::try_update`, `tauri.conf.json` updater config

---

### US-13 — Single-instance enforcement
**As a** user, **I want** only one copy of the app to run, **so that** I don't get duplicate
tray icons.

**Expected behaviour**
- `tauri-plugin-single-instance` ensures a second launch is suppressed (no-op handler) (`lib.rs:11`).

**Source:** `lib.rs`

---

### US-14 — One-time legacy data migration
**As a** returning user (upgrading from the old build id), **I want** my settings preserved,
**so that** language/name-mode/autostart survive the rename.

**Expected behaviour**
- On startup, if a legacy `com.bxnny24.steamquickswitch` data dir exists, copy `settings.json` to the new dir (only if the new one doesn't exist yet), then delete the legacy Roaming + Local folders (`settings.rs::migrate_legacy_data`).
- Idempotent: once legacy folders are gone it does nothing.
- Runs before any store read so the autostart flag carries over (`lib.rs:21`).

**Source:** `settings.rs::migrate_legacy_data`, `lib.rs`

---

### US-15 — Locate the Steam installation
**As a** user, **I want** the app to find Steam automatically, **so that** I don't configure paths.

**Expected behaviour**
- Prefer `HKCU\Software\Valve\Steam\SteamPath` (forward slashes normalised to back-slashes); fall back to `HKLM` `InstallPath` (incl. WOW6432Node) (`registry.rs::steam_path`).
- If Steam can't be found, `list_accounts` returns an error and the menu shows the empty state (US-16).

**Source:** `registry.rs::steam_path`

---

### US-16 — Empty state when no accounts are found
**As a** user with no remembered accounts (or no Steam), **I want** a clear placeholder,
**so that** the app doesn't look broken.

**Expected behaviour**
- When `list_accounts` yields nothing (no Steam, no `loginusers.vdf`, or empty), the menu shows a single disabled item: "No accounts found" / "Keine Konten gefunden" (`tray.rs:51`, `i18n.rs`).
- Settings + Quit remain available.

**Source:** `tray.rs::build_menu`, `i18n.rs`

---

### US-17 — Quit the app
**As a** user, **I want** a Quit option, **so that** I can fully exit the app.

**Expected behaviour**
- **Quit** menu item calls `app.exit(0)` (`tray.rs:222`), the only path that overrides `prevent_exit`.

**Source:** `tray.rs`, `lib.rs`

---

### US-18 — Reject unsafe `installdir` (path-traversal guard)
**As a** user, **I want** a crafted appmanifest to never be able to broaden the kill set
beyond the real game directory, **so that** switching can't terminate unrelated processes.

**Expected behaviour**
- `installdir` is only accepted if it is a non-empty relative path made solely of normal components — no `..`, `.`, drive, or UNC prefix (`switch.rs::is_safe_relative_dir`, `installdir_from_manifest`).
- Empty / absolute / traversal values are rejected (covered by unit tests `traversal_installdir_is_rejected`, `empty_installdir_is_rejected`, `safe_relative_dir_accepts_plain_names_only`).

**Source:** `switch.rs`

---

## Test results — defects found (phase 2)

| Ref | Title | Story | Type | Severity | Status |
|-----|-------|-------|------|----------|--------|
| BUG-1 | `Account.avatar` base64 data-URL is built on every `list_accounts()` call but never consumed (no `invoke_handler`, frontend is `null`, tray draws icons via `avatar_path`/`round_icon_rgba`). Redundant disk read + base64 of every avatar on every menu rebuild. | US-02/US-03 | Logistical (dead code / wasted I/O) | Medium | ✅ **Fixed** — removed the `avatar` field, the `avatar_data_url` fn, and the now-unused `base64` dependency. |
| BUG-2 | Switch failures are swallowed (`let _ = switch_account(...)`) and `steam_path() == None` silently does nothing. On a real failure (e.g. Steam fails to close within 5 s) the user clicks, Steam may vanish, and **no message is shown**. | US-05 | UX (silent failure on the primary action) | Medium | ✅ **Fixed** — `switch_to` now captures the `Result` and shows a localized native error dialog (`show_error`, `MessageBoxW`, no new dependency). |
| BUG-3 | `refresh()` parsed accounts twice — once in `build_menu`, once in `refresh_icon`. Correct, but redundant work. | US-04/US-07 | Logistical (minor perf) | Low | ✅ **Fixed** — `refresh`/`setup` fetch accounts once and pass `&[Account]` to both helpers. |
| BUG-4 | clippy: `collapsible_if` (`settings.rs:88`), `unnecessary_sort_by` (`accounts.rs:34`). Not gated by CI (audit only). | US-14 | Code quality | Low | ✅ **Fixed** — collapsed the `if`, switched to `sort_by_key(Reverse(..))`. clippy now clean. |

## Re-test results (phase 4)

Re-ran the full verification after the fixes:

- `cargo test --lib` → **5/5 pass**, 1 ignored (unchanged).
- `cargo clippy --all-targets` → **0 warnings** (both prior lints gone, new FFI clean).
- `npm run build` → still bundles clean (frontend untouched).
- **Behaviour re-traced for all 18 stories — no regressions:**
  - US-02 sort is unchanged semantically (`sort_by_key(Reverse(timestamp))` ≡ descending `Timestamp`); removing the unused `avatar` field does not affect the menu, which never read it.
  - US-03 / US-07 still render avatars via the untouched `avatar_path` + `round_icon_rgba`.
  - US-14 migration logic is byte-for-byte equivalent after collapsing the nested `if` (same short-circuit).
  - US-05 happy path unchanged; only adds a user-visible error dialog on failure.
- Note: the native error dialog (BUG-2) is verified to compile and the FFI signature is correct; the on-screen dialog itself cannot be asserted in a headless run — it requires a live failed switch on a Windows desktop session.

## Resolved (verified not a defect)
- **Version `0.1.0` in source vs `0.1.6` in README** — intentional. `release.yml` rewrites the version from the git tag at build time and `scripts/update-readme-version.mjs` updates the badge; source files are not meant to track releases (per `RELEASING.md`). ✅
- **`capabilities/default.json` references window `main`** — inert. The app has no windows and no IPC commands, so the capability applies to nothing. Left unchanged to avoid affecting any future window permissions. ✅
</content>
