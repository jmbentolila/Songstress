use rusqlite::Connection;
use std::collections::HashMap;

/// All settings at once (startup hydration).
pub fn all(conn: &Connection) -> rusqlite::Result<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
    let mut out = HashMap::new();
    for row in rows {
        let (k, v) = row?;
        out.insert(k, v);
    }
    Ok(out)
}

/// Upsert one setting. Values are JSON text; the DB layer stays schemaless.
pub fn set(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings(key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, value],
    )?;
    Ok(())
}

/// First-run migration of frontend localStorage values: only fills keys that
/// don't exist yet — the DB never clobbers values it already owns.
pub fn init_missing(conn: &Connection, values: &HashMap<String, String>) -> rusqlite::Result<()> {
    for (key, value) in values {
        conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO NOTHING",
            [key.as_str(), value.as_str()],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::db;
    use super::*;

    #[test]
    fn settings_round_trip_and_init_semantics() {
        let path = std::env::temp_dir()
            .join("songstress-m1-tests")
            .join(format!("settings-{}.db", std::process::id()));
        let conn = db::open(&path).expect("open");

        // First run: localStorage values seed the DB.
        let mut local = HashMap::new();
        local.insert("theme".to_string(), "\"dark\"".to_string());
        local.insert("volume".to_string(), "80".to_string());
        init_missing(&conn, &local).expect("init");

        // User changes something → DB wins over later init calls.
        set(&conn, "volume", "35").expect("set");
        let mut stale = HashMap::new();
        stale.insert("volume".to_string(), "99".to_string());
        stale.insert("tileSize".to_string(), "180".to_string());
        init_missing(&conn, &stale).expect("re-init");

        let all = all(&conn).expect("all");
        assert_eq!(all.get("volume").unwrap(), "35"); // not clobbered
        assert_eq!(all.get("theme").unwrap(), "\"dark\"");
        assert_eq!(all.get("tileSize").unwrap(), "180");
        assert_eq!(all.len(), 3);
        let _ = std::fs::remove_file(&path);
    }
}
