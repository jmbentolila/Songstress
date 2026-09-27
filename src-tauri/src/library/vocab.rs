//! Tag vocabulary: the distinct values the library already holds, per field,
//! for the tag editors' as-you-type suggestions (migration v6).
//!
//! Why the DB and not the files: building a suggestion list by reading tags
//! costs a full lofty pass on every editor open, and the scan has already paid
//! for most of it — v6 persists genre/composer/label/grouping (plus the album
//! artist) for exactly this. Counts ride along because the editors rank by
//! popularity *after* match position (the most-used value is what a bare "Ro"
//! should complete to); they are never displayed.
//!
//! Not authoritative by design: a value no file has contributed simply is not
//! suggested — the same deal the album datalist always made with
//! `library.albums`. Empty and whitespace-only values are not suggestions.

use rusqlite::Connection;

/// Per-field cap. A large library can hold thousands of distinct groupings;
/// no list benefits from rendering them, and this keeps both the IPC payload
/// and the popover's DOM bounded.
pub const CAP: i64 = 400;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct VocabValue {
    pub value: String,
    pub count: i64,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct Vocabulary {
    pub artist: Vec<VocabValue>,
    pub album_artist: Vec<VocabValue>,
    pub album: Vec<VocabValue>,
    pub genre: Vec<VocabValue>,
    pub composer: Vec<VocabValue>,
    pub label: Vec<VocabValue>,
    pub grouping: Vec<VocabValue>,
}

fn query(stmt: &mut rusqlite::Statement<'_>, limit: i64) -> Result<Vec<VocabValue>, String> {
    let rows = stmt
        .query_map([limit], |r| {
            Ok(VocabValue {
                value: r.get(0)?,
                count: r.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|e| e.to_string())
}

/// Case-insensitive folding: "rock" and "Rock" are ONE suggestion, wearing the
/// spelling the library uses most (ties go to the ASCII-first spelling so the
/// order is reproducible). The SQL delivered rows most-used first, so the
/// first row of a folded group is already the spelling to keep.
///
/// Diacritics are NOT folded here: "Motörhead" and "Motorhead" are different
/// names and the user may care which one their files carry. The editors'
/// *matching* does fold them, so typing "motor" still finds both.
fn merge_folded(rows: Vec<VocabValue>, limit: i64) -> Vec<VocabValue> {
    let mut out: Vec<VocabValue> = Vec::new();
    for row in rows {
        match out
            .iter_mut()
            .find(|v| v.value.eq_ignore_ascii_case(&row.value))
        {
            Some(slot) => slot.count += row.count,
            None => out.push(row),
        }
    }
    out.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.value.to_lowercase().cmp(&b.value.to_lowercase()))
    });
    out.truncate(limit as usize);
    out
}

/// Distinct trimmed non-empty values of one tracks column, most used first,
/// then folded alphabetically. `column` is a literal from this file — never
/// user input — so the interpolation is safe; SQLite cannot parametrise an
/// identifier.
fn column(conn: &Connection, column: &str, limit: i64) -> Result<Vec<VocabValue>, String> {
    let sql = format!(
        "SELECT trim({column}) AS value, COUNT(*) AS n FROM tracks
          WHERE {column} IS NOT NULL AND trim({column}) != ''
          GROUP BY trim({column})
          ORDER BY n DESC, value COLLATE NOCASE, value
          LIMIT ?1"
    );
    // Twice the cap into the fold: case-variant spellings are merged after the
    // query, so a library with many of them must not lose its long tail.
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = query(&mut stmt, limit * 2)?;
    Ok(merge_folded(rows, limit))
}

/// The Artist field suggests two things at once: the grouped artist (who owns
/// the album — `tracks.artist` is NULL for most rows, the album artist stands
/// in) and the per-track artist (guest spots, feats).
fn artists(conn: &Connection, limit: i64) -> Result<Vec<VocabValue>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT trim(value) AS value, COUNT(*) AS n FROM (
                 SELECT name AS value FROM artists
                 UNION ALL
                 SELECT artist AS value FROM tracks
                  WHERE artist IS NOT NULL AND trim(artist) != ''
             )
             WHERE trim(value) != ''
             GROUP BY trim(value)
             ORDER BY n DESC, value COLLATE NOCASE, value
             LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = query(&mut stmt, limit * 2)?;
    Ok(merge_folded(rows, limit))
}

/// Album titles, counted by the tracks that live on them — a suggestion list
/// ranked by how much of the library a title actually names.
fn albums(conn: &Connection, limit: i64) -> Result<Vec<VocabValue>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT trim(a.title) AS value, COUNT(t.id) AS n
               FROM albums a JOIN tracks t ON t.album_id = a.id
              WHERE trim(a.title) != ''
              GROUP BY trim(a.title)
              ORDER BY n DESC, value COLLATE NOCASE, value
              LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = query(&mut stmt, limit * 2)?;
    Ok(merge_folded(rows, limit))
}

