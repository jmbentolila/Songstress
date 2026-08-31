//! Import staging (PLAN.md Step 2a): a two-step flow where importing COPIES
//! files into a staging area under the cache dir (playable immediately, kept
//! until the user saves them into the library dir or discards them).
//!
//! Staging layout mirrors the save layout: `<import>/<sourceFolderName>/…`,
//! so "save to library" is a straight re-rooting of the relative path into
//! `<musicDir>/`. The scanner walks the import root as a second root; the
//! frontend learns which albums/tracks are staged via path prefixes in the
//! library dump (no schema change).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

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

/// Where one staged album's files belong.
///
/// Decided from the library's own contents, never from a `<Artist>/<Album>`
/// template: a template assumes the collection is arranged the way the code
/// wishes it were. This user's albums live under `<root>/Music Files/<Artist>/…`,
/// so rebuilding the path sent every imported album to a second regime at the
/// root's top level — `Music/Ghost/Impera` next to `Music/Music Files/Ghost/
/// Prequelle` — which splits one artist's catalogue across the tree. The rules,
/// in order of tag specificity: the same album already on disk wins, then the
/// folder already holding that artist's albums, then the plain convention under
/// the primary root for a genuinely new artist.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Destination {
    pub folder: PathBuf,
    /// Which rule chose the folder, so the UI can say so rather than leaving the
    /// user to infer it from a path: "merge" | "artist" | "new".
    pub rule: &'static str,
    /// The album this one joins, for "merge".
    pub merged_into: Option<String>,
}

fn dirname(p: &Path) -> Option<PathBuf> {
    p.parent().map(|d| d.to_path_buf())
}

fn under_any(p: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|r| p.starts_with(r))
}

fn norm_title(s: &str) -> String {
    s.trim().to_lowercase()
}

