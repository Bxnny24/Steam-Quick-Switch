mod i18n;
mod settings;
mod steam;
mod tray;

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_updater::UpdaterExt;

/// Save (or clear) a nickname, rebuild the tray, and close the popup.
#[tauri::command]
fn set_nickname(app: tauri::AppHandle, steam_id64: String, nickname: String) {
    settings::set_nickname(&app, &steam_id64, &nickname);
    tray::refresh(&app);
    if let Some(win) = app.get_webview_window("nickname") {
        let _ = win.close();
    }
}

/// Close the nickname popup without saving.
#[tauri::command]
fn close_nickname(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("nickname") {
        let _ = win.close();
    }
}

/// Open the small nickname popup window for an account.
pub fn open_nickname_popup(app: &tauri::AppHandle, steam_id64: &str) {
    if let Some(win) = app.get_webview_window("nickname") {
        let _ = win.set_focus();
        return;
    }
    let accounts = steam::list_accounts().unwrap_or_default();
    let Some(account) = accounts.iter().find(|a| a.steam_id64 == steam_id64) else {
        return;
    };
    let name = if account.persona_name.trim().is_empty() {
        account.account_name.clone()
    } else {
        account.persona_name.clone()
    };
    let lang = settings::language(app);
    let current = settings::nickname(app, steam_id64).unwrap_or_default();
    let url = format!(
        "index.html?steamid={}&name={}&nick={}&lang={}",
        encode(steam_id64),
        encode(&name),
        encode(&current),
        lang
    );
    let _ = WebviewWindowBuilder::new(app, "nickname", WebviewUrl::App(url.into()))
        .title("Nickname")
        .inner_size(320.0, 150.0)
        .resizable(false)
        .always_on_top(true)
        .center()
        .build();
}

/// Minimal percent-encoding for query-string values.
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(win) = app.get_webview_window("nickname") {
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            tray::setup(app.handle())?;
            // Background auto-update check.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = try_update(handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![set_nickname, close_nickname])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // Keep running in the tray when the (popup) window closes; only a
            // real exit request (the Quit menu calling app.exit) stops the app.
            if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}

async fn try_update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        update.download_and_install(|_, _| {}, || {}).await?;
        app.restart();
    }
    Ok(())
}
