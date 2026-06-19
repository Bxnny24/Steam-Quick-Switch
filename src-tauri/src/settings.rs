//! Persistent settings, stored in `settings.json` (via the store plugin) in
//! the app data directory. Read and written from Rust so the app needs no
//! window.

use serde_json::json;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE: &str = "settings.json";

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

/// The custom nickname for an account, if any.
pub fn nickname(app: &AppHandle, steam_id64: &str) -> Option<String> {
    let store = app.store(STORE).ok()?;
    let value = store.get("nicknames")?;
    let obj = value.as_object()?;
    obj.get(steam_id64)?.as_str().map(|s| s.to_string())
}

/// Set or clear (empty string) the nickname for an account.
pub fn set_nickname(app: &AppHandle, steam_id64: &str, nickname: &str) {
    if let Ok(store) = app.store(STORE) {
        let mut map = store
            .get("nicknames")
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default();
        let trimmed = nickname.trim();
        if trimmed.is_empty() {
            map.remove(steam_id64);
        } else {
            map.insert(steam_id64.to_string(), json!(trimmed));
        }
        store.set("nicknames", serde_json::Value::Object(map));
        let _ = store.save();
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
