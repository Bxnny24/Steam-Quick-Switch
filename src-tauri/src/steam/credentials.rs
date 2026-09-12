//! Which accounts Steam still has a saved login for.
//!
//! `config/loginusers.vdf` keeps listing an account long after its credentials
//! are gone: a Steam reinstall (or a cleared credential store) rewrites that
//! file with every old entry intact, but Steam can no longer log those accounts
//! in without a password. Steam's own credential store is
//! `config/config.vdf` → `InstallConfigStore/Software/Valve/Steam/Accounts`,
//! which holds one entry per account Steam still knows on this machine.
//!
//! Comparing the two tells us which menu entries would silently drop the user
//! at the Steam login screen instead of switching.

use std::collections::HashSet;
use std::path::Path;

use keyvalues_parser::{Obj, Value};

/// The accounts `config.vdf` lists under `Accounts`, by login name and SteamID.
///
/// Both are kept because the two files spell an account differently: the
/// `Accounts` keys are login names (whose case need not match
/// `loginusers.vdf`), while the nested `SteamID` is an exact identifier.
#[derive(Debug, Clone, Default)]
pub struct CachedLogins {
    names: HashSet<String>,
    steam_ids: HashSet<String>,
}

impl CachedLogins {
    /// Whether Steam still has a saved login for this account.
    pub fn contains(&self, account_name: &str, steam_id64: &str) -> bool {
        self.steam_ids.contains(steam_id64) || self.names.contains(&account_name.to_lowercase())
    }
}

/// Read the saved-login set from `<steam_path>/config/config.vdf`.
///
/// `None` means "unknown", and callers must then treat every account as fine.
/// It is returned when the file is missing or unparsable, and also when the
/// `Accounts` section is absent or empty — a Steam build that stores its
/// credentials elsewhere would otherwise make the app flag *every* account as
/// needing a password. A false "all good" only reproduces today's behaviour;
/// a false "login required" on every entry would look broken.
pub fn cached_logins(steam_path: &Path) -> Option<CachedLogins> {
    let path = steam_path.join("config").join("config.vdf");
    let content = std::fs::read_to_string(path).ok()?;
    parse_cached_logins(&content)
}

/// The parsing half of [`cached_logins`], split out so it can be tested
/// without a Steam installation.
fn parse_cached_logins(content: &str) -> Option<CachedLogins> {
    let vdf = keyvalues_parser::parse(content).ok()?;
    let Value::Obj(root) = vdf.value else {
        return None;
    };

    // config.vdf nests the store as
    // InstallConfigStore (the parsed root) / Software / Valve / Steam / Accounts.
    let mut obj = &root;
    for key in ["Software", "Valve", "Steam", "Accounts"] {
        obj = child_obj(obj, key)?;
    }

    let mut logins = CachedLogins::default();
    for (name, values) in obj.iter() {
        logins.names.insert(name.to_lowercase());
        if let Some(Value::Obj(entry)) = values.first() {
            if let Some(id) = obj_str(entry, "SteamID") {
                logins.steam_ids.insert(id.to_string());
            }
        }
    }

    if logins.names.is_empty() {
        return None;
    }

    Some(logins)
}

/// The nested object stored under `key` (case-insensitive), if any.
fn child_obj<'a, 'input>(obj: &'a Obj<'input>, key: &str) -> Option<&'a Obj<'input>> {
    for (k, values) in obj.iter() {
        if k.eq_ignore_ascii_case(key) {
            if let Some(Value::Obj(child)) = values.first() {
                return Some(child);
            }
        }
    }
    None
}

/// Read a string field from a KeyValues object (case-insensitive on the key).
fn obj_str<'a>(obj: &'a Obj, key: &str) -> Option<&'a str> {
    for (k, values) in obj.iter() {
        if k.eq_ignore_ascii_case(key) {
            if let Some(Value::Str(s)) = values.first() {
                return Some(s.as_ref());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shaped like the real file: the store sits four levels below the root.
    const CONFIG: &str = r#"
"InstallConfigStore"
{
    "Software"
    {
        "Valve"
        {
            "Steam"
            {
                "Accounts"
                {
                    "example_login"
                    {
                        "SteamID"		"76561190000000001"
                    }
                    "second_login"
                    {
                        "SteamID"		"76561190000000002"
                    }
                }
                "CellIDServerOverride"		"87"
            }
        }
    }
}
"#;

    #[test]
    fn accounts_section_is_read() {
        let logins = parse_cached_logins(CONFIG).expect("Accounts section present");
        assert!(logins.contains("example_login", "76561190000000001"));
        assert!(logins.contains("second_login", "76561190000000002"));
    }

    #[test]
    fn accounts_missing_from_the_section_are_reported_as_uncached() {
        let logins = parse_cached_logins(CONFIG).expect("Accounts section present");
        // Still listed in loginusers.vdf after a Steam reinstall, but Steam has
        // no credentials for it any more.
        assert!(!logins.contains("stale_login", "76561190000000003"));
    }

    #[test]
    fn login_name_matches_case_insensitively() {
        let logins = parse_cached_logins(CONFIG).expect("Accounts section present");
        // The SteamID will not match, so this only passes via the name set.
        assert!(logins.contains("Example_Login", "0"));
    }

    #[test]
    fn matches_on_steam_id_when_the_login_name_differs() {
        let logins = parse_cached_logins(CONFIG).expect("Accounts section present");
        assert!(logins.contains("renamed_login", "76561190000000001"));
    }

    #[test]
    fn missing_accounts_section_is_unknown() {
        let config = r#"
"InstallConfigStore"
{
    "Software"
    {
        "Valve"
        {
            "Steam"
            {
                "CellIDServerOverride"		"87"
            }
        }
    }
}
"#;
        assert!(parse_cached_logins(config).is_none());
    }

    #[test]
    fn empty_accounts_section_is_unknown() {
        // Treated as "cannot tell" rather than "nothing is cached": flagging
        // every account would look broken on a Steam build that keeps its
        // credentials somewhere else.
        let config = r#"
"InstallConfigStore"
{
    "Software"
    {
        "Valve"
        {
            "Steam"
            {
                "Accounts"
                {
                }
            }
        }
    }
}
"#;
        assert!(parse_cached_logins(config).is_none());
    }

    #[test]
    fn unparsable_config_is_unknown() {
        assert!(parse_cached_logins("this is not KeyValues {{{").is_none());
    }
}
