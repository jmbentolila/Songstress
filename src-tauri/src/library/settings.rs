use rusqlite::Connection;
use std::collections::HashMap;

/// All settings at once (startup hydration).
pub fn all(conn: &Connection) -> rusqlite::Result<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
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

// --- per-album display overrides (2026-09-26) ---------------------------------
//
// The panel-gradient override used to live in ONE `panelGradients` blob (album
// id → pair). It moved to one key per album when the sidecar arrived: the scan
// adopts a `.songstress.json` value straight into that album's row, and a blob
// would let the app's in-memory map — hydrated before the scan ran — clobber
// the adopted value the moment it next persisted the whole thing.

pub const ALBUM_GRADIENT_PREFIX: &str = "albumGradient:";

pub fn album_gradient(conn: &Connection, album_id: &str) -> Option<[String; 2]> {
    let key = format!("{ALBUM_GRADIENT_PREFIX}{album_id}");
    let raw: String = conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [&key], |r| {
            r.get(0)
        })
        .ok()?;
    let pair: Vec<String> = serde_json::from_str(&raw).ok()?;
    <[String; 2]>::try_from(pair).ok()
}

/// Set (Some) or clear (None) one album's gradient override.
pub fn set_album_gradient(
    conn: &Connection,
    album_id: &str,
    pair: Option<&[String; 2]>,
) -> rusqlite::Result<()> {
    let key = format!("{ALBUM_GRADIENT_PREFIX}{album_id}");
    match pair {
        Some(pair) => set(conn, &key, &serde_json::to_string(pair).unwrap_or_default()),
        None => {
            conn.execute("DELETE FROM settings WHERE key = ?1", [&key])?;
            Ok(())
        }
    }
}

/// One-time conversion of the legacy `panelGradients` blob into per-album keys.
/// Idempotent: no blob, nothing to do. Returns how many entries moved.
pub fn migrate_panel_gradients(conn: &Connection) -> rusqlite::Result<usize> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'panelGradients'",
            [],
            |r| r.get(0),
        )
        .ok();
    let Some(raw) = raw else { return Ok(0) };
    let map: HashMap<String, serde_json::Value> = serde_json::from_str(&raw).unwrap_or_default();
    let mut moved = 0;
    for (album_id, value) in &map {
        let pair: Option<[String; 2]> = (|| {
            let c1 = value.get("c1")?.as_str()?.to_string();
            let c2 = value.get("c2")?.as_str()?.to_string();
            Some([c1, c2])
        })();
        if let Some(pair) = pair {
            // Never overwrite a per-album key that already exists — a run of
            // this migration must not undo what the sidecar adopted.
            let key = format!("{ALBUM_GRADIENT_PREFIX}{album_id}");
            let exists: bool = conn
                .query_row("SELECT 1 FROM settings WHERE key = ?1", [&key], |_| {
                    Ok(true)
                })
                .unwrap_or(false);
            if !exists {
                set_album_gradient(conn, album_id, Some(&pair))?;
                moved += 1;
            }
        }
    }
    conn.execute("DELETE FROM settings WHERE key = 'panelGradients'", [])?;
    Ok(moved)
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

    #[test]
    fn album_gradient_round_trips_and_clears() {
        let path = std::env::temp_dir()
            .join("songstress-m1-tests")
            .join(format!("gradient-{}.db", std::process::id()));
        let conn = db::open(&path).expect("open");
        assert_eq!(album_gradient(&conn, "al-1"), None, "nothing stored yet");
        let pair = ["ff8800".to_string(), "0044cc".to_string()];
        set_album_gradient(&conn, "al-1", Some(&pair)).expect("set");
        assert_eq!(album_gradient(&conn, "al-1"), Some(pair.clone()));
        // Another album is untouched — the keys are per album, not one blob.
        assert_eq!(album_gradient(&conn, "al-2"), None);
        set_album_gradient(&conn, "al-1", None).expect("clear");
        assert_eq!(album_gradient(&conn, "al-1"), None);
        // A malformed value reads as absent rather than panicking.
        set(&conn, "albumGradient:al-3", "[\"only-one\"]").expect("seed");
        assert_eq!(album_gradient(&conn, "al-3"), None);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn legacy_panel_gradient_blob_migrates_once() {
        let path = std::env::temp_dir()
            .join("songstress-m1-tests")
            .join(format!("blob-{}.db", std::process::id()));
        let conn = db::open(&path).expect("open");
        set(
            &conn,
            "panelGradients",
            "{\"al-1\":{\"c1\":\"ff0000\",\"c2\":\"00ff00\"},\"al-2\":{\"c1\":\"111111\",\"c2\":\"222222\"}}",
        )
        .expect("seed blob");
        // A sidecar already adopted one album: the migration must NOT undo it.
        let adopted = ["aaaaaa".to_string(), "bbbbbb".to_string()];
        set_album_gradient(&conn, "al-2", Some(&adopted)).expect("pre-adopt");

        assert_eq!(migrate_panel_gradients(&conn).expect("migrate"), 1);
        assert_eq!(
            album_gradient(&conn, "al-1"),
            Some(["ff0000".to_string(), "00ff00".to_string()])
        );
        assert_eq!(
            album_gradient(&conn, "al-2"),
            Some(adopted),
            "the adopted value survives the migration"
        );
        assert!(
            !all(&conn).expect("all").contains_key("panelGradients"),
            "the blob is retired"
        );
        assert_eq!(
            migrate_panel_gradients(&conn).expect("again"),
            0,
            "idempotent"
        );
        let _ = std::fs::remove_file(&path);
    }
}
