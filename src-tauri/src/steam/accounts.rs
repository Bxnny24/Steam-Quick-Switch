//! Aggregating Steam accounts into the model exposed to the frontend.

use serde::Serialize;

use crate::steam::{avatar, registry, vdf};

/// An account as presented to the UI. Field names are camelCase in JSON.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub steam_id64: String,
    pub account_name: String,
    pub persona_name: String,
    pub remember_password: bool,
    pub most_recent: bool,
    pub timestamp: u64,
    /// True if this account is the one Steam will auto-login (the active one).
    pub is_current: bool,
    /// Base64 PNG data URL of the cached avatar, if available.
    pub avatar: Option<String>,
}

/// List all Steam accounts known to this machine, newest first, marking the
/// account that is currently set to auto-login.
pub fn list_accounts() -> Result<Vec<Account>, String> {
    let steam_path =
        registry::steam_path().ok_or_else(|| "Steam installation not found".to_string())?;

    let current = registry::auto_login_user()
        .unwrap_or_default()
        .to_lowercase();

    let mut users = vdf::parse_login_users(&steam_path)?;
    users.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let accounts = users
        .into_iter()
        .map(|u| {
            let is_current = !current.is_empty() && u.account_name.to_lowercase() == current;
            let avatar = avatar::avatar_data_url(&steam_path, &u.steam_id64);
            Account {
                steam_id64: u.steam_id64,
                account_name: u.account_name,
                persona_name: u.persona_name,
                remember_password: u.remember_password,
                most_recent: u.most_recent,
                timestamp: u.timestamp,
                is_current,
                avatar,
            }
        })
        .collect();

    Ok(accounts)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test against the real machine. Ignored by default (needs Steam).
    /// Run with: cargo test -- --ignored --nocapture print_real_accounts
    #[test]
    #[ignore]
    fn print_real_accounts() {
        let accounts = list_accounts().expect("list_accounts failed");
        for a in &accounts {
            println!(
                "{} | account={} persona={:?} remember={} current={} ts={}",
                a.steam_id64,
                a.account_name,
                a.persona_name,
                a.remember_password,
                a.is_current,
                a.timestamp
            );
        }
        assert!(!accounts.is_empty(), "expected at least one account");
    }
}
