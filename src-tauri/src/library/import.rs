//! Import staging (PLAN.md Step 2a): a two-step flow where importing COPIES
//! files into a staging area under the cache dir (playable immediately, kept
//! until the user saves them into the library dir or discards them).
//!
//! Staging layout mirrors the save layout: `<import>/<sourceFolderName>/…`,
//! so "save to library" is a straight re-rooting of the relative path into
//! `<musicDir>/`. The scanner walks the import root as a second root; the
//! frontend learns which albums/tracks are staged via path prefixes in the
//! library dump (no schema change).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use super::scan;

/// The import staging root under the app cache dir.
pub fn import_dir(cache_dir: &Path) -> PathBuf {
    cache_dir.join("import")
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct ImportCounts {
    pub copied: usize,
    pub skipped: usize,
}

/// (lowercased file name, size) index of music files we already hold —
/// library DB rows plus anything already staged. Importing skips sources
/// found here, so "add file then add its folder" (or re-importing from a
/// differently-named folder) can never produce duplicate tracks.
pub struct KnownFiles(pub HashSet<(String, u64)>);

impl KnownFiles {
    #[cfg(test)]
    pub fn empty() -> Self {
        KnownFiles(HashSet::new())
    }

    pub fn from_db(conn: &Connection) -> Self {
        let mut set = HashSet::new();
        let Ok(mut stmt) = conn.prepare("SELECT path, size FROM tracks") else {
            return KnownFiles(set);
        };
        if let Ok(rows) = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        }) {
            for row in rows.flatten() {
                // Stale rows (file deleted outside the app) must not block
                // re-importing that file.
                if !Path::new(&row.0).exists() {
                    continue;
                }
                set.insert(Self::key(&row.0, row.1));
            }
        }
        KnownFiles(set)
    }

    pub fn from_dir(dir: &Path) -> Self {
        let mut set = HashSet::new();
        for entry in walkdir::WalkDir::new(dir).follow_links(false) {
            let Ok(entry) = entry else { continue };
            if !entry.file_type().is_file() {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                set.insert(Self::key(&entry.path().to_string_lossy(), meta.len() as i64));
            }
        }
        KnownFiles(set)
    }

    fn key(path: &str, size: i64) -> (String, u64) {
        let name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        (name, size.max(0) as u64)
    }

    pub fn contains(&self, path: &Path) -> bool {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        self.0.contains(&(name, size))
    }
}

/// Copy files/folders into staging. Directories keep their name and internal
/// structure; loose files land under their parent folder's name. Existing
/// identical files are skipped; same-name different-content files get a
/// " (2)"-style suffix — never overwritten. Sources present in `known`
/// (already in the library or staged) are skipped outright.
pub fn import_paths(
    cache_dir: &Path,
    paths: &[PathBuf],
    known: &KnownFiles,
    mut progress: impl FnMut(usize, usize),
) -> Result<ImportCounts, String> {
    let root = import_dir(cache_dir);
    std::fs::create_dir_all(&root).map_err(|e| format!("create import dir: {e}"))?;

    // Plan (src, dest) pairs first so progress totals are known upfront.
    let mut plan: Vec<(PathBuf, PathBuf)> = Vec::new();
    for path in paths {
        if path.is_dir() {
            let folder = folder_name(path);
            for entry in walkdir::WalkDir::new(path).follow_links(false) {
                let entry = entry.map_err(|e| format!("walk {:?}: {e}", path))?;
                if entry.file_type().is_file() && scan::is_supported(entry.path()) {
                    if known.contains(entry.path()) {
                        continue;
                    }
                    let rel = entry
                        .path()
                        .strip_prefix(path)
                        .map_err(|e| e.to_string())?;
                    plan.push((entry.path().to_path_buf(), root.join(&folder).join(rel)));
                }
            }
        } else if path.is_file() && scan::is_supported(path) && !known.contains(path) {
            let folder = path
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Imported".into());
            let name = path
                .file_name()
                .map(|n| n.to_os_string())
                .unwrap_or_default();
            plan.push((path.clone(), root.join(folder).join(name)));
        }
    }

    let mut counts = ImportCounts::default();
    let total = plan.len();
    for (i, (src, dest)) in plan.into_iter().enumerate() {
        progress(i, total);
        match resolve_collision(&src, &dest)? {
            Some(final_dest) => {
                if let Some(parent) = final_dest.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("mkdir {parent:?}: {e}"))?;
                }
                std::fs::copy(&src, &final_dest).map_err(|e| format!("copy {src:?}: {e}"))?;
                counts.copied += 1;
            }
            None => counts.skipped += 1,
        }
    }
    progress(total, total);
    Ok(counts)
}

