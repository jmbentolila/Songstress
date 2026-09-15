use rusqlite::Connection;

/// Ordered, forward-only migrations. Never edit an applied migration — append
/// a new one. `schema_migrations` tracks what has run; each entry executes
/// inside its own transaction.
pub const MIGRATIONS: &[&str] = &[
    // v1 — initial schema (PHASE2.md §4.1). IDs are stable text hashes so
    // rescans upsert in place and MPRIS trackids stay valid later.
    "CREATE TABLE artists (
         id        TEXT PRIMARY KEY,
         name      TEXT NOT NULL,
         sort_name TEXT NOT NULL
     );
     CREATE TABLE albums (
         id        TEXT PRIMARY KEY,
         artist_id TEXT NOT NULL REFERENCES artists(id),
         title     TEXT NOT NULL,
         year      INTEGER,
         cover     TEXT,
         color_c1  TEXT,
         color_c2  TEXT,
         UNIQUE(artist_id, title, year)
     );
     CREATE TABLE tracks (
         id           TEXT PRIMARY KEY,
         album_id     TEXT NOT NULL REFERENCES albums(id),
         disc         INTEGER NOT NULL DEFAULT 1,
         track        INTEGER,
         title        TEXT NOT NULL,
         duration_sec REAL NOT NULL,
         path         TEXT NOT NULL UNIQUE,
         mtime_ns     INTEGER NOT NULL,
         size         INTEGER NOT NULL
     );
     CREATE INDEX idx_tracks_album ON tracks(album_id, disc, track);
     CREATE TABLE settings (
         key   TEXT PRIMARY KEY,
         value TEXT NOT NULL
     );",
    // v2 — staging is a FLAG, not a folder. Imported files are indexed where
    // they are and carry `staged` until the user saves (the app moves them into
    // the library) or discards them (the row is forgotten, the file is left
    // alone). The cache directory that used to hold a second copy of everything
    // imports is retired by this. Rows the copy era left under the staging dir
    // are flagged at startup by `import::mark_legacy_staged`, which knows where
    // the cache lives and a migration here does not.
    "ALTER TABLE tracks ADD COLUMN staged INTEGER NOT NULL DEFAULT 0;",
    // v3 — per-track artist. The scan always knew it (guest/feats stay guests
    // under the albumartist), it just never stored it — so the playbar could
    // only show the ALBUM artist. Nullable: pre-v3 rows read back NULL until
    // a rescan fills them, and the frontend falls back to the album artist.
    "ALTER TABLE tracks ADD COLUMN artist TEXT;",
    // v4 — re-extract panel colors under the Step 9b rule (two named hues:
    // most vivid significant family + most colorful distant family, quiet
    // first — never the whole-artwork average, which photographed bimodal
    // art as mud). NULLing refills them through the normal artwork refresh
    // on the next scan; covers are untouched, so no thumbnail churn beyond
    // a byte-identical rewrite. Settings overrides win at render time and
    // are unaffected by this migration either way.
    "UPDATE albums SET color_c1 = NULL, color_c2 = NULL;",
    // v5 — same refill, second round: the anchor score is now hue
    // opposition × chroma (RGB distance let warm-grey mush outscore the
    // real complement — measured tan beating teal on Moonflower). Mush is
    // hue-adjacent to every hot star by construction, so whatever wins now
    // is the greenest distant region available, never the mud.
    "UPDATE albums SET color_c1 = NULL, color_c2 = NULL;",
];

fn apply_migrations(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         CREATE TABLE IF NOT EXISTS schema_migrations (
             version    INTEGER PRIMARY KEY,
             applied_at TEXT NOT NULL DEFAULT (datetime('now'))
         );",
    )?;
    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;
    for (idx, sql) in MIGRATIONS.iter().enumerate() {
        let version = (idx + 1) as i64;
        if version <= current {
            continue;
        }
        // Migration + bookkeeping commit atomically: a failed migration must
        // not leave its version marked as applied.
        conn.execute_batch(&format!(
            "BEGIN; {sql}; INSERT INTO schema_migrations(version) VALUES ({version}); COMMIT;"
        ))?;
    }
    Ok(())
}

/// Open (creating if needed) a library database with all migrations applied.
pub fn open(path: &std::path::Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    apply_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db_path(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("songstress-m1-tests");
        std::fs::create_dir_all(&dir).expect("create test dir");
        dir.join(format!("{tag}-{}.db", std::process::id()))
    }

    #[test]
    fn migrations_apply_and_are_idempotent() {
        let path = temp_db_path("idem");
        {
            let conn = open(&path).expect("first open");
            let v: i64 = conn
                .query_row("SELECT MAX(version) FROM schema_migrations", [], |r| r.get(0))
                .unwrap();
            assert_eq!(v, MIGRATIONS.len() as i64);
        }
        {
            // Second open must not re-run or fail on existing tables.
            let conn = open(&path).expect("second open");
            let n: i64 = conn
                .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
                .unwrap();
            assert_eq!(n, MIGRATIONS.len() as i64);
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn schema_supports_expected_queries() {
        let path = temp_db_path("schema");
        let conn = open(&path).expect("open");
        conn.execute_batch(
            "INSERT INTO artists VALUES ('ar-1', 'Helloween', 'helloween');
             INSERT INTO albums VALUES ('al-1', 'ar-1', 'Giants & Monsters', 2021, NULL, 'ff8800', '0044cc');
             INSERT INTO tracks VALUES ('tr-1', 'al-1', 1, 1, 'Silent Echoes', 336.0,
                 '/music/helloween/giants/01.flac', 123, 456, 0, NULL);",
        )
        .expect("seed");
        let (title, c1): (String, String) = conn
            .query_row(
                "SELECT t.title, a.color_c1 FROM tracks t JOIN albums a ON a.id = t.album_id",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("join query");
        assert_eq!(title, "Silent Echoes");
        assert_eq!(c1, "ff8800");
        let _ = std::fs::remove_file(&path);
    }
}
