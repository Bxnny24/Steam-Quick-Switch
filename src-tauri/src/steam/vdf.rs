//! Parsing Steam's `config/loginusers.vdf` (Valve KeyValues text format).
//!
//! Only accounts listed here can be switched without re-entering a password,
//! because Steam has cached their credentials.

use std::borrow::Cow;
use std::path::Path;

use keyvalues_parser::{Obj, Value};

/// A single account entry as stored in `loginusers.vdf`.
#[derive(Debug, Clone)]
pub struct VdfUser {
    pub steam_id64: String,
    pub account_name: String,
    pub persona_name: String,
    pub remember_password: bool,
    pub most_recent: bool,
    pub timestamp: u64,
}

/// Parse `<steam_path>/config/loginusers.vdf` into a list of users.
pub fn parse_login_users(steam_path: &Path) -> Result<Vec<VdfUser>, String> {
    let path = steam_path.join("config").join("loginusers.vdf");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

    let vdf = keyvalues_parser::parse(&content)
        .map_err(|e| format!("failed to parse loginusers.vdf: {e}"))?;

    let users_obj = match vdf.value {
        Value::Obj(obj) => obj,
        _ => return Err("loginusers.vdf root is not an object".into()),
    };

    let mut users = Vec::new();
    for (steam_id, values) in users_obj.iter() {
        let Some(Value::Obj(user)) = values.first() else {
            continue;
        };

        users.push(VdfUser {
            steam_id64: steam_id.to_string(),
            account_name: obj_str(user, "AccountName").unwrap_or_default().to_string(),
            persona_name: obj_str(user, "PersonaName").unwrap_or_default().to_string(),
            remember_password: obj_bool(user, "RememberPassword"),
            most_recent: obj_bool(user, "MostRecent"),
            timestamp: obj_str(user, "Timestamp")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0),
        });
    }

    Ok(users)
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

/// Read a `"0"`/`"1"` flag as a bool.
fn obj_bool(obj: &Obj, key: &str) -> bool {
    obj_str(obj, key).map(|s| s == "1").unwrap_or(false)
}

/// Mark `target_steam_id64` as the most-recently-used account in
/// `loginusers.vdf`, clearing the flag on all others. A one-time `.bak`
/// backup is written before the first modification.
pub fn set_most_recent(steam_path: &Path, target_steam_id64: &str) -> Result<(), String> {
    let path = steam_path.join("config").join("loginusers.vdf");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read loginusers.vdf: {e}"))?;

    let mut vdf = keyvalues_parser::parse(&content)
        .map_err(|e| format!("failed to parse loginusers.vdf: {e}"))?;

    if let Some(users) = vdf.value.get_mut_obj() {
        for (steam_id, values) in users.iter_mut() {
            let is_target = steam_id.as_ref() == target_steam_id64;
            if let Some(Value::Obj(user)) = values.first_mut() {
                set_str(user, "MostRecent", if is_target { "1" } else { "0" });
                if is_target {
                    // Maximise the chance Steam auto-logs in without a prompt.
                    set_str(user, "RememberPassword", "1");
                    set_str(user, "AllowAutoLogin", "1");
                }
            }
        }
    }

    let rendered = vdf.to_string();

    let backup = path.with_extension("vdf.bak");
    if !backup.exists() {
        let _ = std::fs::copy(&path, &backup);
    }
    std::fs::write(&path, rendered).map_err(|e| format!("failed to write loginusers.vdf: {e}"))?;

    Ok(())
}

/// Set (or insert) a string field on a KeyValues object, matching the key
/// case-insensitively.
fn set_str(obj: &mut Obj, key: &str, value: &str) {
    for (k, values) in obj.iter_mut() {
        if k.eq_ignore_ascii_case(key) {
            *values = vec![Value::Str(Cow::Owned(value.to_string()))];
            return;
        }
    }
    obj.insert(
        Cow::Owned(key.to_string()),
        vec![Value::Str(Cow::Owned(value.to_string()))],
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#""users"
{
    "111"
    {
        "AccountName"        "alice"
        "PersonaName"        "Alice"
        "RememberPassword"        "1"
        "MostRecent"        "1"
        "Timestamp"        "100"
    }
    "222"
    {
        "AccountName"        "bob"
        "PersonaName"        "Bob"
        "RememberPassword"        "1"
        "MostRecent"        "0"
        "Timestamp"        "200"
    }
}
"#;

    #[test]
    fn set_most_recent_flips_flags_losslessly() {
        let mut vdf = keyvalues_parser::parse(SAMPLE).unwrap();
        {
            let users = vdf.value.get_mut_obj().unwrap();
            for (steam_id, values) in users.iter_mut() {
                let flag = if steam_id.as_ref() == "222" { "1" } else { "0" };
                if let Some(Value::Obj(user)) = values.first_mut() {
                    set_str(user, "MostRecent", flag);
                }
            }
        }
        let rendered = vdf.to_string();

        let reparsed = keyvalues_parser::parse(&rendered).unwrap();
        let users = match reparsed.value {
            Value::Obj(obj) => obj,
            _ => panic!("root not an object"),
        };

        let mut seen = 0;
        for (steam_id, values) in users.iter() {
            let Some(Value::Obj(user)) = values.first() else {
                continue;
            };
            seen += 1;
            let most_recent = obj_bool(user, "MostRecent");
            let account = obj_str(user, "AccountName").unwrap_or_default();
            match steam_id.as_ref() {
                "111" => {
                    assert_eq!(account, "alice");
                    assert!(!most_recent, "alice should no longer be most recent");
                }
                "222" => {
                    assert_eq!(account, "bob");
                    assert!(most_recent, "bob should be most recent");
                }
                other => panic!("unexpected steam id {other}"),
            }
        }
        assert_eq!(seen, 2, "both accounts must survive the round-trip");
    }
}
