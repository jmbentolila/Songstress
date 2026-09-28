//! `.songstress.json` — the per-album display state, stored in the album's own
//! folder so it travels with the music. Owner ask, 2026-09-26:
//!
//! > the important thing is that the user is able to easily go from one device
//! > to another and keep its library having the exact same behavior inside
//! > Songstress
//!
//! The DB stays authoritative at RUNTIME; this file is the portable half:
//!
//!   - **A SCAN adopts.** The file wins wherever it carries info, so a
//!     hand-edit plus a rescan is a supported way to change a look, and a
//!     library copied to a fresh install arrives styled.
//!   - **An in-app change mirrors.** The app is the author there, so the DB
//!     state is written INTO the file (no adoption — see `sync_album`'s note;
//!     the first version re-adopted and put the old value back).
//!   - **One file per folder, one entry per album.** The folder is where the
//!     file can live; the album is the identity. A folder CAN hold several
//!     albums — measured in the owner's library: a flat compilation folder
//!     carrying three different album tags — so the file is a map keyed by
//!     album id. A per-album filename would need slugs (collisions, unicode)
//!     and a single flat object let them overwrite each other.
//!   - **Written for every album's folder**, version-only when there is
//!     nothing to carry: the owner wants the file "just in case", and an empty
//!     one is what makes it hand-editable before there is anything to edit.
//!   - Never rewritten when the bytes already match, so the file's mtime does
//!     not churn (and the watcher, which ignores this filename, is not asked).
//!
//! Keys in each entry:
//!
//!   - `colors` — the artwork-derived pair, mirroring `albums.color_c1/c2`.
//!     Carried as a FALLBACK: it is used only when there is no artwork to
//!     derive from (copied library, lost `folder.jpg`, unreadable format), so
//!     "use album colors" always reflects the artwork you actually have, while
//!     an album that lost its art keeps its look. Delete the key (or the file)
//!     to let the app re-derive.
//!   - `gradient` — the user's panel-override pair, mirroring the per-album
//!     setting `albumGradient:<album id>`. A real override: it wins on scan.
//!
//! Where the next per-album display state goes: another key in `AlbumState`,
//! behind the same `version`. Nothing here is a tag — tags belong to the files
//! and are read by the scan; this is only what Songstress remembers about how
//! it SHOWS the album.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::settings;
use rusqlite::Connection;

/// The filename, hidden so file managers and tag editors leave it alone.
pub const FILE: &str = ".songstress.json";
/// Format version. Bump when a change cannot be read by the previous reader.
pub const VERSION: u32 = 1;

/// One album's stored display state. Absent keys mean "nothing to carry", and
/// an album with nothing to carry gets no entry at all.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AlbumState {
    /// Artwork-derived colors (hex without `#`, the DB's own form) — a fallback
    /// for when the artwork cannot be read, not an override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colors: Option<[String; 2]>,
    /// The user's panel-gradient override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gradient: Option<[String; 2]>,
}

impl AlbumState {
    fn is_empty(&self) -> bool {
        self.colors.is_none() && self.gradient.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sidecar {
    pub version: u32,
    /// Album id → state, for the albums whose files live in this folder.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub albums: BTreeMap<String, AlbumState>,
}

impl Default for Sidecar {
    fn default() -> Self {
        Sidecar {
            version: VERSION,
            albums: BTreeMap::new(),
        }
    }
}

pub fn path_in(folder: &Path) -> PathBuf {
    folder.join(FILE)
}

/// Is this path ours? The watcher asks, so our own writes never schedule a
/// scan — and the answer covers the temp name too, because a rename shows up
/// as its own event (`.songstress.json.tmp-<pid>` → `.songstress.json`).
pub fn is_sidecar_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with(FILE))
}

/// Read a folder's sidecar. A missing file is `None` (the normal case for a
/// library that has never been opened by this version); a malformed one is also
/// `None`, but says so in the journal — the file is hand-editable, so silence
/// would hide a typo instead of showing it.
pub fn read(folder: &Path) -> Option<Sidecar> {
    let path = path_in(folder);
    let raw = std::fs::read_to_string(&path).ok()?;
    match serde_json::from_str::<Sidecar>(&raw) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("[sidecar] ignoring malformed {}: {e}", path.display());
            None
        }
    }
}

