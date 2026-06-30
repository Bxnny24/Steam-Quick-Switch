//! Persistent settings, stored in `settings.json` (via the store plugin) in
//! the app data directory. Read and written from Rust so the app needs no
//! window.

use std::fs;

use serde_json::json;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_store::StoreExt;

const STORE: &str = "settings.json";

/// The previous reverse-domain bundle identifier. Data directories created
/// under this name (before the rename to `steam-quick-switch`) are migrated
/// and removed by [`migrate_legacy_data`].
const LEGACY_IDENTIFIER: &str = "com.bxnny24.steamquickswitch";

/// The UI language: "en" or "de". Falls back to the OS locale.
pub fn language(app: &AppHandle) -> String {
    if let Ok(store) = app.store(STORE) {
        if let Some(value) = store.get("language") {
            if let Some(s) = value.as_str() {
                if s == "de" || s == "en" {
                    return s.to_string();
                }
            }
        }
    }
    detect_language()
}

pub fn set_language(app: &AppHandle, lang: &str) {
    if let Ok(store) = app.store(STORE) {
        store.set("language", json!(lang));
        let _ = store.save();
    }
}

/// How account names are shown: "persona" (Steam profile name) or "account".
pub fn name_mode(app: &AppHandle) -> String {
    if let Ok(store) = app.store(STORE) {
        if let Some(value) = store.get("nameMode") {
            if let Some(s) = value.as_str() {
                if s == "account" || s == "persona" {
                    return s.to_string();
                }
            }
        }
    }
    "persona".to_string()
}

pub fn set_name_mode(app: &AppHandle, mode: &str) {
    if let Ok(store) = app.store(STORE) {
        store.set("nameMode", json!(mode));
        let _ = store.save();
    }
}

/// On first run, enable "start with Windows" by default (once). If the user
/// later turns it off, it is not re-enabled.
pub fn ensure_autostart_default(app: &AppHandle) {
    if let Ok(store) = app.store(STORE) {
        if store.get("autostartConfigured").is_none() {
            let _ = app.autolaunch().enable();
            store.set("autostartConfigured", json!(true));
            let _ = store.save();
        }
    }
}

/// One-time migration from the old `com.bxnny24.steamquickswitch` data
/// directories to the new `steam-quick-switch` ones. Copies the settings store
/// across (so the user's language, name mode and autostart flag survive the
/// rename) and removes the orphaned legacy folders in both Roaming and Local
/// app data. Idempotent: once the legacy folders are gone it does nothing.
///
/// The install directory (`%LOCALAPPDATA%\Steam Quick Switch`) is derived from
/// the product name, not the identifier, so it is never touched here.
pub fn migrate_legacy_data(app: &AppHandle) {
    // Roaming: carry the settings store over, then drop the legacy folder.
    if let Ok(new_data) = app.path().app_data_dir() {
        if let Some(legacy_data) = new_data.parent().map(|p| p.join(LEGACY_IDENTIFIER)) {
            if legacy_data.is_dir() {
                let legacy_store = legacy_data.join(STORE);
                let new_store = new_data.join(STORE);
                if legacy_store.is_file()
                    && !new_store.exists()
                    && fs::create_dir_all(&new_data).is_ok()
                {
                    let _ = fs::copy(&legacy_store, &new_store);
                }
                let _ = fs::remove_dir_all(&legacy_data);
            }
        }
    }
    // Local: only WebView2 cache lived here; it is recreated on demand, so just
    // remove the legacy folder.
    if let Ok(new_local) = app.path().app_local_data_dir() {
        if let Some(legacy_local) = new_local.parent().map(|p| p.join(LEGACY_IDENTIFIER)) {
            if legacy_local.is_dir() {
                let _ = fs::remove_dir_all(&legacy_local);
            }
        }
    }
}

fn detect_language() -> String {
    let locale = sys_locale::get_locale().unwrap_or_default().to_lowercase();
    if locale.starts_with("de") {
        "de".to_string()
    } else {
        "en".to_string()
    }
}