/// Tracks currently staged (path under the import root), optionally scoped
/// to one album or a single track. Empty when the import dir was never
/// created.
pub fn staged_tracks(
    conn: &Connection,
    cache_dir: &Path,
    album_id: Option<&str>,
    track_id: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    let root = import_dir(cache_dir);
    let mut sql = String::from("SELECT path FROM tracks");
    let mut clauses: Vec<&str> = Vec::new();
    if album_id.is_some() {
        clauses.push("album_id = ?1");
    }
    if track_id.is_some() {
        clauses.push(if album_id.is_some() { "id = ?2" } else { "id = ?1" });
    }
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let q = |r: &rusqlite::Row| r.get::<_, String>(0);
    let rows = match (album_id, track_id) {
        (Some(a), Some(t)) => stmt.query_map([a, t], q),
        (Some(a), None) => stmt.query_map([a], q),
        (None, Some(t)) => stmt.query_map([t], q),
        (None, None) => stmt.query_map([], q),
    }
    .map_err(|e| e.to_string())?;
    Ok(rows
        .filter_map(|r| r.ok())
        .map(PathBuf::from)
        .filter(|p| p.starts_with(&root))
        .collect())
}

/// Copy staged tracks into the library dir as
/// `<musicDir>/<Artist>/<Album>/<files>` (user-decided layout; the staging
/// source-folder component is dropped, any deeper structure is kept), then
/// delete the staged originals. Same-name-same-size files already in the
/// library are skipped (dedupe); different content gets a suffix.
/// Returns the number of files copied into the library.
pub fn save_to_library(
    conn: &Connection,
    cache_dir: &Path,
    music_dir: &Path,
    staged: &[PathBuf],
    mut progress: impl FnMut(usize, usize),
) -> Result<usize, String> {
    let root = import_dir(cache_dir);
    let total = staged.len();
    let mut copied = 0usize;
    for (i, src) in staged.iter().enumerate() {
        progress(i, total);
        let (artist, album) = album_of(conn, src)?;
        let rel = src
            .strip_prefix(&root)
            .map_err(|e| format!("{src:?} is not staged: {e}"))?
            .components()
            .skip(1) // drop the staging source-folder component
            .collect::<PathBuf>();
        if rel.as_os_str().is_empty() {
            return Err(format!("{src:?}: unexpected staging layout"));
        }
        let dest = music_dir
            .join(sanitize_path(&artist))
            .join(sanitize_path(&album))
            .join(rel);
        if let Some(final_dest) = resolve_collision(src, &dest)? {
            if let Some(parent) = final_dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {parent:?}: {e}"))?;
            }
            std::fs::copy(src, &final_dest).map_err(|e| format!("copy {src:?}: {e}"))?;
            copied += 1;
        }
    }
    progress(total, total);
    let refs: Vec<&Path> = staged.iter().map(PathBuf::as_path).collect();
    remove_staged(&refs)?;
    Ok(copied)
}

/// (artist name, album title) for the album a staged track belongs to.
fn album_of(conn: &Connection, track_path: &Path) -> Result<(String, String), String> {
    conn.query_row(
        "SELECT ar.name, al.title FROM tracks t
         JOIN albums al ON al.id = t.album_id
         JOIN artists ar ON ar.id = al.artist_id
         WHERE t.path = ?1",
        [track_path.to_string_lossy().as_ref()],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )
    .map_err(|e| format!("no library row for {track_path:?} (rescan needed?): {e}"))
}

/// Path component sanitizer: '/' and NUL would escape the folder.
pub fn sanitize_path(name: &str) -> String {
    name.trim().replace(['/', '\0'], "_")
}

/// Delete staged files (and any staging folders left empty).
pub fn discard(staged: &[PathBuf]) -> Result<usize, String> {
    let n = staged.len();
    let refs: Vec<&Path> = staged.iter().map(PathBuf::as_path).collect();
    remove_staged(&refs)?;
    Ok(n)
}

