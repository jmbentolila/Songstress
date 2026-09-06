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

/// Tracks currently staged (path under the import root), optionally scoped
/// to one album or a single track. Empty when the import dir was never
/// created.
pub fn staged_tracks(
    conn: &Connection,
    album_id: Option<&str>,
    track_id: Option<&str>,
) -> Result<Vec<PathBuf>, String> {
    let mut sql = String::from("SELECT path FROM tracks WHERE staged = 1");
    let mut clauses: Vec<&str> = Vec::new();
    if album_id.is_some() {
        clauses.push("album_id = ?1");
    }
    if track_id.is_some() {
        clauses.push(if album_id.is_some() { "id = ?2" } else { "id = ?1" });
    }
    if !clauses.is_empty() {
        sql.push_str(" AND ");
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
    Ok(rows.filter_map(|r| r.ok()).map(PathBuf::from).collect())
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
             WHERE al.artist_id = ?1 AND t.staged = 0",
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
    let base = collection_parent(conn, roots)?.unwrap_or_else(|| music_dir.to_path_buf());
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
fn collection_parent(conn: &Connection, roots: &[PathBuf]) -> Result<Option<PathBuf>, String> {
    let mut by_parent: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT album_id, path FROM tracks WHERE staged = 0")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            let path = PathBuf::from(&row.1);
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
) -> Result<Vec<StagedAlbum>, String> {
    let mut by_album: Vec<(String, usize)> = {
        let mut stmt = conn
            .prepare("SELECT album_id, path FROM tracks WHERE staged = 1")
            .map_err(|e| e.to_string())?;
        let mut counts: HashMap<String, usize> = HashMap::new();
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        for row in rows.flatten() {
            *counts.entry(row.0).or_default() += 1;
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
                "SELECT id, title, duration_sec, disc, track FROM tracks
                 WHERE album_id = ?1 AND staged = 1",
            )
            .map_err(|e| e.to_string())?;
        let mut tracks: Vec<StagedTrack> = stmt
            .query_map([album_id.clone()], |r| {
                Ok(StagedTrack {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    duration_sec: r.get(2)?,
                    disc: r.get(3)?,
                    track: r.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?
            .flatten()
            .collect();
        // Staged lists follow the same display rule (this one never even
        // had the unnumbered-first ORDER BY); SQLite can't fold case.
        tracks.sort_by_key(|t| super::track_order_key(t.disc, t.track, &t.title));
        out.push(StagedAlbum {
            destination: album_destination(conn, roots, music_dir, &album_id)?,
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
    // The flag goes WITH the move: from here on the row is an ordinary library
    // track, and a saved album that still read as pending would light the door
    // the user just emptied.
    conn.execute(
        "UPDATE tracks SET id = ?1, path = ?2, staged = 0 WHERE path = ?3",
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
                let d = album_destination(conn, roots, music_dir, &album_id)?;
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
            // Same bytes already in the library. The user's ruling on this case:
            // the file that was already ours is the one that stays, and the one
            // they just pointed at is removed — the alternative is the same
            // recording in the library twice over. Reported, never silent.
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

/// What an import did, for the modal to show as a receipt. `already` is the half
/// that needs explaining: pointing at files the library indexes is not a failure,
/// but a progress ring that ends with nothing on screen reads as one.
#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedAlbum {
    pub artist: String,
    pub title: String,
    pub tracks: usize,
}

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    pub staged: Vec<ImportedAlbum>,
    pub already: Vec<ImportedAlbum>,
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

/// The files an import request expands to: directories are walked, unsupported
/// files dropped. Nothing is copied and nothing is renamed, so this is exactly
/// what the user pointed at — which is also what gets indexed, flagged, and shown
/// in the modal one by one.
pub fn import_targets(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for p in paths {
        if p.is_dir() {
            for entry in walkdir::WalkDir::new(p).follow_links(false) {
                let entry = entry.map_err(|e| format!("walk {p:?}: {e}"))?;
                if entry.file_type().is_file() && scan::is_supported(entry.path()) {
                    out.push(entry.path().to_path_buf());
                }
            }
        } else if p.is_file() && scan::is_supported(p) {
            out.push(p.clone());
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

/// Which album rows a set of files belongs to, counted per album. Per-path lookup
/// rather than one `IN (…)`: an import is tens of files, and the query stays
/// legible. Files with no row yet (flagged before the scan wrote them) drop out.
pub fn album_groups(conn: &Connection, paths: &[PathBuf]) -> Result<Vec<ImportedAlbum>, String> {

    let mut counts: std::collections::HashMap<String, usize> = Default::default();
    for p in paths {
        let album_id = album_id_of_path(conn, p).unwrap_or_default();
        if !album_id.is_empty() {
            *counts.entry(album_id).or_default() += 1;
        }
    }
    let mut out = Vec::with_capacity(counts.len());
    for (album_id, tracks) in counts {
        let (_, artist, title) = album_identity(conn, &album_id)?;
        out.push(ImportedAlbum { artist, title, tracks });
    }
    out.sort_by(|a, b| a.artist.cmp(&b.artist).then(a.title.cmp(&b.title)));
    Ok(out)
}

/// Index-first staging: import writes rows for files where they are and flags
/// them here. Everything downstream — the destination cascade, the collection
/// tally, the plan, the badge, the save and discard verbs — asks the column
/// rather than guessing from a path prefix.
pub fn mark_staged(conn: &Connection, paths: &[PathBuf]) -> Result<usize, String> {
    let mut n = 0;
    for p in paths {
        n += conn
            .execute(
                "UPDATE tracks SET staged = 1 WHERE path = ?1",
                [p.display().to_string()],
            )
            .map_err(|e| format!("flag {p:?}: {e}"))?;
    }
    Ok(n)
}

/// The copy era staged by writing files under the cache directory, so those rows
/// are pending whatever the column says. Idempotent and cheap (it only looks at
/// unflagged rows), so it runs at every startup rather than being a migration
/// that would have to be told where the cache lives.
pub fn mark_legacy_staged(conn: &Connection, staging_root: &Path) -> Result<usize, String> {
    let rows: Vec<PathBuf> = {
        let mut stmt = conn
            .prepare("SELECT path FROM tracks WHERE staged = 0")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok())
            .filter(|p| Path::new(p).starts_with(staging_root))
            .map(PathBuf::from)
            .collect()
    };
    mark_staged(conn, &rows)
}

/// Throw the pile away: forget the rows, and delete the file ONLY where the app
/// wrote that file itself.
///
/// Under in-place staging a staged row points at the user's own file, in a folder
/// they chose, and "discard" means "I don't want this in my library" — not "erase
/// what I own". The exception is the copy era's rows, whose files live under the
/// staging directory because the app put them there; those copies are ours to
/// delete. Nothing outside that directory is deleted, and no directory outside it
/// is pruned: the folder a user imported from is their business.
pub fn discard(
    conn: &Connection,
    staging_root: &Path,
    staged: &[PathBuf],
) -> Result<usize, String> {
    let n = staged.len();
    let ours: Vec<&Path> = staged
        .iter()
        .filter(|p| p.starts_with(staging_root))
        .map(PathBuf::as_path)
        .collect();
    remove_staged(&ours)?;
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

    /// Import no longer copies anything, so what the layout test covered is now:
    /// the request expands to files, unsupported files are not among them, and
    /// the cache directory stays empty.
    #[test]
    fn import_targets_expands_without_copying() {
        let root = temp_dir("targets");
        let src = root.join("source");
        let cache = root.join("cache");
        std::fs::create_dir_all(src.join("Cool Album")).unwrap();
        std::fs::write(src.join("Cool Album/01 - A.mp3"), b"a").unwrap();
        std::fs::write(src.join("Cool Album/02 - B.mp3"), b"b").unwrap();
        std::fs::write(src.join("Cool Album/cover.jpg"), b"not audio").unwrap();
        std::fs::write(src.join("loose.mp3"), b"c").unwrap();

        let got = import_targets(&[src.join("Cool Album"), src.join("loose.mp3")]).unwrap();
        assert_eq!(got.len(), 3, "two album files + one loose, no cover.jpg");
        assert!(got.contains(&src.join("Cool Album/01 - A.mp3")));
        assert!(!import_dir(&cache).exists(), "import writes nothing of its own");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The copy era's staging: a file the APP wrote under the staging directory.
    /// Once that directory has been scanned, `mark_legacy_staged` turns those rows
    /// into the pile — the same call the app makes at startup against an upgraded
    /// database, which is why these tests exercise it rather than flagging by hand.
    fn write_legacy_copy(cache: &Path, rel: &str, filler: u8) -> PathBuf {
        let at = import_dir(cache).join(rel);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        tiny_mp3(&at, filler);
        at
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
        let d = album_destination(&conn, &[music.clone()], &music, &staged).unwrap();
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
        let d = album_destination(&conn, &[music.clone()], &music, "al-impera").unwrap();
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
        let d = album_destination(&conn, &[music.clone()], &music, &staged).unwrap();
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
        let d = album_destination(&conn, &[music.clone()], &music, &staged).unwrap();
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
        let d = album_destination(&conn, &[music.clone()], &music, &staged).unwrap();
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
        let other = seed_album(
            &conn,
            "other-staged",
            "Ghost",
            "Impera",
            &[staging.join("Somewhere Else/01.mp3")],
        );
        // Flagged, because flagged is what staged means now.
        mark_staged(&conn, &[staging.join("Somewhere Else/01.mp3")]).unwrap();
        let _ = other;
        let staged = seed_album(
            &conn,
            "st-impera",
            "Ghost",
            "Impera",
            &[staging.join("Impera/01 Imperium.mp3")],
        );
        mark_staged(&conn, &[staging.join("Impera/01 Imperium.mp3")]).unwrap();
        let d = album_destination(&conn, &[music.clone()], &music, &staged).unwrap();
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
        mark_staged(
            &conn,
            &[
                staging.join("Impera/01 Imperium.mp3"),
                staging.join("Impera/02 Kaisarion.mp3"),
            ],
        )
        .unwrap();
        seed_album(
            &conn,
            "in-library",
            "Avantasia",
            "Ghostlights",
            &[music.join("Music Files/Avantasia/Ghostlights/1-01.mp3")],
        );
        let plan = staged_plan(&conn, &[music.clone()], &music).unwrap();
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
        std::fs::write(src.join("song.mp3"), b"original").unwrap();
        let staged_at = write_legacy_copy(&cache, "source/song.mp3", 1);
        assert_eq!(staged_at, cache.join("import/source/song.mp3"));

        // Scan BOTH roots so the staged copy is a known track.
        let mut conn = scanned_db(&root);
        crate::library::scan::run_scan_roots(
            &mut conn,
            &[music.clone(), import_dir(&cache)],
            |_, _| {},
            false,
        )
        .unwrap();
        mark_legacy_staged(&conn, &import_dir(&cache)).unwrap();
        let staged = staged_tracks(&conn, None, None).unwrap();
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

        // Saving the same content again (re-import) dedupes by hash.
        write_legacy_copy(&cache, "source/song.mp3", 1);
        let mut conn = scanned_db(&root);
        crate::library::scan::run_scan_roots(
            &mut conn,
            &[music.clone(), import_dir(&cache)],
            |_, _| {},
            false,
        )
        .unwrap();
        mark_legacy_staged(&conn, &import_dir(&cache)).unwrap();
        let staged = staged_tracks(&conn, None, None).unwrap();
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
        write_legacy_copy(&cache, "source/Impera/01 - Imperium.mp3", 1);
        write_legacy_copy(&cache, "source/Impera/02 - Kaisarion.mp3", 2);

        let mut conn = crate::library::db::open(&root.join("g.db")).unwrap();
        crate::library::scan::run_scan_roots(
            &mut conn,
            &[music.clone(), import_dir(&cache)],
            |_, _| {},
            false,
        )
        .unwrap();
        mark_legacy_staged(&conn, &import_dir(&cache)).unwrap();
        let staged = staged_tracks(&conn, None, None).unwrap();
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
        write_legacy_copy(&cache, "Album/01.mp3", 1);
        write_legacy_copy(&cache, "Album/02.mp3", 2);
        let mut conn = crate::library::db::open(&root.join("v.db")).unwrap();
        crate::library::scan::run_scan_roots(
            &mut conn,
            &[music.clone(), import_dir(&cache)],
            |_, _| {},
            false,
        )
        .unwrap();
        mark_legacy_staged(&conn, &import_dir(&cache)).unwrap();
        let staged = staged_tracks(&conn, None, None).unwrap();
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

    /// The user-reported duplicate, in the model that fixed it: add one file,
    /// then add the folder that contains it. Nothing is skipped by name and size
    /// any more — a path that is already a row has nothing to decide, and a new
    /// path's content decides at Apply. What must hold either way is the thing the
    /// bug violated: one row per track, and a pile that does not grow a second
    /// copy of an album the user already has in it.
    #[test]
    fn re_importing_a_folder_that_holds_an_indexed_file_adds_nothing_twice() {
        let root = temp_dir("reimport");
        let folder = root.join("downloads/Impera");
        std::fs::create_dir_all(&folder).unwrap();
        let a = folder.join("01 - Imperium.mp3");
        let b = folder.join("02 - Kaisarion.mp3");
        tiny_mp3(&a, 1);
        tiny_mp3(&b, 2);

        let mut conn = empty_db(&root);
        import_in_place(&mut conn, &folder, &[a.clone()]);
        import_in_place(&mut conn, &folder, &[a.clone(), b.clone()]);
        import_in_place(&mut conn, &folder, &[a.clone(), b.clone()]);

        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 2, "one row per track after three overlapping imports");
        let pending = staged_tracks(&conn, None, None).unwrap();
        assert_eq!(pending.len(), 2, "and the pile holds each file once");
        let albums: i64 = conn
            .query_row("SELECT COUNT(DISTINCT album_id) FROM tracks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(albums, 1, "one album, not a staged twin of it");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn discarding_a_copy_era_row_deletes_the_apps_copy_and_prunes_its_folder() {
        let root = temp_dir("discard-legacy");
        let cache = root.join("cache");
        let src = root.join("source/Album");
        std::fs::create_dir_all(&src).unwrap();
        tiny_mp3(&src.join("01.mp3"), 1);
        tiny_mp3(&src.join("02.mp3"), 2);
        let copies = vec![
            write_legacy_copy(&cache, "Album/01.mp3", 1),
            write_legacy_copy(&cache, "Album/02.mp3", 2),
        ];
        let conn = scanned_db(&cache);
        mark_legacy_staged(&conn, &import_dir(&cache)).unwrap();
        assert_eq!(staged_tracks(&conn, None, None).unwrap().len(), 2);

        assert_eq!(discard(&conn, &import_dir(&cache), &copies).unwrap(), 2);
        assert!(
            !import_dir(&cache).join("Album").exists(),
            "a folder the app made is the app's to prune"
        );
        assert!(src.join("01.mp3").is_file(), "the user\'s folder is not");
        let left: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(left, 0, "and no row is left behind to haunt the album");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The promise the flag model buys: discarding an in-place import cannot
    /// touch a file. The 28 ghosts in this library came from deleting a file and
    /// leaving the row; the mirrored bug — deleting the user\'s file because the
    /// pile said discard — would be worse, so it is locked here.
    #[test]
    fn discarding_an_in_place_import_forgets_the_row_and_touches_nothing() {
        let root = temp_dir("discard-in-place");
        let cache = root.join("cache");
        let downloads = root.join("downloads/2013 - Infestissumam");
        std::fs::create_dir_all(&downloads).unwrap();
        let files = vec![
            downloads.join("01 Infestissumam.mp3"),
            downloads.join("02 Per Aspera Ad Inferi.mp3"),
        ];
        let conn = empty_db(&root);
        seed_album(&conn, "st-inf", "Ghost B.C.", "Infestissumam", &files);
        for f in &files {
            std::fs::write(f, b"the user\'s own bytes").unwrap();
        }
        mark_staged(&conn, &files).unwrap();

        assert_eq!(
            discard(&conn, &import_dir(&cache), &files).unwrap(),
            2
        );
        for f in &files {
            assert!(f.is_file(), "their file, their call: {f:?}");
        }
        assert!(
            downloads.is_dir(),
            "the folder they imported from is outside our boundaries"
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The new import path minus Tauri: expand the request, index the folder the
    /// files live in with indexing restricted to the request, flag what was not
    /// in the library before. Same order as `import_music`.
    fn import_in_place(conn: &mut Connection, folder: &Path, files: &[PathBuf]) {
        let only: std::collections::HashSet<PathBuf> = files.iter().cloned().collect();
        crate::library::scan::run_scan_files(conn, &[folder.to_path_buf()], Some(&only), |_, _| {}, false)
            .unwrap();
        mark_staged(conn, files).unwrap();
    }

    #[test]
    fn an_import_indexes_the_file_where_it_lives_and_flags_it() {
        let root = temp_dir("in-place");
        let cache = root.join("cache");
        let music = root.join("music");
        let downloads = root.join("downloads");
        std::fs::create_dir_all(&downloads).unwrap();
        let a = downloads.join("01 Secular Haze.mp3");
        tiny_mp3(&a, 1);

        let mut conn = empty_db(&root);
        import_in_place(&mut conn, &downloads, &[a.clone()]);

        let path: String = conn
            .query_row("SELECT path FROM tracks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(Path::new(&path), a, "the row points at the original");
        let staged: i64 = conn
            .query_row("SELECT staged FROM tracks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(staged, 1, "and it is the pile, by the column");
        assert!(
            !import_dir(&cache).exists(),
            "nothing was copied into the cache to make that true"
        );
        assert!(staged_tracks(&conn, None, None).unwrap().len() == 1);
        let _ = std::fs::remove_dir_all(&root);
        let _ = music;
    }

    /// One file out of ~/Downloads must not import ~/Downloads. The folder is
    /// walked so album identity has context, but only the requested file is
    /// indexed — and a partial survey must not call the files it did not look at
    /// missing, which would otherwise be reported as a pile of lost music.
    #[test]
    fn a_filtered_import_leaves_its_neighbours_alone() {
        let root = temp_dir("filtered");
        let folder = root.join("downloads");
        std::fs::create_dir_all(&folder).unwrap();
        let asked = folder.join("01 Asked.mp3");
        tiny_mp3(&asked, 1);
        tiny_mp3(&folder.join("02 Not Asked.mp3"), 2);
        std::fs::write(folder.join("cover.jpg"), b"artwork").unwrap();

        let mut conn = empty_db(&root);
        let only = crate::library::scan::run_scan_files(
            &mut conn,
            &[folder.clone()],
            Some(&std::collections::HashSet::from([asked.clone()])),
            |_, _| {},
            false,
        )
        .unwrap();
        assert_eq!(only.added, 1);
        assert_eq!(only.missing, 0, "a partial survey claims nothing missing");
        let rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 1, "the neighbour is still not in the library");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Saving in-place staged music moves the file the user actually imported,
    /// clears the flag, and leaves the folder it came from exactly as it found
    /// it — the third boundary the user drew: files are ours to move, folders
    /// outside the library are not ours to tidy.
    #[test]
    fn saving_an_in_place_import_moves_the_original_and_ignores_its_folder() {
        let root = temp_dir("save-in-place");
        let cache = root.join("cache");
        let music = root.join("music");
        std::fs::create_dir_all(music.join("Ghost/Impera")).unwrap();
        let downloads = root.join("downloads");
        std::fs::create_dir_all(&downloads).unwrap();
        let file = downloads.join("01 Imperium.mp3");
        tiny_mp3(&file, 1);

        let mut conn = empty_db(&root);
        import_in_place(&mut conn, &downloads, &[file.clone()]);
        let staged = staged_tracks(&conn, None, None).unwrap();
        assert_eq!(staged.len(), 1);

        let report =
            save_to_library(&conn, &cache, &[music.clone()], &music, &staged, |_, _| {}).unwrap();
        assert_eq!(report.moved, 1);
        let landed = music.join("Unknown Artist/Unknown Album/01 Imperium.mp3");
        assert!(landed.is_file(), "moved into the library");
        assert!(!file.exists(), "the original is the file that moved");
        assert!(downloads.is_dir(), "the folder it came from is left alone");
        let staged: i64 = conn
            .query_row("SELECT staged FROM tracks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(staged, 0, "a saved album is not pending; the door must empty");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The user\'s ruling on the duplicate case: the copy that was already in the
    /// library is the one that stays, the one they just pointed at is removed, and
    /// it is reported. Deleting a file of theirs is the one thing a music app gets
    /// blamed for, so it happens only here, only after Apply, and never quietly.
    #[test]
    fn a_duplicate_save_removes_the_new_file_and_keeps_the_library_one() {
        let root = temp_dir("dup-in-place");
        let cache = root.join("cache");
        let music = root.join("music/Ghost/Impera");
        std::fs::create_dir_all(&music).unwrap();
        let downloads = root.join("downloads");
        std::fs::create_dir_all(&downloads).unwrap();
        tiny_mp3(&music.join("01 Imperium.mp3"), 7);
        let mine = downloads.join("01 Imperium.mp3");
        tiny_mp3(&mine, 7);

        let mut conn = empty_db(&root);
        crate::library::scan::run_scan_files(
            &mut conn,
            &[music.clone()],
            None,
            |_, _| {},
            false,
        )
        .unwrap();
        import_in_place(&mut conn, &downloads, &[mine.clone()]);
        let staged = staged_tracks(&conn, None, None).unwrap();
        assert_eq!(staged.len(), 1);

        let report =
            save_to_library(&conn, &cache, &[root.join("music")], &root.join("music"), &staged, |_, _| {})
                .unwrap();
        assert_eq!(report.moved, 0);
        assert_eq!(report.duplicates, 1, "reported, not swallowed");
        assert!(!mine.exists(), "theirs went");
        assert!(music.join("01 Imperium.mp3").is_file(), "ours stayed");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Upgrading must not lose the pile: rows the copy era wrote under the cache
    /// directory are pending under the flag model too, and the modal has to say so
    /// the morning after the upgrade rather than the day the user notices.
    #[test]
    fn legacy_staged_rows_survive_the_upgrade() {
        let root = temp_dir("upgrade");
        let cache = root.join("cache");
        write_legacy_copy(&cache, "2013 - Infestissumam/01.mp3", 1);
        let conn = scanned_db(&cache);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM tracks WHERE staged = 1", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0,
            "a fresh scan does not know the pile is a pile"
        );
        assert_eq!(mark_legacy_staged(&conn, &import_dir(&cache)).unwrap(), 1);
        assert_eq!(staged_tracks(&conn, None, None).unwrap().len(), 1);
        // Idempotent: the second call has nothing left to do.
        assert_eq!(mark_legacy_staged(&conn, &import_dir(&cache)).unwrap(), 0);
        let _ = std::fs::remove_dir_all(&root);
    }
}
