//! Minimal tray-menu translations (English / German).

pub struct Labels {
    pub settings: &'static str,
    pub language: &'static str,
    pub autostart: &'static str,
    pub rename: &'static str,
    pub quit: &'static str,
    pub active: &'static str,
    pub no_accounts: &'static str,
}

pub fn labels(lang: &str) -> Labels {
    if lang == "de" {
        Labels {
            settings: "Einstellungen",
            language: "Sprache",
            autostart: "Mit Windows starten",
            rename: "Profil umbenennen",
            quit: "Beenden",
            active: "aktiv",
            no_accounts: "Keine Konten gefunden",
        }
    } else {
        Labels {
            settings: "Settings",
            language: "Language",
            autostart: "Start with Windows",
            rename: "Rename profile",
            quit: "Quit",
            active: "active",
            no_accounts: "No accounts found",
        }
    }
}
