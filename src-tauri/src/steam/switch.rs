//! Switching the active Steam account.
//!
//! The mechanism (no admin rights required):
//!   1. Close the running foreground game (the one Steam reports via
//!      `RunningAppID`), then hard-kill Steam (`steam.exe` + `steamwebhelper.exe`).
//!   2. Point `AutoLoginUser` at the target account (so the tray reflects it).
//!   3. Relaunch via `steam.exe -login <account>`, which uses Steam's own
//!      cached login token — the same path as Steam's built-in account
//!      switcher, so remembered accounts log in without a prompt.
//!
//! The running game is terminated directly, without waiting for it to save:
//! switching accounts has to close it anyway, and the user is expected to save
//! before switching. Only the foreground game is touched — background Steam
//! apps (e.g. Wallpaper Engine) keep running.

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use keyvalues_parser::Value;
use sysinfo::System;

use crate::steam::registry;

/// How long to wait for Steam to disappear after the force-kill before giving
/// up. A force-kill is near-instant, so this only guards against a stuck handle.
const KILL_CONFIRM_TIMEOUT: Duration = Duration::from_secs(5);
const POLL_INTERVAL: Duration = Duration::from_millis(300);
/// `CREATE_NO_WINDOW`: stops console helpers (tasklist/taskkill) from flashing.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Build a `Command` that never pops up a console window.
fn silent_command<S: AsRef<std::ffi::OsStr>>(program: S) -> Command {
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// Switch the active Steam account: close the running foreground game, hard-kill
/// Steam, then relaunch it logged in to `account_name` via Steam's own `-login`
/// path (uses the cached token).
pub fn switch_account(steam_path: &Path, account_name: &str) -> Result<(), String> {
    let steam_exe = steam_path.join("steam.exe");
    if !steam_exe.exists() {
        return Err(format!("steam.exe not found at {}", steam_exe.display()));
    }

    if is_steam_running() {
        // Read RunningAppID and close only the foreground game *before* killing
        // Steam (killing Steam resets RunningAppID to 0). Background Steam apps
        // such as Wallpaper Engine are not the running game, so they stay up.
        kill_running_game(steam_path);
        hard_close_steam()?;
    }

    // Keep AutoLoginUser in sync so the tray reflects the new account at once.
    registry::set_auto_login_user(account_name)?;

    // Relaunch through Steam's own login path; remembered accounts log in
    // without a prompt, exactly like Steam's built-in account switcher.
    silent_command(&steam_exe)
        .arg("-login")
        .arg(account_name)
        .spawn()
        .map_err(|e| format!("failed to relaunch Steam: {e}"))?;

    Ok(())
}

/// Force-terminate the foreground game Steam currently reports as running (via
/// `RunningAppID`). Only processes inside that one game's install directory are
/// killed, so background Steam apps are left alone. No graceful shutdown.
fn kill_running_game(steam_path: &Path) {
    let Some(game_dir) = running_game_dir(steam_path) else {
        return;
    };
    let prefix = format!("{}\\", game_dir.to_string_lossy().to_lowercase());

    let sys = System::new_all();
    for process in sys.processes().values() {
        let Some(exe) = process.exe() else {
            continue;
        };
        if exe.to_string_lossy().to_lowercase().starts_with(&prefix) {
            let _ = process.kill();
        }
    }
}

/// The install directory of the currently running game, resolved from Steam's
/// `RunningAppID` and the matching `appmanifest_<id>.acf`. `None` if no game is
/// running or the manifest cannot be located.
fn running_game_dir(steam_path: &Path) -> Option<PathBuf> {
    let appid = registry::running_app_id()?;

    for root in steam_library_roots(steam_path) {
        let manifest = root
            .join("steamapps")
            .join(format!("appmanifest_{appid}.acf"));
        let Ok(content) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        let Ok(parsed) = keyvalues_parser::parse(&content) else {
            continue;
        };
        if let Some(installdir) = installdir_from_manifest(&parsed.value) {
            return Some(root.join("steamapps").join("common").join(installdir));
        }
    }

    None
}

/// Read the `installdir` value out of a parsed `appmanifest` `AppState` object.
///
/// The value is only returned if it is a safe, plain relative folder name. An
/// empty, absolute, or traversal (`..`) `installdir` is rejected: it is joined
/// onto the trusted library path to form the directory whose processes
/// [`kill_running_game`] terminates, so a crafted value must never be able to
/// escape `steamapps\common` and broaden the kill set.
fn installdir_from_manifest(appstate: &Value) -> Option<String> {
    if let Value::Obj(obj) = appstate {
        for (key, vals) in obj.iter() {
            if key.eq_ignore_ascii_case("installdir") {
                if let Some(Value::Str(s)) = vals.first() {
                    let s = s.as_ref();
                    if is_safe_relative_dir(s) {
                        return Some(s.to_string());
                    }
                }
            }
        }
    }
    None
}

/// Whether `name` is a non-empty relative path made only of normal components
/// (no `..`, no `.`, no root/drive prefix). Joining such a value onto a base
/// directory always stays inside that base, so it is safe against traversal.
fn is_safe_relative_dir(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return false;
    }
    Path::new(trimmed)
        .components()
        .all(|c| matches!(c, std::path::Component::Normal(_)))
}

