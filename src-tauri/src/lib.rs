mod steam;
mod tray;

/// Return the Steam accounts known to this machine, for the switcher UI.
#[tauri::command]
fn list_accounts() -> Result<Vec<steam::Account>, String> {
    steam::list_accounts()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            tray::build_tray(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_accounts])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