fn remove_staged(staged: &[&Path]) -> Result<(), String> {
    let mut parents: Vec<PathBuf> = Vec::new();
    for p in staged {
        match std::fs::remove_file(p) {
            Ok(()) => {
                if let Some(parent) = p.parent() {
                    parents.push(parent.to_path_buf());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("remove {p:?}: {e}")),
        }
    }
    // Prune now-empty staging dirs, deepest first.
    parents.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    for dir in parents {
        let _ = std::fs::remove_dir(&dir); // fails harmlessly when non-empty
    }
    Ok(())
}

/// Collision rule for copying `src` onto `dest`:
/// * dest missing ⇒ Some(dest) — plain copy;
/// * dest exists with the SAME size as src ⇒ None — already imported, skip;
/// * dest exists, different size ⇒ Some(" (N)"-suffixed sibling) — never
///   overwrite user files.
pub fn resolve_collision(src: &Path, dest: &Path) -> Result<Option<PathBuf>, String> {
    let src_size = std::fs::metadata(src)
        .map_err(|e| format!("stat {src:?}: {e}"))?
        .len();
    let dest_size = match std::fs::metadata(dest) {
        Ok(m) => m.len(),
        Err(_) => return Ok(Some(dest.to_path_buf())),
    };
    if dest_size == src_size {
        return Ok(None);
    }
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    let parent = dest.parent().unwrap_or(Path::new("."));
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s.to_string(), Some(e.to_string())),
        _ => (name.clone(), None),
    };
    for n in 2..10_000u32 {
        let cand = match &ext {
            Some(e) => parent.join(format!("{stem} ({n}).{e}")),
            None => parent.join(format!("{stem} ({n})")),
        };
        match std::fs::metadata(&cand) {
            Err(_) => return Ok(Some(cand)), // free
            Ok(m) if m.len() == src_size => return Ok(None), // already imported under a suffix
            Ok(_) => continue,
        }
    }
    Err(format!("no free suffixed name for {dest:?}"))
}

