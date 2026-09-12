//! System tray — the entire UI. A native menu lists all Steam accounts (with
//! avatars) plus settings, and the tray icon shows the active account's avatar.

use std::time::Duration;

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
/// How often to poll for account switches made outside this app.
const WATCH_INTERVAL: Duration = Duration::from_secs(3);
/// How often, and how many times, to confirm after startup that the tray icon
/// really reached the notification area (see `start_registration_guard`).
const REGISTRATION_CHECKS: u32 = 5;
const REGISTRATION_INTERVAL: Duration = Duration::from_secs(12);

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

/// Order accounts for the menu, per the user's sort setting.
///
/// "recent" keeps the order `list_accounts` produced — active account pinned
/// first, then most-recently-used. "name" is an explicit alphabetical choice,
/// so the active account sorts in with the rest: pinning it would move a name
/// away from the letter the user is scanning for.
fn sorted_accounts<'a>(app: &AppHandle, accounts: &'a [Account]) -> Vec<&'a Account> {
    let mut ordered: Vec<&Account> = accounts.iter().collect();
    if settings::sort_mode(app) == "name" {
        // Sort by what the menu actually shows, so the order always matches the
        // visible labels under either display-name mode. The sort is stable, so
        // accounts sharing a name keep their most-recent-first order.
        ordered.sort_by_cached_key(|a| sort_key(&display_name(app, a)));
    }
    ordered
}

/// Case-insensitive sort key. German umlauts fold to their base letter so
/// "Ärger" sorts under A instead of after Z — the app ships English and German,
/// and raw code-point order would push every umlaut to the end.
fn sort_key(name: &str) -> String {
    let mut key = String::with_capacity(name.len());
    for ch in name.to_lowercase().chars() {
        match ch {
            'ä' => key.push('a'),
            'ö' => key.push('o'),
            'ü' => key.push('u'),
            'ß' => key.push_str("ss"),
            _ => key.push(ch),
        }
    }
    key
}

/// The rounded avatar icon for an account. Falls back to Steam's own "no
/// avatar" placeholder so accounts without a profile picture still get an icon.
fn avatar_icon(
    steam_path: &std::path::Path,
    steam_id64: &str,
    size: u32,
) -> Option<Image<'static>> {
    let path = steam::avatar::avatar_path(steam_path, steam_id64)
        .or_else(|| steam::avatar::blank_avatar_path(steam_path))?;
    let (rgba, size) = steam::avatar::round_icon_rgba(&path, size)?;
    Some(Image::new_owned(rgba, size, size))
}

