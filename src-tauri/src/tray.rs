//! System tray: dynamic avatar icon, tooltip, and context menu.

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

use crate::steam::{self, Account};

pub const TRAY_ID: &str = "main-tray";
const ICON_SIZE: u32 = 32;

/// Bring the main window to the foreground (used by the tray and single-instance).
pub fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Hide the window to the tray instead of quitting when the user closes it,
/// so the app keeps running in the background.
pub fn setup_close_to_tray(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let win = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = win.hide();
            }
        });
    }
}

/// Build the tray icon with a context menu, then apply the current avatar.
/// Left click opens the switcher window, right click shows the menu.
pub fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Steam Quick Switch")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    refresh_tray_icon(app);
    Ok(())
}

/// Update the tray icon + tooltip to reflect the currently active account.
/// Safe to call repeatedly (e.g. after a switch).
pub fn refresh_tray_icon(app: &tauri::AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };

    let accounts = match steam::list_accounts() {
        Ok(accounts) => accounts,
        Err(_) => return,
    };

    let Some(current) = accounts
        .iter()
        .find(|a| a.is_current)
        .or_else(|| accounts.first())
    else {
        return;
    };

    let _ = tray.set_tooltip(Some(format!(
        "Steam Quick Switch — {}",
        display_name(current)
    )));

    if let Some(steam_path) = steam::registry::steam_path() {
        if let Some(path) = steam::avatar::avatar_path(&steam_path, &current.steam_id64) {
            if let Some((rgba, size)) = steam::avatar::round_icon_rgba(&path, ICON_SIZE) {
                let _ = tray.set_icon(Some(Image::new_owned(rgba, size, size)));
            }
        }
    }
}

/// Best display name available without user settings (refined in settings later).
fn display_name(account: &Account) -> String {
    if account.persona_name.trim().is_empty() {
        account.account_name.clone()
    } else {
        account.persona_name.clone()
    }
}