fn folder_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Imported".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("songstress-import-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    fn tiny_mp3(path: &Path, filler: u8) {
        // Two MPEG-1 Layer III frames (one fails lofty validation) + a filler
        // byte so different "editions" of a file have different sizes.
        let mut frame = vec![0xFFu8, 0xFB, 0x90, 0x00];
        frame.resize(417, filler);
        let mut data = frame.clone();
        data.extend_from_slice(&frame);
        // Filler also varies the LENGTH so size-based dedupe can tell
        // "editions" apart in the collision tests.
        data.extend(std::iter::repeat(filler).take(filler as usize));
        std::fs::write(path, data).expect("write mp3");
    }

    #[test]
    fn import_folder_and_loose_files_stage_correctly() {
        let root = temp_dir("layout");
        let cache = root.join("cache");
        let src = root.join("source");
        std::fs::create_dir_all(src.join("Cool Album")).unwrap();
        tiny_mp3(&src.join("Cool Album/01 - A.mp3"), 1);
        tiny_mp3(&src.join("Cool Album/02 - B.mp3"), 2);
        tiny_mp3(&src.join("loose.mp3"), 3);

        let counts = import_paths(
            &cache,
            &[src.join("Cool Album"), src.join("loose.mp3")],
            &KnownFiles::empty(),
            |_, _| {},
        )
        .unwrap();
        assert_eq!(counts.copied, 3, "two album files + one loose");
        assert!(cache.join("import/Cool Album/01 - A.mp3").is_file());
        assert!(cache.join("import/Cool Album/02 - B.mp3").is_file());
        // Loose file lands under its parent folder's name.
        assert!(cache.join("import/source/loose.mp3").is_file());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn import_collision_skips_identical_and_suffixes_different() {
        let root = temp_dir("collision");
        let cache = root.join("cache");
        let src = root.join("source");
        std::fs::create_dir_all(&src).unwrap();
        tiny_mp3(&src.join("song.mp3"), 1);

        import_paths(&cache, &[src.join("song.mp3")], &KnownFiles::empty(), |_, _| {}).unwrap();
        let staged = cache.join("import/source/song.mp3");

        // Same content again → skipped, no overwrite, no extra file.
        let counts = import_paths(&cache, &[src.join("song.mp3")], &KnownFiles::empty(), |_, _| {}).unwrap();
        assert_eq!((counts.copied, counts.skipped), (0, 1));
        assert_eq!(std::fs::read_dir(staged.parent().unwrap()).unwrap().count(), 1);

        // Different content, same name → " (2)" sibling, original untouched.
        tiny_mp3(&src.join("song.mp3"), 9);
        import_paths(&cache, &[src.join("song.mp3")], &KnownFiles::empty(), |_, _| {}).unwrap();
        assert!(staged.is_file(), "original kept");
        assert!(cache.join("import/source/song (2).mp3").is_file());
        let _ = std::fs::remove_dir_all(&root);
    }

    fn scanned_db(root: &Path) -> Connection {
        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        crate::library::scan::run_scan_roots(&mut conn, &[root.to_path_buf()], |_, _| {}, false)
            .expect("scan");
        conn
    }

    #[test]
    fn save_moves_staged_into_library_and_dedupes() {
        let root = temp_dir("save");
        let cache = root.join("cache");
        let music = root.join("music");
        let src = root.join("source");
        std::fs::create_dir_all(&music).unwrap();
        std::fs::create_dir_all(&src).unwrap();
        tiny_mp3(&src.join("song.mp3"), 1);
        import_paths(&cache, &[src.join("song.mp3")], &KnownFiles::empty(), |_, _| {}).unwrap();

        // Scan BOTH roots so the staged copy is a known track.
        let mut conn = scanned_db(&root);
        crate::library::scan::run_scan_roots(
            &mut conn,
            &[music.clone(), import_dir(&cache)],
            |_, _| {},
            false,
        )
        .unwrap();
        let staged = staged_tracks(&conn, &cache, None, None).unwrap();
        assert_eq!(staged.len(), 1);

        // Save: copies to <music>/<Artist>/<Album>/song.mp3, removes staged.
        // (Untagged tiny_mp3 → scanner placeholders.)
        let copied = save_to_library(&conn, &cache, &music, &staged, |_, _| {}).unwrap();
        assert_eq!(copied, 1);
        assert!(music.join("Unknown Artist/Unknown Album/song.mp3").is_file());
        assert!(!staged[0].exists(), "staged original removed");
        assert!(!import_dir(&cache).join("source").exists(), "empty folder pruned");

        // Saving the same content again (re-import) dedupes by size.
        tiny_mp3(&src.join("song.mp3"), 1);
        import_paths(&cache, &[src.join("song.mp3")], &KnownFiles::empty(), |_, _| {}).unwrap();
        let mut conn = scanned_db(&root);
        crate::library::scan::run_scan_roots(
            &mut conn,
            &[music.clone(), import_dir(&cache)],
            |_, _| {},
            false,
        )
        .unwrap();
        let staged = staged_tracks(&conn, &cache, None, None).unwrap();
        let copied = save_to_library(&conn, &cache, &music, &staged, |_, _| {}).unwrap();
        assert_eq!(copied, 0, "identical library file → deduped, nothing copied");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The user-reported dup: add a file, then add a folder that contains it
    /// (under a different staging folder name) — the known-set must skip it.
    #[test]
    fn folder_reimport_of_staged_file_is_skipped() {
        let root = temp_dir("reimport");
        let cache = root.join("cache");
        let src = root.join("source/Impera");
        std::fs::create_dir_all(&src).unwrap();
        tiny_mp3(&src.join("01 - Imperium.mp3"), 1);
        tiny_mp3(&src.join("02 - Kaisarion.mp3"), 2);

        // Add ONE file, then make it "known" (as the DB/staging would be).
        import_paths(
            &cache,
            &[src.join("01 - Imperium.mp3")],
            &KnownFiles::empty(),
            |_, _| {},
        )
        .unwrap();
        let known = KnownFiles::from_dir(&import_dir(&cache));

        // Add the whole folder: only the NOT-yet-held file may be copied.
        let counts = import_paths(&cache, &[src], &known, |_, _| {}).unwrap();
        assert_eq!(counts.copied, 1, "only Kaisarion is new");
        let staged: Vec<PathBuf> = walkdir::WalkDir::new(import_dir(&cache))
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect();
        assert_eq!(staged.len(), 2, "Imperium never duplicated");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn discard_removes_files_and_prunes_dirs() {
        let root = temp_dir("discard");
        let cache = root.join("cache");
        let src = root.join("source");
        std::fs::create_dir_all(src.join("Album")).unwrap();
        tiny_mp3(&src.join("Album/01.mp3"), 1);
        tiny_mp3(&src.join("Album/02.mp3"), 2);
        import_paths(&cache, &[src.join("Album")], &KnownFiles::empty(), |_, _| {}).unwrap();

        let staged: Vec<PathBuf> = walkdir::WalkDir::new(import_dir(&cache))
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect();
        assert_eq!(staged.len(), 2);

        assert_eq!(discard(&staged).unwrap(), 2);
        assert!(!import_dir(&cache).join("Album").exists(), "pruned");
        assert!(src.join("Album/01.mp3").is_file(), "source untouched");
        let _ = std::fs::remove_dir_all(&root);
    }
}
