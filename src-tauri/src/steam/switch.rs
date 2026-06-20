//! Switching the active Steam account.
//!
//! The mechanism (no admin rights required):
//!   1. Hard-kill any running game, then ask Steam to shut down and wait.
//!   2. Point `AutoLoginUser`/`RememberPassword` at the target account.
//!   3. Mark the target as most-recent in `loginusers.vdf` (best effort).
//!   4. Relaunch Steam, which auto-logs into the target account.
//!
//! Running games are terminated directly, without waiting for them to save:
//! switching accounts has to close them anyway, and the user is expected to
//! save before switching.

use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use keyvalues_parser::Value;
use sysinfo::System;

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
        // Hard-close any running game before shutting Steam down. Killing only
        // Steam would leave the game orphaned, so we terminate the game process
        // directly and deliberately do not wait for it to save.
        kill_running_games(steam_path);

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

/// Force-terminate every process running from a Steam library's
/// `steamapps/common` directory (i.e. installed games). No graceful shutdown.
fn kill_running_games(steam_path: &Path) {
    let commons = steam_library_common_dirs(steam_path);
    if commons.is_empty() {
        return;
    }

    let sys = System::new_all();
    for process in sys.processes().values() {
        let Some(exe) = process.exe() else {
            continue;
        };
        let exe_lower = exe.to_string_lossy().to_lowercase();
        if commons.iter().any(|dir| exe_lower.starts_with(dir)) {
            let _ = process.kill();
        }
    }
}

/// All `steamapps/common` directories across every Steam library, lowercased
/// and terminated with a separator so they match as path prefixes.
fn steam_library_common_dirs(steam_path: &Path) -> Vec<String> {
    let mut dirs = vec![steam_path.join("steamapps").join("common")];

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
                                dirs.push(
                                    PathBuf::from(path.as_ref())
                                        .join("steamapps")
                                        .join("common"),
                                );
                            }
                        }
                    }
                }
            }
        }
        break; // the first existing libraryfolders.vdf wins
    }

    let mut normalised: Vec<String> = dirs
        .iter()
        .map(|d| format!("{}\\", d.to_string_lossy().to_lowercase()))
        .collect();
    normalised.sort();
    normalised.dedup();
    normalised
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