/// Build the full tray menu from the current accounts and settings.
fn build_menu(app: &AppHandle, accounts: &[Account]) -> tauri::Result<Menu<Wry>> {
    let lang = settings::language(app);
    let mode = settings::name_mode(app);
    let sort = settings::sort_mode(app);
    let l = i18n::labels(&lang);
    let steam_path = steam::registry::steam_path();
    let ordered = sorted_accounts(app, accounts);

    let menu = Menu::new(app)?;

    if accounts.is_empty() {
        let item = MenuItem::with_id(app, "noop", l.no_accounts, false, None::<&str>)?;
        menu.append(&item)?;
    } else {
        for &account in &ordered {
            let mut label = display_name(app, account);
            if account.is_current {
                label = format!("{label}  •  {}", l.active);
            } else if !account.has_cached_login {
                // Steam still lists this account but has no saved login for it,
                // so a switch would land on the login screen. Say so up front.
                label = format!("{label}  •  {}", l.login_required);
            }
            let icon = steam_path
                .as_deref()
                .and_then(|p| avatar_icon(p, &account.steam_id64, MENU_ICON_SIZE));
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

    // Settings submenu: language, display name, sort order, autostart.
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

    let sort_recent = CheckMenuItem::with_id(
        app,
        "sort:recent",
        l.sort_recent,
        true,
        sort == "recent",
        None::<&str>,
    )?;
    let sort_name = CheckMenuItem::with_id(
        app,
        "sort:name",
        l.sort_name,
        true,
        sort == "name",
        None::<&str>,
    )?;
    let sort_menu = SubmenuBuilder::new(app, l.sort_order)
        .item(&sort_recent)
        .item(&sort_name)
        .build()?;

    let autostart_on = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart =
        CheckMenuItem::with_id(app, "autostart", l.autostart, true, autostart_on, None::<&str>)?;

    let settings_menu = SubmenuBuilder::new(app, l.settings)
        .item(&lang_menu)
        .item(&name_menu)
        .item(&sort_menu)
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
    let accounts = steam::list_accounts().unwrap_or_default();
    if let Ok(menu) = build_menu(app, &accounts) {
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = tray.set_menu(Some(menu));
        }
    }
    refresh_icon(app, &accounts);
}

fn refresh_icon(app: &AppHandle, accounts: &[Account]) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    let Some(current) = accounts
        .iter()
        .find(|a| a.is_current)
        .or_else(|| accounts.first())
    else {
        return;
    };
    let _ = tray.set_tooltip(Some(display_name(app, current)));
    // Always replace the icon, never just skip on failure: an account without a
    // cached avatar would otherwise keep showing the previous account's picture.
    let icon = steam::registry::steam_path()
        .as_deref()
        .and_then(|p| avatar_icon(p, &current.steam_id64, TRAY_ICON_SIZE))
        .or_else(|| app.default_window_icon().cloned().map(Image::to_owned));
    if let Some(icon) = icon {
        let _ = tray.set_icon(Some(icon));
    }
}

/// Create the tray icon and menu. Split out of [`setup`] because
/// [`start_registration_guard`] may have to build the icon a second time.
fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    let accounts = steam::list_accounts().unwrap_or_default();
    let menu = build_menu(app, &accounts)?;
    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Steam Quick Switch")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| handle_menu_event(app, event.id().as_ref()))
        .build(app)?;
    refresh_icon(app, &accounts);
    Ok(())
}

/// Create the tray icon and menu on startup.
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    create_tray(app)?;
    start_account_watcher(app);
    start_registration_guard(app);
    Ok(())
}

/// Confirm the tray icon actually landed in the notification area, and rebuild
/// it if it did not.
///
/// Windows is allowed to reject `Shell_NotifyIcon(NIM_ADD)` while the shell is
/// still coming up at logon — precisely when this app starts, since it is
/// registered for autostart. `tray-icon` swallows that failure (its error
/// branch is an empty `if` block), so `TrayIconBuilder::build` still returns
/// `Ok`: the app runs on with a working menu, watcher and switching, but with
/// no icon in the tray. It only re-registers on a `TaskbarCreated` broadcast,
/// which Windows sends when the taskbar is *created* — so if Explorer was
/// already running that broadcast is in the past and never arrives again, and
/// the icon stays missing for the whole process lifetime.
///
/// `rect()` is backed by `Shell_NotifyIconGetRect`, which fails for an icon the
/// shell does not know about. It can also fail for a merely hidden icon on some
/// Windows versions, so rebuilding is not free of false positives: it can cost
/// the user’s “always show this icon” placement. Hence the hard attempt
/// limit and the stop on first success — at worst a couple of re-adds in the
/// first minute, never a loop.
fn start_registration_guard(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        for _ in 0..REGISTRATION_CHECKS {
            std::thread::sleep(REGISTRATION_INTERVAL);
            let registered = app
                .tray_by_id(TRAY_ID)
                .and_then(|tray| tray.rect().ok().flatten())
                .is_some();
            if registered {
                return;
            }
            // Building a tray icon creates a window, so it belongs on the main
            // thread — the same reason `refresh` is dispatched there.
            let handle = app.clone();
            let _ = app.run_on_main_thread(move || {
                // Dropping the removed icon is what unregisters it, so the
                // stale entry cannot survive alongside the new one.
                let _ = handle.remove_tray_by_id(TRAY_ID);
                let _ = create_tray(&handle);
            });
        }
    });
}

