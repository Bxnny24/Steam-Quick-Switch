//! Parsing Steam's `config/loginusers.vdf` (Valve KeyValues text format).
//!
//! Only accounts listed here can be switched without re-entering a password,
//! because Steam has cached their credentials.

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
