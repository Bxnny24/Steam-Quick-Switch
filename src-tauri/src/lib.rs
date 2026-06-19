mod steam;
mod tray;

/// Return the Steam accounts known to this machine, for the switcher UI.
#[tauri::command]
fn list_accounts() -> Result<Vec<steam::Account>, String> {
    steam::list_accounts()
}

/// Switch the active Steam account (closes and relaunches Steam), then refresh
/// the tray icon. The blocking work runs off the async runtime.
#[tauri::command]
async fn switch_account(
    app: tauri::AppHandle,
    account_name: String,
    steam_id64: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let steam_path = steam::registry::steam_path()
            .ok_or_else(|| "Steam installation not found".to_string())?;
        steam::switch::switch_account(&steam_path, &account_name, &steam_id64)
    })
    .await
    .map_err(|e| format!("switch task panicked: {e}"))??;

    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        tray::refresh_tray_icon(&handle);
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            tray::build_tray(app.handle())?;
            tray::setup_close_to_tray(app.handle());
            // Show the window on a normal launch; stay hidden when autostarted.
            if !std::env::args().any(|arg| arg == "--minimized") {
                tray::show_main_window(app.handle());
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_accounts, switch_account])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