/// (artist id, artist name, album title) of an album row.
pub fn album_identity(conn: &Connection, album_id: &str) -> Result<(String, String, String), String> {
    conn.query_row(
        "SELECT al.artist_id, ar.name, al.title FROM albums al
         JOIN artists ar ON ar.id = al.artist_id WHERE al.id = ?1",
        [album_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )
    .map_err(|e| format!("album {album_id}: {e}"))
}

fn album_id_of_path(conn: &Connection, path: &Path) -> Result<String, String> {
    conn.query_row(
        "SELECT album_id FROM tracks WHERE path = ?1",
        [path.to_string_lossy().as_ref()],
        |r| r.get::<_, String>(0),
    )
    .map_err(|e| format!("no library row for {path:?} (rescan needed?): {e}"))
}

/// Destination for a staged album, given what the library holds right now.
/// `roots` are the configured music dirs WITHOUT the staging area: files there
/// are copies, and merging a staged album into a path that is about to be
/// deleted is how an album ends up pointing at nothing.
pub fn album_destination(
    conn: &Connection,
    roots: &[PathBuf],
    music_dir: &Path,
    staging_root: &Path,
    album_id: &str,
) -> Result<Destination, String> {
    let (artist_id, artist, title) = album_identity(conn, album_id)?;
    // NOTE: deliberately NOT `AND al.id != ?album_id`. Staged files are folded
    // into the album row their tags name, so the album being saved is usually the
    // SAME row as the one already in the library — excluding it hides the very
    // folder to merge into, and rule 2 then creates a sibling: that is how
    // `03. Spillways.mp3` ended up alone in `Music Files/Ghost/Impera` while the
    // other ten files of that album sat in `Music/Ghost/Impera`. The staging-path
    // filter below is what keeps staged copies out of the running, and it is
    // enough.
    let mut stmt = conn
        .prepare(
            "SELECT al.title, t.path FROM tracks t
             JOIN albums al ON al.id = t.album_id
             WHERE al.artist_id = ?1",
        )
        .map_err(|e| format!("prepare: {e}"))?;
    let rows: Vec<(String, PathBuf)> = stmt
        .query_map(params![artist_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                PathBuf::from(r.get::<_, String>(1)?),
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter(|(_, p)| !p.starts_with(staging_root))
        .collect();

    // 1. This album is already on disk: join its folder, whatever that folder is
    //    called. Album identity is album artist + title; year is not part of the
    //    match, because the folder — not the DB row — is what an album IS on
    //    disk, and the scan that follows a save collapses any year disagreement
    //    into one folder consensus anyway.
    let mut merge: HashMap<PathBuf, usize> = HashMap::new();
    for (_t, p) in rows.iter().filter(|(t, _)| norm_title(t) == norm_title(&title)) {
        if let Some(d) = dirname(p).filter(|d| under_any(d, roots)) {
            *merge.entry(d).or_default() += 1;
        }
    }
    if let Some((folder, _)) = merge.into_iter().max_by_key(|(_, n)| *n) {
        return Ok(Destination {
            folder,
            rule: "merge",
            merged_into: Some(title),
        });
    }

    // 2. An artist we already have: join the folder holding most of their albums,
    //    so new work lands beside the work it belongs to. Multi-disc albums are
    //    one folder here too — the disc tag, not the directory, is what splits
    //    an album in the UI (Ghostlights: 36 files, three discs, flat).
    let mut artist_folders: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();
    for (_, p) in rows.iter() {
        let Some(album_dir) = dirname(p).filter(|d| under_any(d, roots)) else {
            continue;
        };
        if let Some(parent) = dirname(&album_dir) {
            artist_folders.entry(parent).or_default().insert(album_dir);
        }
    }
    if let Some((folder, _)) = artist_folders.into_iter().max_by_key(|(_, dirs)| dirs.len()) {
        return Ok(Destination {
            folder: folder.join(sanitize_path(&title)),
            rule: "artist",
            merged_into: None,
        });
    }

    // 3. A new artist: a new artist folder inside the folder the collection
    //    already lives in — not blindly at the primary root. The distinction is
    //    not academic here: most of this library sits one level below its root
    //    (248 albums under `<root>/Music Files/`), so the plain convention would
    //    have started a second regime for every artist that was not already
    //    indexed. Empty library → the primary root, which is the only sane
    //    answer when there is no collection to learn from.
    let base = collection_parent(conn, roots, staging_root)?.unwrap_or_else(|| music_dir.to_path_buf());
    Ok(Destination {
        folder: base
            .join(sanitize_path(&artist))
            .join(sanitize_path(&title)),
        rule: "new",
        merged_into: None,
    })
}

/// The folder holding more of the library's albums than any other — the
/// collection's de-facto home, whatever depth the root happens to organize it at
/// (`<root>/Music Files/` here, with 248 of 250 albums, against a root the two
/// old imports left bare). Counted by distinct album; ties break by shallowest
/// path, then lexicographically, so the answer is deterministic rather than
/// whatever the query planner returned.
fn collection_parent(
    conn: &Connection,
    roots: &[PathBuf],
    staging_root: &Path,
) -> Result<Option<PathBuf>, String> {
    let mut by_parent: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT album_id, path FROM tracks")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            let path = PathBuf::from(&row.1);
            if path.starts_with(staging_root) {
                continue;
            }
            let Some(album_dir) = dirname(&path).filter(|d| under_any(d, roots)) else {
                continue;
            };
            // Three levels, because that is what `<home>/<Artist>/<Album>/<file>`
            // means: file → album folder → artist folder → the folder holding the
            // artists. A library with albums directly under the root therefore
            // yields the root itself, which is the plain convention. An album
            // whose audio nests one level deeper — `Blind Guardian/Legacy Of The
            // Dark Lands/CD1, Album/` in this library, 68 files, scans kept in
            // the folder above — votes for its artist folder instead of the
            // collection home. It is one album out of 249, so a tally by
            // distinct album is never going to mistake that for the majority,
            // which is why the count is by album and not by file.
            let Some(artist_dir) = dirname(&album_dir) else { continue };
            let Some(home) = dirname(&artist_dir) else { continue };
            by_parent.entry(home).or_default().insert(row.0);
        }
    }
    // (albums, inverse depth so shallower wins, path so ties are deterministic)
    let key = |parent: &PathBuf, albums: &HashSet<String>| {
        (
            albums.len(),
            usize::MAX - parent.components().count(),
            parent.clone(),
        )
    };
    let best = by_parent
        .iter()
        .max_by_key(|(parent, albums)| key(parent, albums));
    Ok(best.map(|(parent, _)| parent.clone()))
}

/// Every staged album, with the folder its files would move to. The modal shows
/// this before anything is committed: the destination is what the decision is
/// actually about, and "2 albums staged" is not a thing a user can decide from.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StagedTrack {
    pub id: String,
    pub title: String,
    pub duration_sec: f64,
    pub disc: i64,
    pub track: Option<i64>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StagedAlbum {
    pub album_id: String,
    pub artist: String,
    pub title: String,
    pub year: Option<i64>,
    /// The staged files, in the order they will be listed: disc, then track
    /// number. The modal expands an album to show these, because "Ghost —
    /// Impera, 11 tracks" is a decision about an album, and the thing being
    /// decided on is the album.
    pub tracks: Vec<StagedTrack>,
    pub destination: Destination,
}


pub fn staged_plan(
    conn: &Connection,
    roots: &[PathBuf],
    music_dir: &Path,
    staging_root: &Path,
) -> Result<Vec<StagedAlbum>, String> {
    let mut by_album: Vec<(String, usize)> = {
        let mut stmt = conn
            .prepare("SELECT album_id, path FROM tracks")
            .map_err(|e| e.to_string())?;
        let mut counts: HashMap<String, usize> = HashMap::new();
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            if Path::new(&row.1).starts_with(staging_root) {
                *counts.entry(row.0).or_default() += 1;
            }
        }
        counts.into_iter().collect()
    };
    by_album.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = Vec::with_capacity(by_album.len());
    for (album_id, _) in by_album {
        let (_, artist, title) = album_identity(conn, &album_id)?;
        let year: Option<i64> = conn
            .query_row("SELECT year FROM albums WHERE id = ?1", [&album_id], |r| {
                r.get(0)
            })
            .unwrap_or(None);
        // The staged files of that album — an album row can span both regimes
        // (a re-import of an indexed album), and what is being decided on is the
        // staged half. Prefix-matched in Rust rather than with LIKE, because a
        // path is not a LIKE pattern and the counting pass above already does it
        // this way; two rules for one concept would drift.
        let mut stmt = conn
            .prepare(
                "SELECT id, title, duration_sec, disc, track, path FROM tracks
                 WHERE album_id = ?1 ORDER BY disc, track, title",
            )
            .map_err(|e| e.to_string())?;
        let tracks = stmt
            .query_map([album_id.clone()], |r| {
                Ok((
                    StagedTrack {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        duration_sec: r.get(2)?,
                        disc: r.get(3)?,
                        track: r.get(4)?,
                    },
                    r.get::<_, String>(5)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .flatten()
            .filter(|(_, path)| Path::new(path).starts_with(staging_root))
            .map(|(t, _)| t)
            .collect();
        out.push(StagedAlbum {
            destination: album_destination(conn, roots, music_dir, staging_root, &album_id)?,
            tracks,
            album_id,
            artist,
            title,
            year,
        });
    }
    Ok(out)
}

/// What `plan_move` decided about one file.
enum MovePlan {
    Place(PathBuf),
    AlreadyHave,
}

fn content_hash(p: &Path) -> Result<blake3::Hash, String> {
    use std::io::Read;
    let mut f = std::io::BufReader::new(
        std::fs::File::open(p).map_err(|e| format!("open {p:?}: {e}"))?,
    );
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf).map_err(|e| format!("read {p:?}: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize())
}

/// Where a file being MOVED may go, and whether the library already holds it.
///
/// `resolve_collision`'s size-only rule is right for staging, where a wrong call
/// costs a redundant copy. Here it would cost the user a file: with a move,
/// "same name, same size" means deleting their file on the strength of its byte
/// count. So equality is name + size + content hash; a same-name file with
/// different bytes gets a suffix, exactly as before.
fn plan_move(src: &Path, dest: &Path) -> Result<MovePlan, String> {
    let src_size = std::fs::metadata(src)
        .map_err(|e| format!("stat {src:?}: {e}"))?
        .len();
    let mut src_hash: Option<blake3::Hash> = None;
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    let parent = dest.parent().unwrap_or(Path::new("."));
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s.to_string(), Some(e.to_string())),
        _ => (name.clone(), None),
    };
    for n in 0..10_000u32 {
        let cand = if n == 0 {
            dest.to_path_buf()
        } else {
            match &ext {
                Some(e) => parent.join(format!("{stem} ({n}).{e}")),
                None => parent.join(format!("{stem} ({n})")),
            }
        };
        match std::fs::metadata(&cand) {
            Err(_) => return Ok(MovePlan::Place(cand)),
            Ok(m) if m.len() != src_size => continue,
            Ok(_) => {
                let mine = match &src_hash {
                    Some(h) => *h,
                    None => {
                        let h = content_hash(src)?;
                        src_hash = Some(h);
                        h
                    }
                };
                if content_hash(&cand)? == mine {
                    return Ok(MovePlan::AlreadyHave);
                }
            }
        }
    }
    Err(format!("no free name for {dest:?}"))
}

/// Rename, falling back to copy-then-remove for a cross-device move (the picked
/// folder was on a card, a phone, another partition).
fn move_into(src: &Path, dest: &Path) -> Result<(), String> {
    match std::fs::rename(src, dest) {
        Ok(()) => Ok(()),
        Err(_) => {
            std::fs::copy(src, dest).map_err(|e| format!("copy {src:?} -> {dest:?}: {e}"))?;
            std::fs::remove_file(src).map_err(|e| format!("remove {src:?}: {e}"))?;
            Ok(())
        }
    }
}

/// Move a track row after its file, so the library follows the file instead of
/// losing it. The ID moves too: a track's id IS its path (see `stable_id`),
/// which is what lets a rescan upsert rather than duplicate. A row kept at the
/// old id with a new path looks harmless and breaks the NEXT re-import of the
/// same file, whose staging path is free again after the move and therefore
/// derives the id this row is now borrowing — and the scan dies on the UNIQUE
/// constraint. That is not hypothetical: import, save, import the same file
/// again, and the scan fails.
fn relink(conn: &Connection, from: &Path, to: &Path) -> Result<(), String> {
    let to_str = to.to_string_lossy().into_owned();
    conn.execute(
        "UPDATE tracks SET id = ?1, path = ?2 WHERE path = ?3",
        params![
            super::stable_id("tr", &[to_str.as_str()]),
            to_str,
            from.to_string_lossy().as_ref()
        ],
    )
    .map_err(|e| format!("relink {from:?} -> {to:?}: {e}"))?;
    Ok(())
}

/// Drop the row of a staged file the library already held, so it does not linger
/// as a ghost: the scan keeps vanished rows and flags them missing, and a missing
/// track keeps its album alive — which is how one save used to leave the same
/// album in the grid twice, one tile with the missing icon.
fn forget_row(conn: &Connection, path: &Path) -> Result<(), String> {
    conn
        .execute(
            "DELETE FROM tracks WHERE path = ?1",
            [path.to_string_lossy().as_ref()],
        )
        .map_err(|e| format!("forget {path:?}: {e}"))?;
    Ok(())
}

/// Remove staging folders left empty by a save, deepest first, so the staging
/// area never accumulates the ghost of an imported album.
fn prune_staged_dirs(root: &Path) {
    let mut dirs: Vec<PathBuf> = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.path().to_path_buf())
        .collect();
    dirs.sort_by(|a, b| b.components().count().cmp(&a.components().count()));
    for d in dirs {
        if d != root && std::fs::remove_dir(&d).is_ok() {
            // was empty
        }
    }
}

/// What one Save did, in the words the UI will use ("3 saved · 1 already there").
#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveReport {
    pub moved: usize,
    pub duplicates: usize,
    /// Staged rows whose file was already gone. The old copy-based save left
    /// rows pointing into the cache after deleting the copy, and the scan keeps
    /// vanished rows and flags them missing — so those albums still report
    /// themselves as staged, and their track counts are doubled. A save now
    /// forgets such a row instead of failing on `stat`, which is what lets a
    /// library written by the old code heal itself.
    pub vanished: usize,
}

/// Move staged files into the library through their resolved destinations,
/// flattening into the album folder — a `<Album>/CD1/01.flac` substructure is
/// dropped, because the disc tag is what groups an album in the UI and a
/// half-flat, half-nested library is worse than either. Filenames are kept
/// exactly as the user has them (something outside this app may refer to them);
/// same-name-same-size is skipped, same-name-different-content gets a suffix.
/// Then the staging folders they left behind are pruned.
pub fn save_to_library(
    conn: &Connection,
    cache_dir: &Path,
    roots: &[PathBuf],
    music_dir: &Path,
    staged: &[PathBuf],
    mut progress: impl FnMut(usize, usize),
) -> Result<SaveReport, String> {
    let root = import_dir(cache_dir);
    let total = staged.len();
    let mut report = SaveReport::default();
    let mut by_album: HashMap<String, Destination> = HashMap::new();
    for (i, src) in staged.iter().enumerate() {
        progress(i, total);
        let album_id = album_id_of_path(conn, src)?;
        let dest = match by_album.get(&album_id) {
            Some(d) => d.clone(),
            None => {
                let d = album_destination(conn, roots, music_dir, &root, &album_id)?;
                by_album.insert(album_id, d.clone());
                d
            }
        };
        let name = src
            .file_name()
            .ok_or_else(|| format!("{src:?}: no file name"))?;
        if !src.exists() {
            forget_row(conn, src)?;
            report.vanished += 1;
            continue;
        }
        match plan_move(src, &dest.folder.join(name))? {
            MovePlan::Place(at) => {
                if let Some(parent) = at.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| format!("mkdir {parent:?}: {e}"))?;
                }
                move_into(src, &at)?;
                // The row follows the file, keeping its id: a scan that gets
                // there first sees "deleted + added" and hands back exactly the
                // duplicate-with-missing-icon the user reported.
                relink(conn, src, &at)?;
                report.moved += 1;
            }
            MovePlan::AlreadyHave => match std::fs::remove_file(src) {
                Ok(()) => {
                    forget_row(conn, src)?;
                    report.duplicates += 1;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(format!("remove {src:?}: {e}")),
            },
        }
    }
    progress(total, total);
    prune_staged_dirs(&root);
    Ok(report)
}