/// Write a folder's sidecar, atomically, and only when the bytes differ.
/// Returns whether the file changed.
///
/// The temp file is ours and lives beside the target (same directory, so the
/// rename cannot cross a filesystem): the app NEVER derives a path from an
/// audio file's name, and never touches anything in the folder but this one
/// name plus its own temp. See the 2026-09-04 rule in AGENTS.md.
pub fn write(folder: &Path, sidecar: &Sidecar) -> Result<bool, String> {
    let path = path_in(folder);
    let mut body = serde_json::to_string_pretty(sidecar).map_err(|e| e.to_string())?;
    body.push('\n');
    if std::fs::read_to_string(&path).is_ok_and(|current| current == body) {
        return Ok(false);
    }
    let tmp = folder.join(format!("{FILE}.tmp-{}", std::process::id()));
    std::fs::write(&tmp, body.as_bytes()).map_err(|e| format!("sidecar temp: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("sidecar rename: {e}")
    })?;
    Ok(true)
}

// --- the per-album pass -------------------------------------------------------

#[derive(Debug, Default, PartialEq)]
pub struct SyncCounts {
    /// Albums visited (their folder exists).
    pub albums: usize,
    /// Files actually (re)written — identical bytes are skipped.
    pub written: usize,
    /// Entries that overrode a gradient the DB did not already hold.
    pub adopted_gradient: usize,
    /// Albums whose colours had to come from the file (no artwork to derive from).
    pub adopted_colors: usize,
}

fn pair_of(c1: Option<String>, c2: Option<String>) -> Option<[String; 2]> {
    match (c1, c2) {
        (Some(a), Some(b)) if !a.trim().is_empty() && !b.trim().is_empty() => Some([a, b]),
        _ => None,
    }
}

/// The folder holding most of an album's tracks — the same "primary folder"
/// rule the import destination cascade uses, so an album that spans several
/// folders (a regular edition plus a special edition, measured) gets ONE entry
/// in ONE file rather than a copy per disc.
fn primary_folder(conn: &Connection, album_id: &str) -> Result<Option<String>, String> {
    let mut stmt = conn
        .prepare("SELECT path FROM tracks WHERE album_id = ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([album_id], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let mut tally: HashMap<String, usize> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for row in rows {
        let path = row.map_err(|e| e.to_string())?;
        let Some(parent) = Path::new(&path).parent() else {
            continue;
        };
        let parent = parent.to_string_lossy().into_owned();
        if !tally.contains_key(&parent) {
            order.push(parent.clone());
        }
        *tally.entry(parent).or_insert(0) += 1;
    }
    Ok(order.into_iter().max_by_key(|p| tally[p]))
}

/// Every album that has a folder on disk, grouped by that folder. Albums whose
/// tracks are all missing (no folder left) are simply absent — there is nowhere
/// to keep a file.
fn albums_by_folder(conn: &Connection) -> Result<BTreeMap<String, Vec<String>>, String> {
    let ids: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT id FROM albums ORDER BY id")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|e| e.to_string())?
    };
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for album_id in ids {
        let Some(folder) = primary_folder(conn, &album_id)? else {
            continue;
        };
        if !Path::new(&folder).is_dir() {
            continue;
        }
        out.entry(folder).or_default().push(album_id);
    }
    Ok(out)
}

fn album_colors(conn: &Connection, album_id: &str) -> Result<Option<[String; 2]>, String> {
    let (c1, c2): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT color_c1, color_c2 FROM albums WHERE id = ?1",
            [album_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|e| e.to_string())?;
    Ok(pair_of(c1, c2))
}

/// Bring ONE album's entry in a folder's sidecar in line, adopting first when
/// this is a scan.
///
/// `adopt` is the direction, and getting it wrong is not subtle: the first
/// version re-adopted on the write path, so setting a gradient wrote the new
/// value to the DB and then put the OLD one back into both.
fn sync_entry(
    conn: &Connection,
    file: &mut Sidecar,
    album_id: &str,
    adopt: bool,
    counts: &mut SyncCounts,
) -> Result<(), String> {
    let recorded = file.albums.get(album_id).cloned().unwrap_or_default();

    // A gradient is a user override: on a scan the file wins, always.
    if adopt
        && let Some(pair) = recorded.gradient.clone()
            && settings::album_gradient(conn, album_id).as_ref() != Some(&pair) {
                settings::set_album_gradient(conn, album_id, Some(&pair))
                    .map_err(|e| e.to_string())?;
                counts.adopted_gradient += 1;
            }

    // Colours belong to the artwork. The file answers ONLY the case where there
    // is no pair to derive from (no artwork, or extraction failed) — it is an
    // initial state for a library that moved, not a daily referral.
    let derived = album_colors(conn, album_id)?;
    let colors = match derived.clone() {
        Some(pair) => Some(pair),
        None => recorded.colors.clone(),
    };
    if derived.is_none() && adopt
        && let Some(pair) = colors.clone() {
            conn.execute(
                "UPDATE albums SET color_c1 = ?2, color_c2 = ?3 WHERE id = ?1",
                rusqlite::params![album_id, pair[0], pair[1]],
            )
            .map_err(|e| e.to_string())?;
            counts.adopted_colors += 1;
        }

    let state = AlbumState {
        colors,
        gradient: settings::album_gradient(conn, album_id),
    };
    if state.is_empty() {
        file.albums.remove(album_id);
    } else {
        file.albums.insert(album_id.to_string(), state);
    }
    Ok(())
}

