//! System tray — the entire UI. A native menu lists all Steam accounts (with
//! avatars) plus settings, and the tray icon shows the active account's avatar.

use tauri::{
    image::Image,
    menu::{CheckMenuItem, IconMenuItem, Menu, MenuItem, PredefinedMenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    AppHandle, Wry,
};
use tauri_plugin_autostart::ManagerExt;

use crate::steam::{self, Account};
use crate::{i18n, settings};

pub const TRAY_ID: &str = "main-tray";
const TRAY_ICON_SIZE: u32 = 32;
const MENU_ICON_SIZE: u32 = 18;

/// Display name: Steam profile name or account name, per the user's setting.
fn display_name(app: &AppHandle, account: &Account) -> String {
    if settings::name_mode(app) == "account" {
        return account.account_name.clone();
    }
    if account.persona_name.trim().is_empty() {
        account.account_name.clone()
    } else {
        account.persona_name.clone()
    }
}

fn menu_icon(steam_path: &std::path::Path, steam_id64: &str) -> Option<Image<'static>> {
    let path = steam::avatar::avatar_path(steam_path, steam_id64)?;
    let (rgba, size) = steam::avatar::round_icon_rgba(&path, MENU_ICON_SIZE)?;
    Some(Image::new_owned(rgba, size, size))
}

/// Build the full tray menu from the current accounts and settings.
fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let lang = settings::language(app);
    let mode = settings::name_mode(app);
    let l = i18n::labels(&lang);
    let steam_path = steam::registry::steam_path();
    let accounts = steam::list_accounts().unwrap_or_default();

    let menu = Menu::new(app)?;

    if accounts.is_empty() {
        let item = MenuItem::with_id(app, "noop", l.no_accounts, false, None::<&str>)?;
        menu.append(&item)?;
    } else {
        for account in &accounts {
            let mut label = display_name(app, account);
            if account.is_current {
                label = format!("{label}  •  {}", l.active);
            }
            let icon = steam_path
                .as_deref()
                .and_then(|p| menu_icon(p, &account.steam_id64));
            let item = IconMenuItem::with_id(
                app,
                format!("switch:{}", account.steam_id64),
                label.as_str(),
                !account.is_current,
                icon,
                None::<&str>,
            )?;
            menu.append(&item)?;
        }
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Settings submenu: language, display name, autostart.
    let lang_en =
        CheckMenuItem::with_id(app, "lang:en", "English", true, lang == "en", None::<&str>)?;
    let lang_de =
        CheckMenuItem::with_id(app, "lang:de", "Deutsch", true, lang == "de", None::<&str>)?;
    let lang_menu = SubmenuBuilder::new(app, l.language)
        .item(&lang_en)
        .item(&lang_de)
        .build()?;

    let name_persona = CheckMenuItem::with_id(
        app,
        "name:persona",
        l.name_persona,
        true,
        mode == "persona",
        None::<&str>,
    )?;
    let name_account = CheckMenuItem::with_id(
        app,
        "name:account",
        l.name_account,
        true,
        mode == "account",
        None::<&str>,
    )?;
    let name_menu = SubmenuBuilder::new(app, l.display_name)
        .item(&name_persona)
        .item(&name_account)
        .build()?;

    let autostart_on = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart =
        CheckMenuItem::with_id(app, "autostart", l.autostart, true, autostart_on, None::<&str>)?;

    let settings_menu = SubmenuBuilder::new(app, l.settings)
        .item(&lang_menu)
        .item(&name_menu)
        .item(&autostart)
        .build()?;
    menu.append(&settings_menu)?;

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    let quit = MenuItem::with_id(app, "quit", l.quit, true, None::<&str>)?;
    menu.append(&quit)?;

    Ok(menu)
}

/// Rebuild and apply the tray menu and icon. Safe to call repeatedly.
pub fn refresh(app: &AppHandle) {
    if let Ok(menu) = build_menu(app) {
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = tray.set_menu(Some(menu));
        }
    }
    refresh_icon(app);
}

fn refresh_icon(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let accounts = steam::list_accounts().unwrap_or_default();
    let Some(current) = accounts
        .iter()
        .find(|a| a.is_current)
        .or_else(|| accounts.first())
    else {
        return;
    };
    let _ = tray.set_tooltip(Some(display_name(app, current)));
    if let Some(steam_path) = steam::registry::steam_path() {
        if let Some(path) = steam::avatar::avatar_path(&steam_path, &current.steam_id64) {
            if let Some((rgba, size)) = steam::avatar::round_icon_rgba(&path, TRAY_ICON_SIZE) {
                let _ = tray.set_icon(Some(Image::new_owned(rgba, size, size)));
            }
        }
    }
}

/// Create the tray icon and menu on startup.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Steam Quick Switch")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| handle_menu_event(app, event.id().as_ref()))
        .build(app)?;
    refresh_icon(app);
    Ok(())
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    if let Some(steam_id64) = id.strip_prefix("switch:") {
        switch_to(app, steam_id64.to_string());
    } else if id == "lang:en" {
        settings::set_language(app, "en");
        refresh(app);
    } else if id == "lang:de" {
        settings::set_language(app, "de");
        refresh(app);
    } else if id == "name:persona" {
        settings::set_name_mode(app, "persona");
        refresh(app);
    } else if id == "name:account" {
        settings::set_name_mode(app, "account");
        refresh(app);
    } else if id == "autostart" {
        let manager = app.autolaunch();
        let result = if manager.is_enabled().unwrap_or(false) {
            manager.disable()
        } else {
            manager.enable()
        };
        let _ = result;
        refresh(app);
    } else if id == "quit" {
        app.exit(0);
    }
}

/// Perform an account switch off the main thread, then refresh the tray.
fn switch_to(app: &AppHandle, steam_id64: String) {
    let accounts = steam::list_accounts().unwrap_or_default();
    let Some(account) = accounts.into_iter().find(|a| a.steam_id64 == steam_id64) else {
        return;
    };
    let app = app.clone();
    std::thread::spawn(move || {
        if let Some(steam_path) = steam::registry::steam_path() {
            let _ = steam::switch::switch_account(
                &steam_path,
                &account.account_name,
                &account.steam_id64,
            );
        }
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || refresh(&handle));
    });
}