/// Path component sanitizer: '/' and NUL would escape the folder.
pub fn sanitize_path(name: &str) -> String {
    name.trim().replace(['/', '\0'], "_")
}

/// Delete staged files (and any staging folders left empty).
pub fn discard(conn: &Connection, staged: &[PathBuf]) -> Result<usize, String> {
    let n = staged.len();
    let refs: Vec<&Path> = staged.iter().map(PathBuf::as_path).collect();
    remove_staged(&refs)?;
    // Forget the rows. Deleting only the file hands the row to the scan, and the
    // scan's policy for a vanished file is to KEEP it as missing — right for a
    // file the user moved themselves, wrong for one they just threw away: the
    // row still points inside the staging dir, so the album stays flagged staged
    // and the door the user just emptied lights up again. Those 28 rows of
    // debris in this library were born exactly this way.
    for p in staged {
        conn.execute("DELETE FROM tracks WHERE path = ?1", [p.display().to_string()])
            .map_err(|e| format!("forget {p:?}: {e}"))?;
    }
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


    /// An empty library, opened at `root`.
    fn empty_db(root: &Path) -> Connection {
        crate::library::db::open(&root.join("dest.db")).expect("open")
    }

    /// Seed an album and its track rows straight into the DB. `tiny_mp3` files
    /// are untagged, so a real scan would placeholder every one of them as
    /// Unknown Artist — the resolver only reads rows, and what is under test is
    /// the decision, not the tags that produced it.
    fn seed_album(
        conn: &Connection,
        id: &str,
        artist: &str,
        title: &str,
        paths: &[PathBuf],
    ) -> String {
        let artist_id = format!("ar-{artist}");
        conn.execute(
            // Real rows carry the display name and its sort key; the id is
            // opaque, and rule 3 must not leak it into a folder name.
            "INSERT OR IGNORE INTO artists(id, name, sort_name) VALUES(?1,?2,?2)",
            params![artist_id, artist],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO albums(id, artist_id, title) VALUES(?1,?2,?3)",
            params![id, artist_id, title],
        )
        .unwrap();
        for (i, p) in paths.iter().enumerate() {
            conn.execute(
                "INSERT INTO tracks(id, album_id, title, path, mtime_ns, size, duration_sec)
                 VALUES(?1,?2,?3,?4,0,1,1)",
                params![format!("{id}-t{i}"), id, format!("track {i}"), p.to_string_lossy()],
            )
            .unwrap();
        }
        id.to_string()
    }

    #[test]
    fn destination_merges_into_the_folder_the_album_is_actually_in() {
        let root = temp_dir("dest-merge");
        let music = root.join("music");
        let staging = import_dir(&root.join("cache"));
        let conn = empty_db(&root);
        // The collection lives one level deeper than the template would guess.
        seed_album(
            &conn,
            "lib-impera",
            "Ghost",
            "Impera",
            &[music.join("Music Files/Ghost/Impera/01 Imperium.mp3")],
        );
        let staged = seed_album(
            &conn,
            "st-impera",
            "Ghost",
            "Impera",
            &[staging.join("Impera/01 Imperium.mp3")],
        );
        let d = album_destination(&conn, &[music.clone()], &music, &staging, &staged).unwrap();
        assert_eq!(d.folder, music.join("Music Files/Ghost/Impera"));
        assert_eq!(d.rule, "merge");
        assert_eq!(d.merged_into.as_deref(), Some("Impera"));
        let _ = std::fs::remove_dir_all(&root);
    }


    /// The case the exclusion broke: a staged addition to an album that is
    /// already indexed shares that album's row (tags decide), so rule 1 must
    /// still see the album's existing folder. Before, the album excluded itself,
    /// rule 2 fired, and the new file joined a fresh sibling folder — splitting
    /// one album across two places on disk.
    #[test]
    fn an_addition_to_an_indexed_album_joins_the_album_it_belongs_to() {
        let root = temp_dir("dest-addition");
        let music = root.join("music");
        let staging = import_dir(&root.join("cache"));
        let conn = empty_db(&root);
        // One album, its files in the library, one more file staged — all in the
        // same album row, which is what the scanner's tag identity produces.
        seed_album(
            &conn,
            "al-impera",
            "Ghost",
            "Impera",
            &[
                music.join("Ghost/Impera/01. Imperium.mp3"),
                music.join("Ghost/Impera/02. Kaisarion.mp3"),
                staging.join("2022 - Impera/03. Spillways.mp3"),
            ],
        );
        // Another album by the same artist, so rule 2 has somewhere to go wrong.
        seed_album(
            &conn,
            "al-prequelle",
            "Ghost",
            "Prequelle",
            &[music.join("Music Files/Ghost/Prequelle/Ashes.mp3")],
        );
        let d = album_destination(&conn, &[music.clone()], &music, &staging, "al-impera").unwrap();
        assert_eq!(d.rule, "merge", "the album is on disk: join it");
        assert_eq!(d.folder, music.join("Ghost/Impera"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_new_album_by_a_known_artist_joins_that_artists_folder() {
        let root = temp_dir("dest-artist");
        let music = root.join("music");
        let staging = import_dir(&root.join("cache"));
        let conn = empty_db(&root);
        seed_album(
            &conn,
            "lib-prequelle",
            "Ghost",
            "Prequelle",
            &[music.join("Music Files/Ghost/Prequelle/Ashes.mp3")],
        );
        seed_album(
            &conn,
            "lib-skeleta",
            "Ghost",
            "Skeletá",
            &[music.join("Music Files/Ghost/Skeletá/Cenotaph.mp3")],
        );
        let staged = seed_album(
            &conn,
            "st-impera",
            "Ghost",
            "Impera",
            &[staging.join("Impera/01 Imperium.mp3")],
        );
        let d = album_destination(&conn, &[music.clone()], &music, &staging, &staged).unwrap();
        // Not <music>/Ghost/Impera — that is the answer that split this user's
        // Ghost catalogue across two halves of the tree.
        assert_eq!(d.folder, music.join("Music Files/Ghost/Impera"));
        assert_eq!(d.rule, "artist");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_new_artist_on_an_empty_library_uses_the_primary_root() {
        let root = temp_dir("dest-new");
        let music = root.join("music");
        let staging = import_dir(&root.join("cache"));
        let conn = empty_db(&root);
        let staged = seed_album(
            &conn,
            "st-debut",
            "Kadavar",
            "For The Dead",
            &[staging.join("For The Dead/01.mp3")],
        );
        let d = album_destination(&conn, &[music.clone()], &music, &staging, &staged).unwrap();
        assert_eq!(d.folder, music.join("Kadavar/For The Dead"));
        assert_eq!(d.rule, "new");
        let _ = std::fs::remove_dir_all(&root);
    }


    /// A new artist joins the folder the collection actually lives in, not the
    /// root's top level: the plain convention would start a second regime beside
    /// 248 albums that are all one level deeper.
    #[test]
    fn a_new_artist_lands_where_the_collection_lives() {
        let root = temp_dir("dest-home");
        let music = root.join("music");
        let staging = import_dir(&root.join("cache"));
        let conn = empty_db(&root);
        for (id, artist, title, file) in [
            ("l1", "Ghost", "Prequelle", "Ashes.mp3"),
            ("l2", "Ghost", "Skeletá", "Cenotaph.mp3"),
            ("l3", "Avantasia", "Ghostlights", "1-01.mp3"),
        ] {
            std::fs::create_dir_all(music.join(format!("Music Files/{artist}/{title}"))).unwrap();
            seed_album(
                &conn,
                id,
                artist,
                title,
                &[music.join(format!("Music Files/{artist}/{title}/{file}"))],
            );
        }
        // Two strays at the root's top level: the majority decides, not the
        // shallowest answer.
        seed_album(
            &conn,
            "l4",
            "Ghost",
            "Impera",
            &[music.join("Ghost/Impera/01 Imperium.mp3")],
        );
        let staged = seed_album(
            &conn,
            "st-kadavar",
            "Kadavar",
            "For The Dead",
            &[staging.join("For The Dead/01.mp3")],
        );
        let d = album_destination(&conn, &[music.clone()], &music, &staging, &staged).unwrap();
        assert_eq!(d.folder, music.join("Music Files/Kadavar/For The Dead"));
        assert_eq!(d.rule, "new");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn staged_copies_are_never_a_merge_target() {
        let root = temp_dir("dest-staging");
        let music = root.join("music");
        let staging = import_dir(&root.join("cache"));
        let conn = empty_db(&root);
        // The only same-title album is itself a staged copy: merging into a path
        // that is about to be deleted would leave the album pointing at nothing.
        seed_album(
            &conn,
            "other-staged",
            "Ghost",
            "Impera",
            &[staging.join("Somewhere Else/01.mp3")],
        );
        let staged = seed_album(
            &conn,
            "st-impera",
            "Ghost",
            "Impera",
            &[staging.join("Impera/01 Imperium.mp3")],
        );
        let d = album_destination(&conn, &[music.clone()], &music, &staging, &staged).unwrap();
        assert_eq!(d.rule, "new");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn plan_lists_staged_albums_with_where_each_would_land() {
        let root = temp_dir("dest-plan");
        let music = root.join("music");
        let staging = import_dir(&root.join("cache"));
        let conn = empty_db(&root);
        seed_album(
            &conn,
            "lib-prequelle",
            "Ghost",
            "Prequelle",
            &[music.join("Music Files/Ghost/Prequelle/Ashes.mp3")],
        );
        seed_album(
            &conn,
            "st-impera",
            "Ghost",
            "Impera",
            &[
                staging.join("Impera/01 Imperium.mp3"),
                staging.join("Impera/02 Kaisarion.mp3"),
            ],
        );
        seed_album(
            &conn,
            "in-library",
            "Avantasia",
            "Ghostlights",
            &[music.join("Music Files/Avantasia/Ghostlights/1-01.mp3")],
        );
        let plan = staged_plan(&conn, &[music.clone()], &music, &staging).unwrap();
        assert_eq!(plan.len(), 1, "only the staged album is listed");
        assert_eq!(plan[0].tracks.len(), 2);
        assert_eq!(plan[0].title, "Impera");
        // The modal expands an album in place, so the plan carries the files,
        // listed: disc then track number, not whatever the row order gave.
        let listed: Vec<&str> = plan[0].tracks.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(listed, vec!["track 0", "track 1"]);
        assert_eq!(plan[0].destination.rule, "artist");
        assert_eq!(
            plan[0].destination.folder,
            music.join("Music Files/Ghost/Impera")
        );
        let _ = std::fs::remove_dir_all(&root);
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
        let report =
            save_to_library(&conn, &cache, &[music.clone()], &music, &staged, |_, _| {}).unwrap();
        assert_eq!(report.moved, 1);
        assert_eq!(report.duplicates, 0);
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
        let report =
            save_to_library(&conn, &cache, &[music.clone()], &music, &staged, |_, _| {}).unwrap();
        assert_eq!(
            report.moved, 0,
            "identical library file → already there, nothing moved"
        );
        assert_eq!(report.duplicates, 1, "and it is reported, not swallowed");
        let _ = std::fs::remove_dir_all(&root);
    }


    /// The reported bug: saving staged music used to leave the same album in the
    /// grid twice — a fresh row for the file in the library, and the old row for
    /// the staging copy, which the scan keeps and flags missing, and a missing
    /// track keeps its album alive. One save, one rescan, one row per track.
    #[test]
    fn saving_leaves_no_ghost_of_the_staged_album() {
        let root = temp_dir("save-ghost");
        let cache = root.join("cache");
        let music = root.join("music");
        let src = root.join("source/Impera");
        std::fs::create_dir_all(&music).unwrap();
        std::fs::create_dir_all(&src).unwrap();
        tiny_mp3(&src.join("01 - Imperium.mp3"), 1);
        tiny_mp3(&src.join("02 - Kaisarion.mp3"), 2);
        import_paths(&cache, &[src.clone()], &KnownFiles::empty(), |_, _| {}).unwrap();

        let mut conn = crate::library::db::open(&root.join("g.db")).unwrap();
        crate::library::scan::run_scan_roots(
            &mut conn,
            &[music.clone(), import_dir(&cache)],
            |_, _| {},
            false,
        )
        .unwrap();
        let staged = staged_tracks(&conn, &cache, None, None).unwrap();
        assert_eq!(staged.len(), 2);

        let report =
            save_to_library(&conn, &cache, &[music.clone()], &music, &staged, |_, _| {}).unwrap();
        assert_eq!(report.moved, 2);

        // Exactly what save_imports does afterwards: rescan every root.
        let counts = crate::library::scan::run_scan_roots(
            &mut conn,
            &[music.clone(), import_dir(&cache)],
            |_, _| {},
            false,
        )
        .unwrap();
        let paths: Vec<String> = conn
            .prepare("SELECT path FROM tracks ORDER BY path")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .flatten()
            .collect();
        let albums: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT album_id) FROM tracks",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(paths.len(), 2, "one row per saved track, no ghost");
        assert_eq!(albums, 1, "and one album tile, not two");
        assert_eq!(counts.missing, 0, "nothing points at the deleted staging copy");
        assert!(
            paths.iter().all(|p| p.starts_with(&music.to_string_lossy().as_ref().to_string())),
            "every row lives in the library: {paths:?}"
        );
        assert!(music
            .join("Unknown Artist/Unknown Album/01 - Imperium.mp3")
            .is_file());
        let _ = std::fs::remove_dir_all(&root);
    }


    /// A staged row whose file is already gone is debris, not an error: the old
    /// copy-based save left rows pointing into the cache after deleting the
    /// copy, and the scan keeps vanished rows, so those albums still claim to be
    /// staged and Save used to fail on `stat` — which left the badge stuck for
    /// good. Now a save forgets the row and carries on with the rest.
    #[test]
    fn a_staged_row_with_no_file_is_forgotten_not_fatal() {
        let root = temp_dir("save-vanished");
        let cache = root.join("cache");
        let music = root.join("music");
        let src = root.join("source/Album");
        std::fs::create_dir_all(&music).unwrap();
        std::fs::create_dir_all(&src).unwrap();
        tiny_mp3(&src.join("01.mp3"), 1);
        tiny_mp3(&src.join("02.mp3"), 2);
        import_paths(&cache, &[src.clone()], &KnownFiles::empty(), |_, _| {}).unwrap();
        let mut conn = crate::library::db::open(&root.join("v.db")).unwrap();
        crate::library::scan::run_scan_roots(
            &mut conn,
            &[music.clone(), import_dir(&cache)],
            |_, _| {},
            false,
        )
        .unwrap();
        let staged = staged_tracks(&conn, &cache, None, None).unwrap();
        assert_eq!(staged.len(), 2);
        // The file goes missing behind the library's back (a ghost row).
        std::fs::remove_file(&staged[0]).unwrap();

        let report =
            save_to_library(&conn, &cache, &[music.clone()], &music, &staged, |_, _| {}).unwrap();
        assert_eq!(report.moved, 1, "the file that exists still saves");
        assert_eq!(report.vanished, 1, "the one that does not is reported");
        let left: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tracks WHERE path LIKE ?1",
                [format!("{}%", import_dir(&cache).to_string_lossy())],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(left, 0, "the ghost row is gone, so the staged badge clears");
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
    fn discard_removes_files_prunes_dirs_and_forgets_rows() {
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

        let conn = empty_db(&cache);
        assert_eq!(discard(&conn, &staged).unwrap(), 2);
        assert!(!import_dir(&cache).join("Album").exists(), "pruned");
        assert!(src.join("Album/01.mp3").is_file(), "source untouched");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The half that left 28 rows of debris in a real library: a discarded file
    /// has to leave no trace. Delete the file only, and the following scan sees a
    /// vanished file and keeps it as "missing" — which is the right answer for a
    /// file the user moved, and a lie about one they threw away: the row still
    /// points into the staging dir, so the album reads as still staged.
    /// What the command passes to `discard`: the staged paths of an album, read
    /// back out of the db rather than assumed from the fixture.
    fn staged_paths(conn: &Connection, staging: &Path, album_id: &str) -> Vec<PathBuf> {
        let mut stmt = conn
            .prepare("SELECT path FROM tracks WHERE album_id = ?1")
            .unwrap();
        stmt.query_map([album_id], |r| r.get::<_, String>(0))
            .unwrap()
            .flatten()
            .map(PathBuf::from)
            .filter(|p| p.starts_with(staging))
            .collect()
    }

    #[test]
    fn a_discard_leaves_no_row_behind() {
        let root = temp_dir("discard-row");
        let cache = root.join("cache");
        let staging = import_dir(&cache);
        let conn = empty_db(&root);
        let staged = seed_album(
            &conn,
            "st-popestar",
            "Ghost",
            "Popestar",
            &[
                staging.join("Popestar/01 Square Hammer.mp3"),
                staging.join("Popestar/02 Nocturnal Me.mp3"),
            ],
        );
        for p in [
            staging.join("Popestar/01 Square Hammer.mp3"),
            staging.join("Popestar/02 Nocturnal Me.mp3"),
        ] {
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, b"x").unwrap();
        }
        assert_eq!(
            discard(&conn, &staged_paths(&conn, &staging, &staged)).unwrap(),
            2
        );
        let left: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(left, 0, "a discarded track must not linger as a missing row");
        let _ = std::fs::remove_dir_all(&root);
    }
}
