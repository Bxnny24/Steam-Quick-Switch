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
    pub login_required: &'static str,
    pub login_required_title: &'static str,
    pub login_required_prompt: &'static str,
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
            login_required: "Anmeldung nötig",
            login_required_title: "Kein gespeicherter Steam-Login",
            login_required_prompt: concat!(
                "Steam listet dieses Konto noch, hat aber keine gespeicherten ",
                "Anmeldedaten dafür (z. B. nach einer Steam-Neuinstallation).\n\n",
                "Beim Wechsel fragt Steam nach Passwort und Steam Guard. Danach ist ",
                "das Konto wieder ohne Nachfrage wechselbar.\n\nTrotzdem wechseln?"
            ),
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
            login_required: "login required",
            login_required_title: "No saved Steam login",
            login_required_prompt: concat!(
                "Steam still lists this account but has no saved credentials for ",
                "it (typically after a Steam reinstall).\n\n",
                "Switching will make Steam ask for the password and Steam Guard. ",
                "After that the account switches without a prompt again.",
                "\n\nSwitch anyway?"
            ),
        }
    }
}
