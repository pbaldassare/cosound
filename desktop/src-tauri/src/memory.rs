//! Tool `memory.facts`: the Music Memory store (`memory_facts`, DATABASE.md §2.5).
//! It is the only memory COSOUND keeps; the Adaptive Workflow agent uses category "workflow".
//! Local only. The user can list, edit, delete and clear it; every change is in the operation log.
use crate::db::{log_op, new_id};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Serialize, Clone, Debug)]
pub struct Fact {
    pub id: String,
    pub category: String,
    pub key: String,
    pub value: Value,
    pub origin: String,
    pub scope: String,
    pub confidence: f64,
    pub source_ref: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

const COLS: &str = "id, category, key, value, origin, scope, confidence, source_ref, created_at, updated_at";

fn row(r: &rusqlite::Row) -> rusqlite::Result<Fact> {
    let v: String = r.get(3)?;
    Ok(Fact {
        id: r.get(0)?,
        category: r.get(1)?,
        key: r.get(2)?,
        value: serde_json::from_str(&v).unwrap_or(Value::Null),
        origin: r.get(4)?,
        scope: r.get(5)?,
        confidence: r.get(6)?,
        source_ref: r.get(7)?,
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
    })
}

/// Reason a text must not be stored (passwords, codes, card numbers, keys), if any.
pub fn looks_private(text: &str) -> Option<&'static str> {
    let l = text.to_lowercase();
    const WORDS: [&str; 14] = ["password", "passwd", "passcode", "pwd", "otp", "one-time code", "iban", "card number", "cvv", "api key", "apikey", "secret", "token", "pin code"];
    if WORDS.iter().any(|w| l.contains(w)) {
        return Some("looks like a password, code or credential");
    }
    // 13+ digits in a row, spaces or dashes allowed between them (card, IBAN, account numbers)
    let (mut run, mut gap) = (0, false);
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            run += 1;
            gap = false;
            if run >= 13 {
                return Some("looks like a card or account number");
            }
        } else if (ch == ' ' || ch == '-') && run > 0 && !gap {
            gap = true;
        } else {
            run = 0;
            gap = false;
        }
    }
    if text.split_whitespace().any(|w| w.len() >= 24 && w.chars().any(|c| c.is_ascii_digit()) && w.chars().any(|c| c.is_ascii_alphabetic())) {
        return Some("contains a long random-looking string (key or token)");
    }
    None
}

pub fn add(c: &Connection, category: &str, key: &str, value: &Value, origin: &str, scope: &str, confidence: f64, source_ref: Option<&str>) -> Result<Fact, String> {
    if let Some(why) = looks_private(&format!("{key} {value}")) {
        return Err(format!("not stored: {why}"));
    }
    let id = new_id();
    c.execute(
        "insert into memory_facts(id, category, key, value, origin, scope, confidence, source_ref) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, category, key, value.to_string(), origin, scope, confidence, source_ref],
    )
    .map_err(|e| e.to_string())?;
    log_op(c, None, "memory.add", "low", &json!({"category": category, "key": key}), None).map_err(|e| e.to_string())?;
    get(c, &id)?.ok_or_else(|| "fact not stored".into())
}

pub fn get(c: &Connection, id: &str) -> Result<Option<Fact>, String> {
    c.query_row(&format!("select {COLS} from memory_facts where id = ?1"), [id], row).optional().map_err(|e| e.to_string())
}

pub fn list(c: &Connection, category: Option<&str>) -> Result<Vec<Fact>, String> {
    let mut st = c.prepare(&format!("select {COLS} from memory_facts where (?1 is null or category = ?1) order by updated_at desc, key")).map_err(|e| e.to_string())?;
    let rows = st.query_map(params![category], row).map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

pub fn update(c: &Connection, id: &str, key: Option<&str>, value: Option<&Value>, scope: Option<&str>) -> Result<Fact, String> {
    let old = get(c, id)?.ok_or("memory item not found")?;
    let (k, v) = (key.unwrap_or(&old.key), value.unwrap_or(&old.value));
    if let Some(why) = looks_private(&format!("{k} {v}")) {
        return Err(format!("not stored: {why}"));
    }
    c.execute(
        "update memory_facts set key = ?2, value = ?3, scope = ?4, updated_at = datetime('now') where id = ?1",
        params![id, k, v.to_string(), scope.unwrap_or(&old.scope)],
    )
    .map_err(|e| e.to_string())?;
    log_op(c, None, "memory.update", "low", &json!({"category": old.category, "key": k}), None).map_err(|e| e.to_string())?;
    get(c, id)?.ok_or_else(|| "memory item not found".into())
}

pub fn delete(c: &Connection, id: &str) -> Result<(), String> {
    let old = get(c, id)?.ok_or("memory item not found")?;
    c.execute("delete from memory_facts where id = ?1", [id]).map_err(|e| e.to_string())?;
    log_op(c, None, "memory.delete", "low", &json!({"category": old.category, "key": old.key}), None).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear(c: &Connection, category: &str) -> Result<usize, String> {
    let n = c.execute("delete from memory_facts where category = ?1", [category]).map_err(|e| e.to_string())?;
    log_op(c, None, "memory.clear", "high", &json!({"category": category, "count": n}), None).map_err(|e| e.to_string())?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_and_privacy_filter() {
        let c = crate::db::open_memory();
        let f = add(&c, "workflow", "Logic: bounce in place", &json!({"kind": "shortcut", "keys": "Ctrl+B"}), "explicit", "custom", 1.0, None).unwrap();
        assert_eq!(list(&c, Some("workflow")).unwrap().len(), 1);
        let u = update(&c, &f.id, None, Some(&json!({"kind": "shortcut", "keys": "Ctrl+Alt+B"})), None).unwrap();
        assert_eq!(u.value["keys"], "Ctrl+Alt+B");
        for bad in ["my password is hunter2", "card 4111 1111 1111 1111", "IT60-X054-2811-1010-0000-0123-456", "key Zq8Lm3Rt7Xv2Np5Wd9Hc4Jb6Fs1Gk0Ty"] {
            assert!(add(&c, "workflow", bad, &json!({}), "explicit", "custom", 1.0, None).is_err(), "{bad}");
        }
        assert!(looks_private("Logic: Cmd+Shift+R records 2 takes at 124 BPM").is_none(), "normal shortcuts are fine");
        delete(&c, &f.id).unwrap();
        assert!(delete(&c, &f.id).is_err());
        add(&c, "workflow", "a", &json!(1), "explicit", "standard", 1.0, None).unwrap();
        add(&c, "style", "b", &json!(1), "explicit", "custom", 1.0, None).unwrap();
        assert_eq!(clear(&c, "workflow").unwrap(), 1);
        assert_eq!(list(&c, None).unwrap().len(), 1, "other categories untouched");
    }
}
