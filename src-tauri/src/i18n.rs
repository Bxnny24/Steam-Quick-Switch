//! Minimal tray-menu translations (English / German).

pub struct Labels {
    pub settings: &'static str,
    pub language: &'static str,
    pub autostart: &'static str,
    pub display_name: &'static str,
    pub name_persona: &'static str,
    pub name_account: &'static str,
    pub sort_order: &'static str,
    pub sort_recent: &'static str,
    pub sort_name: &'static str,
    pub quit: &'static str,
    pub active: &'static str,
    pub no_accounts: &'static str,
    pub switch_failed: &'static str,
    pub remove_account: &'static str,
    pub remove_none: &'static str,
    pub remove_failed: &'static str,
}

pub fn labels(lang: &str) -> Labels {
    if lang == "de" {
        Labels {
            settings: "Einstellungen",
            language: "Sprache",
            autostart: "Mit Windows starten",
            display_name: "Angezeigter Name",
            name_persona: "Profilname",
            name_account: "Kontoname",
            sort_order: "Sortierung",
            sort_recent: "Zuletzt benutzt",
            sort_name: "Name (A–Z)",
            quit: "Beenden",
            active: "aktiv",
            no_accounts: "Keine Konten gefunden",
            switch_failed: "Konto konnte nicht gewechselt werden",
            remove_account: "Konto entfernen",
            remove_none: "Keine weiteren Konten",
            remove_failed: "Konto konnte nicht entfernt werden",
        }
    } else {
        Labels {
            settings: "Settings",
            language: "Language",
            autostart: "Start with Windows",
            display_name: "Display name",
            name_persona: "Profile name",
            name_account: "Account name",
            sort_order: "Sort order",
            sort_recent: "Last used",
            sort_name: "Name (A–Z)",
            quit: "Quit",
            active: "active",
            no_accounts: "No accounts found",
            switch_failed: "Couldn't switch account",
            remove_account: "Remove account",
            remove_none: "No other accounts",
            remove_failed: "Couldn't remove account",
        }
    }
}