/// Re-mirror ONE album after an in-app change (the gradient command). Returns
/// whether the file changed. No adoption: the caller just set the DB state, so
/// that is what the file is brought in line with — and the OTHER albums sharing
/// the folder keep their entries.
pub fn sync_one(conn: &Connection, album_id: &str) -> Result<bool, String> {
    let Some(folder) = primary_folder(conn, album_id)? else {
        return Ok(false);
    };
    let folder = PathBuf::from(folder);
    if !folder.is_dir() {
        return Ok(false);
    }
    let mut file = read(&folder).unwrap_or_default();
    let mut counts = SyncCounts::default();
    sync_entry(conn, &mut file, album_id, false, &mut counts)?;
    write(&folder, &file)
}

/// The per-album display-state pass: read each folder's sidecar, adopt what it
/// carries, mirror the final state back, and drop entries whose album no longer
/// lives here (retagged, retitled, moved). Run at the END of a scan, after the
/// artwork pass, so `colors` reflect the freshly derived pair.
///
/// Idempotent by construction: an unchanged album reads what it wrote, adopts
/// nothing, and `write` skips identical bytes.
pub fn sync_all(conn: &Connection) -> Result<SyncCounts, String> {
    let mut counts = SyncCounts::default();
    for (folder, album_ids) in albums_by_folder(conn)? {
        let folder = PathBuf::from(folder);
        let mut file = read(&folder).unwrap_or_default();
        for album_id in &album_ids {
            counts.albums += 1;
            sync_entry(conn, &mut file, album_id, true, &mut counts)?;
        }
        // Prune: an entry whose album is no longer in this folder is stale
        // (the album was retagged or moved away). Leave no orphans behind —
        // the file is the only record, and a stale entry would resurrect a
        // look for an album that no longer exists.
        let live: HashSet<&String> = album_ids.iter().collect();
        let stale: Vec<String> = file
            .albums
            .keys()
            .filter(|id| !live.contains(id))
            .cloned()
            .collect();
        for id in stale {
            file.albums.remove(&id);
        }
        if write(&folder, &file)? {
            counts.written += 1;
        }
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("songstress-sidecar-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn state(colors: Option<[&str; 2]>, gradient: Option<[&str; 2]>) -> AlbumState {
        let pair = |p: Option<[&str; 2]>| p.map(|[a, b]| [a.to_string(), b.to_string()]);
        AlbumState {
            colors: pair(colors),
            gradient: pair(gradient),
        }
    }

    fn pair(a: &str, b: &str) -> [String; 2] {
        [a.to_string(), b.to_string()]
    }

    #[test]
    fn round_trips_and_writes_an_empty_file_for_a_bare_folder() {
        let dir = temp_dir("round");
        assert!(read(&dir).is_none(), "no file yet");
        assert!(write(&dir, &Sidecar::default()).expect("write"));
        let raw = std::fs::read_to_string(path_in(&dir)).expect("raw");
        assert_eq!(
            raw, "{\n  \"version\": 1\n}\n",
            "hand-editable, version only"
        );
        assert_eq!(read(&dir), Some(Sidecar::default()));
        assert!(std::fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(|e| e.ok())
            .all(|e| !e.file_name().to_string_lossy().contains("tmp")));
    }

    #[test]
    fn writes_only_when_the_bytes_change() {
        let dir = temp_dir("idempotent");
        let mut full = Sidecar::default();
        full.albums.insert(
            "al-1".into(),
            state(Some(["ff8800", "0044cc"]), Some(["111111", "222222"])),
        );
        assert!(write(&dir, &full).expect("first write"));
        assert!(!write(&dir, &full).expect("second write"), "no churn");
        let mut other = full.clone();
        other.albums.insert(
            "al-1".into(),
            state(Some(["010101", "020202"]), Some(["111111", "222222"])),
        );
        assert!(write(&dir, &other).expect("changed write"));
        assert_eq!(read(&dir).expect("read"), other);
    }

    #[test]
    fn malformed_file_reads_as_none_without_panicking() {
        let dir = temp_dir("malformed");
        std::fs::write(path_in(&dir), "{ this is not json").expect("seed");
        assert!(read(&dir).is_none());
        assert!(write(&dir, &Sidecar::default()).expect("write"));
        assert_eq!(read(&dir), Some(Sidecar::default()));
    }

    #[test]
    fn reads_entries_and_ignores_absurd_ones() {
        let dir = temp_dir("entries");
        std::fs::write(
            path_in(&dir),
            "{\n  \"version\": 1,\n  \"albums\": {\n    \"al-1\": {\"gradient\": [\"ff0000\", \"00ff00\"]},\n    \"al-2\": {}\n  }\n}\n",
        )
        .expect("seed");
        let got = read(&dir).expect("read");
        assert_eq!(got.albums.len(), 2);
        assert_eq!(
            got.albums.get("al-1").unwrap().gradient,
            Some(pair("ff0000", "00ff00"))
        );
        assert_eq!(got.albums.get("al-1").unwrap().colors, None);
        assert_eq!(got.albums.get("al-2"), Some(&AlbumState::default()));
    }

    #[test]
    fn recognises_itself_and_its_temp() {
        let dir = temp_dir("paths");
        assert!(is_sidecar_path(&path_in(&dir)));
        assert!(is_sidecar_path(&dir.join(".songstress.json.tmp-1234")));
        assert!(!is_sidecar_path(&dir.join("01 - Song.flac")));
        assert!(!is_sidecar_path(&dir.join("songstress.json")));
    }

    /// The in-app write path must NOT let the file talk back — the bug this
    /// pins down: setting a gradient wrote the new value to the DB and then
    /// re-adopted the old one out of the file, in both places.
    #[test]
    fn sync_one_mirrors_the_db_and_never_re_adopts() {
        use crate::library::db;

        let root = temp_dir("sync-one");
        let album_dir = root.join("Aimer/DAWN");
        std::fs::create_dir_all(&album_dir).expect("mkdir");
        let conn = db::open(&root.join("t.db")).expect("open");
        conn.execute_batch(
            "INSERT INTO artists VALUES ('ar-1','Aimer','aimer');
             INSERT INTO albums(id, artist_id, title, year, color_c1, color_c2)
             VALUES ('al-1','ar-1','DAWN',2015,'cee4de','086ba0');",
        )
        .expect("seed");
        conn.execute(
            "INSERT INTO tracks(id, album_id, title, duration_sec, path, mtime_ns, size)
             VALUES ('tr-1','al-1','x',1.0,?1,1,1)",
            [album_dir.join("01.flac").to_string_lossy()],
        )
        .expect("track");
        // The file holds an OLD override; the user has just set a new one.
        let mut seeded = Sidecar::default();
        seeded
            .albums
            .insert("al-1".into(), state(None, Some(["cee4de", "086ba0"])));
        write(&album_dir, &seeded).expect("seed");
        let fresh = pair("112233", "445566");
        settings::set_album_gradient(&conn, "al-1", Some(&fresh)).expect("set");

        assert!(
            sync_one(&conn, "al-1").expect("sync"),
            "the file must change"
        );
        let got = read(&album_dir).expect("read");
        assert_eq!(
            got.albums.get("al-1").unwrap().gradient,
            Some(fresh.clone())
        );
        assert_eq!(
            got.albums.get("al-1").unwrap().colors,
            Some(pair("cee4de", "086ba0")),
            "the artwork's pair is mirrored"
        );
        assert_eq!(settings::album_gradient(&conn, "al-1"), Some(fresh));

        // Clearing drops the key without touching the rest of the entry.
        settings::set_album_gradient(&conn, "al-1", None).expect("clear");
        assert!(sync_one(&conn, "al-1").expect("sync"));
        let cleared = read(&album_dir).expect("read");
        assert_eq!(cleared.albums.get("al-1").unwrap().gradient, None);
        assert_eq!(
            cleared.albums.get("al-1").unwrap().colors,
            Some(pair("cee4de", "086ba0"))
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The whole pass, against a DB and folders that look real: a gradient wins
    /// from the file, colours fill in only where the artwork has none, a SHARED
    /// folder keeps one entry per album, and a stale entry is pruned.
    #[test]
    fn sync_adopts_the_file_and_mirrors_the_result() {
        use crate::library::db;

        let root = temp_dir("sync");
        let solo = root.join("Ghost/Meliora (2015)");
        let shared = root.join("Compilations/Proyecto Anison Latino");
        std::fs::create_dir_all(&solo).expect("mkdir");
        std::fs::create_dir_all(&shared).expect("mkdir");
        let conn = db::open(&root.join("t.db")).expect("open");
        conn.execute_batch(
            "INSERT INTO artists VALUES ('ar-1','Ghost','ghost');
             INSERT INTO albums(id, artist_id, title, year, color_c1, color_c2)
             VALUES ('al-solo','ar-1','Meliora',2015,'aabbcc','ddeeff');
             INSERT INTO albums(id, artist_id, title, year, color_c1, color_c2)
             VALUES ('al-artless','ar-1','Artless',2016,NULL,NULL);
             INSERT INTO albums(id, artist_id, title, year, color_c1, color_c2)
             VALUES ('al-shared-a','ar-1','Proyecto',2020,'111213','212223');
             INSERT INTO albums(id, artist_id, title, year, color_c1, color_c2)
             VALUES ('al-shared-b','ar-1','Anison',2021,'313233','414243');",
        )
        .expect("seed");
        let track = |id: &str, album: &str, dir: &Path| {
            conn.execute(
                "INSERT INTO tracks(id, album_id, title, duration_sec, path, mtime_ns, size)
                 VALUES (?1, ?2, 'x', 1.0, ?3, 1, 1)",
                rusqlite::params![id, album, dir.join(format!("{id}.flac")).to_string_lossy()],
            )
            .expect("track");
        };
        track("tr-solo", "al-solo", &solo);
        track("tr-artless", "al-artless", &solo);
        track("tr-shared-a", "al-shared-a", &shared);
        track("tr-shared-b", "al-shared-b", &shared);

        // The files: gradients for the two solo albums and one of the shared
        // pair; colours only where the DB has none.
        let mut seeded = Sidecar::default();
        seeded.albums.insert(
            "al-solo".into(),
            state(Some(["111111", "222222"]), Some(["ff8800", "0044cc"])),
        );
        seeded
            .albums
            .insert("al-artless".into(), state(Some(["333333", "444444"]), None));
        // Plus an entry for an album that no longer lives here.
        seeded
            .albums
            .insert("al-gone".into(), state(None, Some(["999999", "888888"])));
        write(&solo, &seeded).expect("seed solo");
        let mut shared_file = Sidecar::default();
        shared_file.albums.insert(
            "al-shared-a".into(),
            state(None, Some(["ff8800", "0044cc"])),
        );
        write(&shared, &shared_file).expect("seed shared");

        let first = sync_all(&conn).expect("sync");
        assert_eq!(first.albums, 4);
        assert_eq!(
            first.adopted_gradient, 2,
            "al-solo's and al-shared-a's gradients were both new to the DB"
        );
        assert_eq!(first.adopted_colors, 1, "only the artless album falls back");
        assert_eq!(first.written, 2, "both folders change");

        let solo_file = read(&solo).expect("read solo");
        assert_eq!(
            solo_file.albums.get("al-solo").unwrap().colors,
            Some(pair("aabbcc", "ddeeff")),
            "the artwork's pair is the mirror, NOT the file's guess"
        );
        assert_eq!(
            solo_file.albums.get("al-solo").unwrap().gradient,
            Some(pair("ff8800", "0044cc"))
        );
        assert_eq!(
            solo_file.albums.get("al-artless").unwrap().colors,
            Some(pair("333333", "444444")),
            "with no artwork to derive from, the file is what keeps the look"
        );
        assert!(
            !solo_file.albums.contains_key("al-gone"),
            "an entry whose album is gone is pruned"
        );

        let shared_file = read(&shared).expect("read shared");
        assert_eq!(
            shared_file.albums.len(),
            2,
            "one folder, two albums, two entries: {:#?}",
            shared_file.albums
        );
        assert_eq!(
            shared_file.albums.get("al-shared-a").unwrap().gradient,
            Some(pair("ff8800", "0044cc")),
            "the album whose file carried a gradient kept it"
        );
        assert_eq!(
            shared_file.albums.get("al-shared-b").unwrap().colors,
            Some(pair("313233", "414243")),
            "…and its neighbour is mirrored, not overwritten"
        );
        assert_eq!(
            settings::album_gradient(&conn, "al-shared-a"),
            Some(pair("ff8800", "0044cc"))
        );
        let (c1, c2): (String, String) = conn
            .query_row(
                "SELECT color_c1, color_c2 FROM albums WHERE id = 'al-artless'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("row");
        assert_eq!((c1.as_str(), c2.as_str()), ("333333", "444444"));

        // Second run: nothing left to adopt, nothing to rewrite.
        let second = sync_all(&conn).expect("sync again");
        assert_eq!(
            (
                second.adopted_gradient,
                second.adopted_colors,
                second.written
            ),
            (0, 0, 0),
            "idempotent: a scan must not churn files"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
