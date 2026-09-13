//! Reading and editing Steam's `config/loginusers.vdf` (Valve KeyValues text
//! format) — the saved-account list behind both Steam's own account picker and
//! this app's menu. Whether an account can still sign in silently is a separate
//! question, answered by `credentials.rs`.
//!
//! How Steam treats edits to this file, verified by hand on a live install:
//! - Steam reads it only at startup. Its account picker keeps showing a removed
//!   account until Steam restarts, and while Steam is still running, switching
//!   to such an account works from memory.
//! - A normal Steam exit does not write a removed entry back, so a removal
//!   survives Steam restarts.
//! - Signing in to an account writes its entry again. That is how accounts get
//!   into the file in the first place, not a failed removal.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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

/// Path of `loginusers.vdf` inside a Steam installation.
pub fn login_users_path(steam_path: &Path) -> PathBuf {
    steam_path.join("config").join("loginusers.vdf")
}

/// Parse `<steam_path>/config/loginusers.vdf` into a list of users.
pub fn parse_login_users(steam_path: &Path) -> Result<Vec<VdfUser>, String> {
    let path = login_users_path(steam_path);
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

/// Remove one account from `loginusers.vdf`. An account that is not listed is
/// not an error; nothing is written then.
///
/// Only the account's own block is cut out. Every other byte — entry order,
/// tabs, line endings — stays exactly as Steam wrote it, which is precisely the
/// edit verified against a running Steam (see the module docs). Re-rendering
/// through `keyvalues_parser` is avoided on purpose: its `Obj` is a `BTreeMap`,
/// so it would reorder the entries and reformat the file into a shape Steam
/// never produced.
///
/// The previous content is kept as `loginusers.vdf.sqs-backup`, and the new
/// content lands via a temporary file plus rename, so a crash mid-write cannot
/// leave Steam with a truncated account list.
pub fn remove_login_user(steam_path: &Path, steam_id64: &str) -> Result<(), String> {
    let path = login_users_path(steam_path);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

    let Some(updated) = remove_user_entry(&content, steam_id64)? else {
        return Ok(());
    };

    let backup = path.with_extension("vdf.sqs-backup");
    std::fs::write(&backup, &content)
        .map_err(|e| format!("failed to back up {}: {e}", path.display()))?;

    let temp = path.with_extension("vdf.sqs-tmp");
    std::fs::write(&temp, &updated)
        .map_err(|e| format!("failed to write {}: {e}", temp.display()))?;
    // On Windows `rename` replaces an existing target (MoveFileExW with
    // MOVEFILE_REPLACE_EXISTING), so the file is swapped in one step.
    if let Err(e) = std::fs::rename(&temp, &path) {
        let _ = std::fs::remove_file(&temp);
        return Err(format!("failed to replace {}: {e}", path.display()));
    }

    Ok(())
}

/// Cut the top-level entry `"<steam_id64>" { … }` out of `loginusers.vdf`
/// text. `Ok(None)` when the file lists no such account.
///
/// The result is parsed back before it is returned: it must still be valid
/// KeyValues and list exactly the old accounts minus this one. Anything else is
/// an error, so a scanner mistake can never reach Steam's file.
fn remove_user_entry(content: &str, steam_id64: &str) -> Result<Option<String>, String> {
    let before = top_level_keys(content)?;
    if !before.contains(steam_id64) {
        return Ok(None);
    }

    let (start, end) = find_entry_span(content, steam_id64)
        .ok_or_else(|| format!("could not locate account {steam_id64} in loginusers.vdf"))?;
    let updated = format!("{}{}", &content[..start], &content[end..]);

    let mut expected = before;
    expected.remove(steam_id64);
    if top_level_keys(&updated)? != expected {
        return Err("removing the account would corrupt loginusers.vdf".into());
    }

    Ok(Some(updated))
}

/// The keys directly under the root object — one SteamID64 per saved account.
fn top_level_keys(content: &str) -> Result<BTreeSet<String>, String> {
    let vdf = keyvalues_parser::parse(content)
        .map_err(|e| format!("failed to parse loginusers.vdf: {e}"))?;
    let Value::Obj(users) = vdf.value else {
        return Err("loginusers.vdf root is not an object".into());
    };
    Ok(users.keys().map(|key| key.to_string()).collect())
}

/// Byte span of the top-level entry `"<key>" { … }`, widened to whole lines
/// when the entry sits on lines of its own.
///
/// A small scanner rather than the parser, because the parser does not report
/// positions. It tracks brace depth and skips quoted strings honouring `\`
/// escapes, so a persona name such as `"a\"}{"` cannot throw the depth off, and
/// `//` comments. Only a quoted key at depth 1 (directly inside `"users"`) that
/// is followed by an object counts, so an equal string nested deeper, or used
/// as a value, never matches.
fn find_entry_span(content: &str, key: &str) -> Option<(usize, usize)> {
    let bytes = content.as_bytes();
    let mut depth = 0usize;
    // Start of the last depth-1 string, if it equals `key`: the entry begins
    // there if an object opens next.
    let mut candidate: Option<usize> = None;
    let mut entry_start: Option<usize> = None;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                let token_start = i;
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    // `\"` is an escaped quote inside the string, not its end.
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                // Quotes are ASCII, so both bounds sit on char boundaries.
                let text = &content[token_start + 1..i.min(bytes.len())];
                if depth == 1 && entry_start.is_none() {
                    candidate = (text == key).then_some(token_start);
                }
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'{' => {
                if depth == 1 && entry_start.is_none() {
                    entry_start = candidate.take();
                }
                depth += 1;
            }
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 1 {
                    if let Some(start) = entry_start {
                        return Some(widen_to_lines(bytes, start, i + 1));
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

/// Grow `[start, end)` to whole lines — the key's indentation through the line
/// break after the closing brace — so the removal leaves no blank line behind.
/// If the entry shares a line with anything else, the exact span is kept.
fn widen_to_lines(bytes: &[u8], start: usize, end: usize) -> (usize, usize) {
    let mut line_start = start;
    while line_start > 0 && matches!(bytes[line_start - 1], b' ' | b'\t') {
        line_start -= 1;
    }
    if line_start > 0 && bytes[line_start - 1] != b'\n' {
        return (start, end);
    }

    let mut line_end = end;
    while line_end < bytes.len() && matches!(bytes[line_end], b' ' | b'\t') {
        line_end += 1;
    }
    match bytes.get(line_end) {
        Some(b'\r') if bytes.get(line_end + 1) == Some(&b'\n') => (line_start, line_end + 2),
        Some(b'\n') => (line_start, line_end + 1),
        None => (line_start, line_end),
        Some(_) => (start, end),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: &str = "76561190000000001";
    const SECOND: &str = "76561190000000002";
    const THIRD: &str = "76561190000000003";

    /// One account block, shaped exactly like Steam writes it (tab-indented).
    /// `persona` is inserted verbatim, so escapes can be passed through as-is.
    fn block(steam_id: &str, login: &str, persona: &str, newline: &str) -> String {
        [
            format!("\t\"{steam_id}\""),
            "\t{".to_string(),
            format!("\t\t\"AccountName\"\t\t\"{login}\""),
            format!("\t\t\"PersonaName\"\t\t\"{persona}\""),
            "\t\t\"RememberPassword\"\t\t\"1\"".to_string(),
            "\t\t\"Timestamp\"\t\t\"1780000000\"".to_string(),
            "\t}".to_string(),
        ]
        .iter()
        .map(|line| format!("{line}{newline}"))
        .collect()
    }

    /// A whole `loginusers.vdf` wrapping the given blocks, in order.
    fn users(blocks: &[String], newline: &str) -> String {
        format!("\"users\"{newline}{{{newline}{}}}{newline}", blocks.concat())
    }

    fn lf_blocks() -> [String; 3] {
        [
            block(FIRST, "first_login", "First", "\n"),
            block(SECOND, "second_login", "Second", "\n"),
            block(THIRD, "third_login", "Third", "\n"),
        ]
    }

    #[test]
    fn removes_only_the_target_entry_byte_for_byte() {
        let [a, b, c] = lf_blocks();
        let content = users(&[a.clone(), b, c.clone()], "\n");
        let updated = remove_user_entry(&content, SECOND).unwrap().unwrap();
        assert_eq!(updated, users(&[a, c], "\n"));
    }

    #[test]
    fn removes_the_first_and_the_last_entry() {
        let [a, b, c] = lf_blocks();
        let content = users(&[a.clone(), b.clone(), c.clone()], "\n");
        assert_eq!(
            remove_user_entry(&content, FIRST).unwrap().unwrap(),
            users(&[b.clone(), c], "\n")
        );
        assert_eq!(
            remove_user_entry(&content, THIRD).unwrap().unwrap(),
            users(&[a, b], "\n")
        );
    }

    #[test]
    fn removing_the_only_entry_leaves_an_empty_list() {
        let [a, _, _] = lf_blocks();
        let content = users(&[a], "\n");
        let updated = remove_user_entry(&content, FIRST).unwrap().unwrap();
        assert_eq!(updated, users(&[], "\n"));
        assert!(top_level_keys(&updated).unwrap().is_empty());
    }

    #[test]
    fn unlisted_account_is_not_an_edit() {
        let content = users(&lf_blocks(), "\n");
        assert_eq!(remove_user_entry(&content, "76561190000000009").unwrap(), None);
    }

    #[test]
    fn escaped_quotes_and_braces_in_names_do_not_confuse_the_scanner() {
        // Persona names are free text; an escaped quote followed by braces would
        // shift the brace depth of a naive scanner and cut the wrong span.
        let tricky = block(FIRST, "first_login", r#"a\"}{\"b {c}"#, "\n");
        let [_, b, c] = lf_blocks();
        let content = users(&[tricky.clone(), b.clone(), c.clone()], "\n");

        assert_eq!(
            remove_user_entry(&content, SECOND).unwrap().unwrap(),
            users(&[tricky, c.clone()], "\n")
        );
        assert_eq!(
            remove_user_entry(&content, FIRST).unwrap().unwrap(),
            users(&[b, c], "\n")
        );
    }

    #[test]
    fn a_nested_string_equal_to_the_id_is_never_cut() {
        // The id appears as a value inside another account, but not as a
        // top-level key: nothing may be removed.
        let decoy = block(FIRST, SECOND, "Decoy", "\n");
        let content = users(&[decoy], "\n");
        assert_eq!(remove_user_entry(&content, SECOND).unwrap(), None);
    }

    #[test]
    fn crlf_line_endings_are_preserved() {
        let a = block(FIRST, "first_login", "First", "\r\n");
        let b = block(SECOND, "second_login", "Second", "\r\n");
        let c = block(THIRD, "third_login", "Third", "\r\n");
        let content = users(&[a.clone(), b, c.clone()], "\r\n");
        let updated = remove_user_entry(&content, SECOND).unwrap().unwrap();
        assert_eq!(updated, users(&[a, c], "\r\n"));
    }

    #[test]
    fn unparsable_file_is_an_error_and_never_edited() {
        assert!(remove_user_entry("this is not KeyValues {{{", FIRST).is_err());
    }
}