pub fn vocabulary(conn: &Connection) -> Result<Vocabulary, String> {
    Ok(Vocabulary {
        artist: artists(conn, CAP)?,
        album_artist: column(conn, "album_artist", CAP)?,
        album: albums(conn, CAP)?,
        genre: column(conn, "genre", CAP)?,
        composer: column(conn, "composer", CAP)?,
        label: column(conn, "label", CAP)?,
        grouping: column(conn, "grouping", CAP)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::db;

    fn seeded(tag: &str, seed: &str) -> Connection {
        let dir = std::env::temp_dir().join("songstress-vocab-tests");
        std::fs::create_dir_all(&dir).expect("create test dir");
        let path = dir.join(format!("{tag}-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let conn = db::open(&path).expect("open");
        conn.execute_batch(seed).expect("seed");
        conn
    }

    fn values(v: &[VocabValue]) -> Vec<&str> {
        v.iter().map(|x| x.value.as_str()).collect()
    }

    #[test]
    fn ranks_by_use_then_alphabetically_and_folds_case() {
        let conn = seeded(
            "rank",
            "INSERT INTO artists VALUES ('ar-1','Helloween','helloween');
             INSERT INTO albums VALUES ('al-1','ar-1','Keeper I',1987,NULL,NULL,NULL);
             INSERT INTO tracks(id, album_id, title, duration_sec, path, mtime_ns, size, genre)
             VALUES ('tr-1','al-1','a',1.0,'/m/a.flac',1,1,'Power Metal'),
                    ('tr-2','al-1','b',1.0,'/m/b.flac',1,1,'rock'),
                    ('tr-3','al-1','c',1.0,'/m/c.flac',1,1,'  Rock  '),
                    ('tr-4','al-1','d',1.0,'/m/d.flac',1,1,'Ambient'),
                    ('tr-5','al-1','e',1.0,'/m/e.flac',1,1,'   ');",
        );
        let genres = column(&conn, "genre", CAP).expect("query");
        assert_eq!(
            values(&genres),
            vec!["Rock", "Ambient", "Power Metal"],
            "the case-folded pair is ONE suggestion wearing the ASCII-first \
             spelling, blanks are dropped, ties fall alphabetically"
        );
        assert_eq!(genres[0].count, 2, "the folded pair's counts add up");
    }

    #[test]
    fn diacritics_are_distinct_names() {
        let conn = seeded(
            "diacritics",
            "INSERT INTO artists VALUES ('ar-1','X','x');
             INSERT INTO albums VALUES ('al-1','ar-1','A',2020,NULL,NULL,NULL);
             INSERT INTO tracks(id, album_id, title, duration_sec, path, mtime_ns, size, genre)
             VALUES ('tr-1','al-1','a',1.0,'/m/a.flac',1,1,'Motörhead'),
                    ('tr-2','al-1','b',1.0,'/m/b.flac',1,1,'Motorhead');",
        );
        let genres = column(&conn, "genre", CAP).expect("query");
        assert_eq!(genres.len(), 2, "folding stops at ASCII case: {:?}", values(&genres));
    }

    #[test]
    fn artist_merges_grouped_and_per_track_names() {
        let conn = seeded(
            "artists",
            "INSERT INTO artists VALUES ('ar-1','Ghost','ghost');
             INSERT INTO albums VALUES ('al-1','ar-1','Meliora',2015,NULL,NULL,NULL);
             INSERT INTO tracks(id, album_id, title, duration_sec, path, mtime_ns, size, artist)
             VALUES ('tr-1','al-1','a',1.0,'/m/a.flac',1,1,NULL),
                    ('tr-2','al-1','b',1.0,'/m/b.flac',1,1,'Dave Grohl'),
                    ('tr-3','al-1','c',1.0,'/m/c.flac',1,1,' dave grohl ');",
        );
        let a = artists(&conn, CAP).expect("query");
        assert_eq!(values(&a), vec!["Dave Grohl", "Ghost"]);
        assert_eq!(a[0].count, 2, "trimmed + folded: the guest spot counts twice");
    }

    #[test]
    fn album_titles_count_their_tracks() {
        let conn = seeded(
            "albums",
            "INSERT INTO artists VALUES ('ar-1','X','x');
             INSERT INTO albums VALUES ('al-1','ar-1','Big',2020,NULL,NULL,NULL);
             INSERT INTO albums VALUES ('al-2','ar-1','Small',2021,NULL,NULL,NULL);
             INSERT INTO tracks(id, album_id, title, duration_sec, path, mtime_ns, size)
             VALUES ('tr-1','al-1','a',1.0,'/m/a.flac',1,1),
                    ('tr-2','al-1','b',1.0,'/m/b.flac',1,1),
                    ('tr-3','al-2','c',1.0,'/m/c.flac',1,1);",
        );
        let al = albums(&conn, CAP).expect("query");
        assert_eq!(values(&al), vec!["Big", "Small"]);
        assert_eq!(al[0].count, 2);
    }

    #[test]
    fn limit_caps_every_field_after_folding() {
        let conn = seeded(
            "cap",
            "INSERT INTO artists VALUES ('ar-1','X','x');
             INSERT INTO albums VALUES ('al-1','ar-1','A',2020,NULL,NULL,NULL);
             INSERT INTO tracks(id, album_id, title, duration_sec, path, mtime_ns, size, genre)
             VALUES ('tr-1','al-1','a',1.0,'/m/a.flac',1,1,'One'),
                    ('tr-2','al-1','b',1.0,'/m/b.flac',1,1,'Two'),
                    ('tr-3','al-1','c',1.0,'/m/c.flac',1,1,'two'),
                    ('tr-4','al-1','d',1.0,'/m/d.flac',1,1,'Three');",
        );
        let two = column(&conn, "genre", 2).expect("query");
        assert_eq!(two.len(), 2, "the folded pair still counts as one: {:?}", values(&two));
        assert_eq!(two[0].count, 2, "…and is the most used after folding");
        assert_eq!(artists(&conn, 1).expect("query").len(), 1);
        assert_eq!(albums(&conn, 1).expect("query").len(), 1);
    }

    #[test]
    fn empty_database_yields_empty_lists() {
        let conn = seeded("empty", "");
        let v = vocabulary(&conn).expect("query");
        assert!(v.artist.is_empty() && v.genre.is_empty() && v.album.is_empty());
    }

    /// End to end through the thing that actually fills these columns: tag a
    /// real fixture file, scan it, read the vocabulary back. The unit tests
    /// above seed SQL directly, so without this one nothing would catch the
    /// scan dropping a field on the floor between parse and insert (which is
    /// exactly how `tracks.artist` shipped empty for months).
    #[test]
    fn scan_persists_the_vocabulary_it_reads() {
        use lofty::file::{AudioFile, TaggedFileExt};
        use lofty::tag::{Accessor, ItemKey};

        let dir = std::env::temp_dir()
            .join(format!("songstress-vocab-scan-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let album = dir.join("Ghost/Meliora (2015)");
        std::fs::create_dir_all(&album).expect("mkdir");
        let dest = album.join("01 - Spirit.mp3");
        std::fs::copy(
            "fixtures/library/Helloween/Giants & Monsters (2021)/02 - Throne of the Iron Vigil.mp3",
            &dest,
        )
        .expect("copy fixture");
        {
            let mut tagged = lofty::read_from_path(&dest).expect("read fixture");
            let tag = tagged.primary_tag_mut().expect("primary tag");
            tag.set_genre("Heavy Metal".into());
            tag.set_artist("Ghost".into());
            tag.set_album("Meliora".into());
            tag.insert_text(ItemKey::AlbumArtist, "Ghost".into());
            tag.insert_text(ItemKey::Composer, "A Ghoul Writer".into());
            tag.insert_text(ItemKey::Label, "Loma Vista".into());
            tag.insert_text(ItemKey::ContentGroup, "Act I".into());
            tagged
                .save_to_path(&dest, lofty::config::WriteOptions::default())
                .expect("save tags");
        }

        let dbp = dir.join("t.db");
        let mut conn = db::open(&dbp).expect("open");
        crate::library::scan::run_scan(&mut conn, &dir, |_, _| {}).expect("scan");

        let v = vocabulary(&conn).expect("vocab");
        assert_eq!(values(&v.genre), vec!["Heavy Metal"], "genre persisted");
        assert_eq!(values(&v.composer), vec!["A Ghoul Writer"]);
        assert_eq!(values(&v.label), vec!["Loma Vista"]);
        assert_eq!(values(&v.grouping), vec!["Act I"]);
        assert_eq!(values(&v.album_artist), vec!["Ghost"]);
        assert_eq!(values(&v.album), vec!["Meliora"]);
        assert!(
            values(&v.artist).contains(&"Ghost"),
            "the grouped artist answers for the Artist field: {:?}",
            values(&v.artist)
        );

        let (g, c, l, gr, aa): (
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ) = conn
            .query_row(
                "SELECT genre, composer, label, grouping, album_artist FROM tracks LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .expect("row");
        assert_eq!(g.as_deref(), Some("Heavy Metal"));
        assert_eq!(c.as_deref(), Some("A Ghoul Writer"));
        assert_eq!(l.as_deref(), Some("Loma Vista"));
        assert_eq!(gr.as_deref(), Some("Act I"));
        assert_eq!(aa.as_deref(), Some("Ghost"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}