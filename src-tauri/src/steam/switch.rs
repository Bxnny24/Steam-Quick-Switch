//! Switching the active Steam account.
//!
//! The mechanism (no admin rights required):
//!   1. Ask Steam to shut down and wait for it to exit.
//!   2. Point `AutoLoginUser`/`RememberPassword` at the target account.
//!   3. Mark the target as most-recent in `loginusers.vdf` (best effort).
//!   4. Relaunch Steam, which auto-logs into the target account.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crate::steam::{registry, vdf};

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

/// Switch the active Steam account and restart Steam so it auto-logs in.
pub fn switch_account(
    steam_path: &Path,
    account_name: &str,
    steam_id64: &str,
) -> Result<(), String> {
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

    registry::set_auto_login_user(account_name)?;

    if let Err(e) = vdf::set_most_recent(steam_path, steam_id64) {
        eprintln!("warning: could not update loginusers.vdf: {e}");
    }

    silent_command(&steam_exe)
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
