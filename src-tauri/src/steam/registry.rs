//! Reading Steam-related values from the Windows registry.
//!
//! All reads here are user-scoped (`HKCU`) and need no admin rights. The
//! `HKLM` fallback is only consulted to locate the install directory.

use std::path::PathBuf;

use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

const STEAM_KEY: &str = r"Software\Valve\Steam";

/// Resolve the Steam installation directory.
///
/// Prefers `HKCU\Software\Valve\Steam\SteamPath` (set by the running client),
/// falling back to the machine-wide `InstallPath`.
pub fn steam_path() -> Option<PathBuf> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(steam) = hkcu.open_subkey(STEAM_KEY) {
        if let Ok(path) = steam.get_value::<String, _>("SteamPath") {
            if !path.trim().is_empty() {
                // Steam stores this with forward slashes.
                return Some(PathBuf::from(path.replace('/', "\\")));
            }
        }
    }

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    for key in [r"SOFTWARE\WOW6432Node\Valve\Steam", r"SOFTWARE\Valve\Steam"] {
        if let Ok(steam) = hklm.open_subkey(key) {
            if let Ok(path) = steam.get_value::<String, _>("InstallPath") {
                if !path.trim().is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }
    }

    None
}

/// The account name Steam will auto-login on next launch.
///
/// This is the source of truth for "which account is currently active".
pub fn auto_login_user() -> Option<String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let steam = hkcu.open_subkey(STEAM_KEY).ok()?;
    let user: String = steam.get_value("AutoLoginUser").ok()?;
    if user.trim().is_empty() {
        None
    } else {
        Some(user)
    }
}

/// Point Steam at `account_name` for the next launch and remember the password.
/// Writes to `HKCU` only, so no admin rights are required.
pub fn set_auto_login_user(account_name: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (steam, _) = hkcu
        .create_subkey(STEAM_KEY)
        .map_err(|e| format!("failed to open Steam registry key: {e}"))?;
    steam
        .set_value("AutoLoginUser", &account_name.to_string())
        .map_err(|e| format!("failed to set AutoLoginUser: {e}"))?;
    steam
        .set_value("RememberPassword", &1u32)
        .map_err(|e| format!("failed to set RememberPassword: {e}"))?;
    Ok(())
}