/// The lowercased active-account key, used to detect external switches.
fn current_account_key() -> String {
    steam::registry::auto_login_user()
        .unwrap_or_default()
        .to_lowercase()
}

/// Watch for account switches made outside this app (Steam itself or other
/// tools) and refresh the tray whenever the active account changes.
fn start_account_watcher(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || {
        let mut last = current_account_key();
        loop {
            std::thread::sleep(WATCH_INTERVAL);
            let now = current_account_key();
            if now != last {
                last = now;
                let handle = app.clone();
                let _ = app.run_on_main_thread(move || refresh(&handle));
            }
        }
    });
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
    } else if id == "sort:recent" {
        settings::set_sort_mode(app, "recent");
        refresh(app);
    } else if id == "sort:name" {
        settings::set_sort_mode(app, "name");
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
        let l = i18n::labels(&settings::language(&app));

        // Steam has no saved login for this account, so the switch would close
        // the running game and then stop at the Steam login screen. Ask first
        // rather than springing that on the user. The dialog blocks this worker
        // thread only, so the tray stays responsive while it is open.
        if !account.has_cached_login && !confirm(l.login_required_title, &login_prompt(&account, &l))
        {
            return;
        }

        let result = match steam::registry::steam_path() {
            Some(steam_path) => steam::switch::switch_account(&steam_path, &account.account_name),
            None => Err("Steam installation not found.".to_string()),
        };
        // Never fail silently: switching is the app's primary action, so surface
        // any error in a native dialog instead of leaving the user guessing.
        if let Err(message) = result {
            show_error(l.switch_failed, &message);
        }
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || refresh(&handle));
    });
}

/// The body of the "no saved login" confirmation: which account is affected,
/// then the localized explanation. The account name is named explicitly because
/// the dialog is ownerless and carries no other context.
fn login_prompt(account: &Account, l: &i18n::Labels) -> String {
    let name = if account.persona_name.trim().is_empty() {
        account.account_name.clone()
    } else {
        format!("{} ({})", account.persona_name, account.account_name)
    };
    format!("{name}\n\n{}", l.login_required_prompt)
}

/// Show a native modal error dialog so account-switch failures are never silent.
fn show_error(title: &str, message: &str) {
    const MB_OK: u32 = 0x0000_0000;
    const MB_ICONERROR: u32 = 0x0000_0010;
    message_box(title, message, MB_OK | MB_ICONERROR);
}

/// Ask a yes/no question in a native modal dialog; `true` means the user
/// confirmed. Used before a switch Steam cannot complete without a password.
fn confirm(title: &str, message: &str) -> bool {
    const MB_YESNO: u32 = 0x0000_0004;
    const MB_ICONWARNING: u32 = 0x0000_0030;
    const IDYES: i32 = 6;
    message_box(title, message, MB_YESNO | MB_ICONWARNING) == IDYES
}

/// `MessageBoxW` wrapper returning the raw dialog result. Windows-only,
/// matching the rest of the app (no extra dependency).
fn message_box(title: &str, message: &str, style: u32) -> i32 {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(
            hwnd: *mut core::ffi::c_void,
            text: *const u16,
            caption: *const u16,
            u_type: u32,
        ) -> i32;
    }

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    const MB_SETFOREGROUND: u32 = 0x0001_0000;

    let text = wide(message);
    let caption = wide(title);
    // SAFETY: `text` and `caption` are valid NUL-terminated UTF-16 buffers that
    // live until the call returns; a null owner shows an ownerless modal, which
    // is what a tray-only app needs.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            caption.as_ptr(),
            style | MB_SETFOREGROUND,
        )
    }
}
