//! Switching the active Steam account.
//!
//! The mechanism (no admin rights required):
//!   1. Ask Steam to shut down and wait for it to exit.
//!   2. Point `AutoLoginUser` at the target account (so the tray reflects it).
//!   3. Relaunch via `steam.exe -login <account>`, which uses Steam's own
//!      cached login token — the same path as Steam's built-in account
//!      switcher, so remembered accounts log in without a prompt.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crate::steam::registry;

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_millis(300);
/// `CREATE_NO_WINDOW`: stops console helpers (tasklist/taskkill) from flashing.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Build a `Command` that never pops up a console window.
fn silent_command<S: AsRef<std::ffi::OsStr>>(program: S) -> Command {
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// Switch the active Steam account: shut Steam down, then relaunch it logged
/// in to `account_name` via Steam's own `-login` path (uses the cached token).
pub fn switch_account(steam_path: &Path, account_name: &str) -> Result<(), String> {
    let steam_exe = steam_path.join("steam.exe");
    if !steam_exe.exists() {
        return Err(format!("steam.exe not found at {}", steam_exe.display()));
    }

    if is_steam_running() {
        silent_command(&steam_exe)
            .arg("-shutdown")
            .spawn()
            .map_err(|e| format!("failed to ask Steam to shut down: {e}"))?;
        wait_for_steam_exit()?;
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

/// Wait for Steam to exit after `-shutdown`, force-killing as a last resort.
fn wait_for_steam_exit() -> Result<(), String> {
    let start = Instant::now();
    while start.elapsed() < SHUTDOWN_TIMEOUT {
        if !is_steam_running() {
            return Ok(());
        }
        thread::sleep(POLL_INTERVAL);
    }

    let _ = silent_command("taskkill")
        .args(["/F", "/IM", "steam.exe"])
        .output();
    thread::sleep(Duration::from_millis(500));

    if is_steam_running() {
        Err("Steam did not shut down in time".into())
    } else {
        Ok(())
    }
}