/// Every Steam library root (the directory that contains `steamapps`), starting
/// with the main install and adding the libraries listed in `libraryfolders.vdf`.
fn steam_library_roots(steam_path: &Path) -> Vec<PathBuf> {
    let mut roots = vec![steam_path.to_path_buf()];

    for vdf_path in [
        steam_path.join("steamapps").join("libraryfolders.vdf"),
        steam_path.join("config").join("libraryfolders.vdf"),
    ] {
        let Ok(content) = std::fs::read_to_string(&vdf_path) else {
            continue;
        };
        let Ok(parsed) = keyvalues_parser::parse(&content) else {
            continue;
        };
        if let Value::Obj(obj) = parsed.value {
            for (_index, values) in obj.iter() {
                if let Some(Value::Obj(entry)) = values.first() {
                    for (key, vals) in entry.iter() {
                        if key.eq_ignore_ascii_case("path") {
                            if let Some(Value::Str(path)) = vals.first() {
                                roots.push(PathBuf::from(path.as_ref()));
                            }
                        }
                    }
                }
            }
        }
        break; // the first existing libraryfolders.vdf wins
    }

    roots
}

/// Whether a `steam.exe` process is currently running.
fn is_steam_running() -> bool {
    silent_command("tasklist")
        .args(["/FI", "IMAGENAME eq steam.exe", "/NH", "/FO", "CSV"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .to_lowercase()
                .contains("steam.exe")
        })
        .unwrap_or(false)
}

/// Hard-close Steam immediately: force-kill `steam.exe` and its UI child
/// `steamwebhelper.exe`, then confirm `steam.exe` is gone before relaunching.
fn hard_close_steam() -> Result<(), String> {
    for image in ["steam.exe", "steamwebhelper.exe"] {
        let _ = silent_command("taskkill")
            .args(["/F", "/IM", image])
            .output();
    }

    let start = Instant::now();
    while start.elapsed() < KILL_CONFIRM_TIMEOUT {
        if !is_steam_running() {
            return Ok(());
        }
        thread::sleep(POLL_INTERVAL);
    }

    Err("Steam did not shut down in time".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installdir_is_extracted_from_manifest() {
        let acf = r#"
"AppState"
{
    "appid"      "431960"
    "name"       "Wallpaper Engine"
    "installdir" "wallpaper_engine"
}
"#;
        let parsed = keyvalues_parser::parse(acf).expect("valid ACF");
        assert_eq!(
            installdir_from_manifest(&parsed.value).as_deref(),
            Some("wallpaper_engine")
        );
    }

    #[test]
    fn missing_installdir_returns_none() {
        let acf = r#"
"AppState"
{
    "appid" "12345"
    "name"  "Some Game"
}
"#;
        let parsed = keyvalues_parser::parse(acf).expect("valid ACF");
        assert_eq!(installdir_from_manifest(&parsed.value), None);
    }

    #[test]
    fn traversal_installdir_is_rejected() {
        // A crafted manifest must not be able to point the kill target outside
        // the Steam library via path traversal.
        let acf = r#"
"AppState"
{
    "appid"      "12345"
    "installdir" "..\\..\\..\\Windows\\System32"
}
"#;
        let parsed = keyvalues_parser::parse(acf).expect("valid ACF");
        assert_eq!(installdir_from_manifest(&parsed.value), None);
    }

    #[test]
    fn empty_installdir_is_rejected() {
        let acf = r#"
"AppState"
{
    "appid"      "12345"
    "installdir" ""
}
"#;
        let parsed = keyvalues_parser::parse(acf).expect("valid ACF");
        assert_eq!(installdir_from_manifest(&parsed.value), None);
    }

    #[test]
    fn safe_relative_dir_accepts_plain_names_only() {
        assert!(is_safe_relative_dir("wallpaper_engine"));
        assert!(is_safe_relative_dir("Counter-Strike Global Offensive"));
        assert!(!is_safe_relative_dir(""));
        assert!(!is_safe_relative_dir("   "));
        assert!(!is_safe_relative_dir(".."));
        assert!(!is_safe_relative_dir("..\\evil"));
        assert!(!is_safe_relative_dir("C:\\Windows"));
        assert!(!is_safe_relative_dir("\\\\server\\share"));
    }
}
