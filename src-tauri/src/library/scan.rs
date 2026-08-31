//! Library scanner (PHASE2.md §5): walkdir + lofty → grouped upserts.
//!
//! Incremental by (mtime_ns, size); removal sweep for vanished paths.
//! Grouping is directory-authoritative (same folder ⇒ same album); release
//! artist/title/year are per-folder consensus values, and the albumartist
//! tag outranks differing track artists (guests stay guests). Various
//! Artists applies ONLY to albumartist-less folders with mixed track
//! artists.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::{Accessor, ItemKey};
use rusqlite::Connection;

use super::{sort_key, stable_id};

pub const VARIOUS_ARTISTS_ID: &str = "ar-various";
const VARIOUS_ARTISTS_NAME: &str = "Various Artists";

const EXTENSIONS: &[&str] = &[
    "mp3", "flac", "m4a", "aiff", "aif", "ogg", "oga", "opus", "wav",
];

#[derive(Debug, Default, Clone)]
pub struct ScanCounts {
    pub added: usize,
    pub updated: usize,
    /// Always 0 since missing-file rows are kept for relink/removal.
    pub removed: usize,
    /// Rows whose file vanished from disk (kept, flagged missing in dumps).
    pub missing: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

impl ScanCounts {
    /// Files whose DB row changed (added/updated/removed) — used by the
    /// scan-finished event consumer to decide whether to refresh the UI.
    #[allow(dead_code)]
    pub fn touched(&self) -> usize {
        self.added + self.updated + self.removed
    }
}

struct ParsedTrack {
    title: String,
    artist: Option<String>,
    album_artist: Option<String>,
    album: String,
    year: Option<i64>,
    disc: i64,
    track: Option<i64>,
    duration_sec: f64,
}

struct FileEntry {
    path: String,
    mtime_ns: i64,
    size: i64,
}

pub(crate) fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
}

fn walk(root: &Path) -> Result<Vec<FileEntry>, String> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|e| format!("walk error: {e}"))?;
        if !entry.file_type().is_file() || !is_supported(entry.path()) {
            continue;
        }
        let meta = entry
            .metadata()
            .map_err(|e| format!("stat {:?}: {e}", entry.path()))?;
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as i64)
            .unwrap_or(0);
        out.push(FileEntry {
            path: entry.path().to_string_lossy().into_owned(),
            mtime_ns: mtime,
            size: meta.len() as i64,
        });
    }
    Ok(out)
}

/// Parse tags; wav/sparse-tag files degrade to filename-derived titles.
fn parse_file(path: &Path) -> Result<ParsedTrack, String> {
    let tagged = lofty::read_from_path(path).map_err(|e| format!("{path:?}: {e}"))?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());

    let file_title = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    let opt_string = |v: Option<Cow<'_, str>>| -> Option<String> {
        v.map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    let artist = tag.and_then(|t| opt_string(t.artist()));
    // No Accessor for album artist — it's an ItemKey lookup.
    let album_artist = tag
        .and_then(|t| t.get_string(&ItemKey::AlbumArtist))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let album = tag
        .and_then(|t| opt_string(t.album()))
        .unwrap_or_else(|| "Unknown Album".into());
    let title = tag
        .and_then(|t| opt_string(t.title()))
        .unwrap_or(file_title);
    let year = tag.and_then(|t| t.year()).map(i64::from);
    let disc = tag
        .and_then(|t| t.disk())
        .map(i64::from)
        .filter(|n| *n > 0)
        .unwrap_or(1);
    let track = tag
        .and_then(|t| t.track())
        .map(i64::from)
        .filter(|n| *n > 0);

    let duration_sec = tagged.properties().duration().as_secs_f64();

    Ok(ParsedTrack {
        title,
        artist,
        album_artist,
        album,
        year,
        disc,
        track,
        duration_sec,
    })
}

fn norm(s: &str) -> String {
    sort_key(s)
}

/// Resolve or create the artist row keyed by normalized name; returns id.
/// Existing rows keep their display name (only sort_name is authoritative).
fn ensure_artist(
    conn: &Connection,
    cache: &mut HashMap<String, String>,
    name: &str,
) -> rusqlite::Result<String> {
    let key = norm(name);
    if let Some(id) = cache.get(&key) {
        return Ok(id.clone());
    }
    let existing: Option<String> = conn
        .query_row("SELECT id FROM artists WHERE sort_name = ?1", [&key], |r| r.get(0))
        .ok();
    let id = existing.unwrap_or_else(|| stable_id("ar", &[&key]));
    conn.execute(
        "INSERT INTO artists(id, name, sort_name) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO NOTHING",
        rusqlite::params![id, name.trim(), key],
    )?;
    cache.insert(key, id.clone());
    Ok(id)
}

/// Single-root convenience wrapper (tests); the app scans via run_scan_roots.
#[cfg(test)]
pub fn run_scan(
    conn: &mut Connection,
    root: &Path,
    progress: impl FnMut(usize, usize),
) -> Result<ScanCounts, String> {
    run_scan_roots(conn, &[root.to_path_buf()], progress, false)
}

/// Scan one or more roots in a single run (library dir + import staging).
/// Grouping and identity work exactly as before across all roots; the
/// deletion sweep only removes paths that NO root contains.
///
/// `full = true` reparses every file even when (mtime, size) is unchanged —
/// required whenever GROUPING or TAG-CONSENSUS logic changes, since skipped
/// files never re-enter the grouping pass (their album_id just persists).
/// Convenience for the whole-roots case — the app's own scans pass a filter, so
/// this is the shape the tests and any future unfiltered caller reach for.
#[allow(dead_code)]
pub fn run_scan_roots(
    conn: &mut Connection,
    roots: &[PathBuf],
    progress: impl FnMut(usize, usize),
    full: bool,
) -> Result<ScanCounts, String> {
    run_scan_files(conn, roots, None, progress, full)
}

/// `only` restricts INDEXING to a set of files, while the roots are still
/// walked in full. That is what an import needs: the album identity of a file
/// comes from the directory it sits in, so a single-file import scans its
/// folder and indexes the one file the user asked for. A filtered run also skips
/// the removal sweep — a partial survey of a root has no standing to call the
/// files it did not look at missing.
pub fn run_scan_files(
    conn: &mut Connection,
    roots: &[PathBuf],
    only: Option<&HashSet<PathBuf>>,
    mut progress: impl FnMut(usize, usize),
    full: bool,
) -> Result<ScanCounts, String> {
    let mut files = Vec::new();
    for root in roots {
        if !root.is_dir() {
            return Err(format!("music root {} does not exist", root.display()));
        }
        files.extend(walk(root)?);
    }
    if let Some(only) = only {
        files.retain(|f| only.contains(Path::new(&f.path)));
    }
    let total = files.len();

    let mut counts = ScanCounts::default();

    // path -> (mtime, size) of what's already in the DB.
    let mut existing: HashMap<String, (i64, i64)> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT path, mtime_ns, size FROM tracks")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (p, m, s) = row.map_err(|e| e.to_string())?;
            existing.insert(p, (m, s));
        }
    }

    struct Work {
        entry: FileEntry,
        parsed: ParsedTrack,
        fresh: bool,
    }
    let mut work: Vec<Work> = Vec::new();
    let mut seen: HashSet<String> = HashSet::with_capacity(total);

    for (i, entry) in files.into_iter().enumerate() {
        progress(i + 1, total);
        seen.insert(entry.path.clone());
        if !full {
            if let Some((old_mtime, old_size)) = existing.get(&entry.path) {
                if *old_mtime == entry.mtime_ns && *old_size == entry.size {
                    counts.skipped += 1;
                    continue;
                }
            }
        }
        // full mode: existing files fall through with fresh=false and are
        // counted as updated by the group write loop.
        match parse_file(Path::new(&entry.path)) {
            Ok(parsed) => {
                let fresh = !existing.contains_key(&entry.path);
                work.push(Work { entry, parsed, fresh });
            }
            Err(e) => counts.errors.push(e),
        }
    }

    // ---- Grouping ------------------------------------------------------
    // Directory-authoritative (same folder ⇒ same album), refined:
    //  * inside a directory, differing normalized album titles separate;
    //  * subgroups merge across directories (multi-disc trees) when titles
    //    match and release artists agree (or either side has none);
    //  * artist/title/year are CONSENSUS values per subgroup, so one stray
    //    missing or odd tag can never split an album again. Year is stored
    //    metadata, never identity.
    struct Sub {
        tracks: Vec<(FileEntry, ParsedTrack, bool)>,
        artist: Option<String>,
        title_norm: String,
        title: String,
        year: Option<i64>,
    }

    /// Most frequent value; ties → first seen (deterministic under any walk
    /// order only within a directory listing, which is fine — the winner is
    /// the same value across rescans unless the user retags).
    fn consensus<T>(values: impl Iterator<Item = Option<T>>) -> Option<T>
    where
        T: PartialEq + Eq + std::hash::Hash + Clone,
    {
        let mut counts: HashMap<T, usize> = HashMap::new();
        let mut order: Vec<T> = Vec::new();
        for v in values.flatten() {
            if !counts.contains_key(&v) {
                order.push(v.clone());
            }
            *counts.entry(v).or_default() += 1;
        }
        let mut best: Option<(T, usize)> = None;
        for k in order {
            let c = counts[&k];
            if best.as_ref().is_none_or(|(_, bc)| c > *bc) {
                best = Some((k, c));
            }
        }
        best.map(|(k, _)| k)
    }

    /// Like `consensus` but counts case/diacritic-insensitively and keeps the
    /// first-seen display spelling of the winning key.
    fn consensus_ci(values: impl Iterator<Item = Option<String>>) -> Option<String> {
        let mut counts: HashMap<String, usize> = HashMap::new();
        let mut order: Vec<(String, String)> = Vec::new(); // (norm, display)
        for v in values.flatten() {
            let n = norm(&v);
            if !counts.contains_key(&n) {
                order.push((n.clone(), v));
            }
            *counts.entry(n).or_default() += 1;
        }
        let mut best: Option<(String, String, usize)> = None;
        for (n, d) in order {
            let c = counts[&n];
            if best.as_ref().is_none_or(|(_, _, bc)| c > *bc) {
                best = Some((n, d, c));
            }
        }
        best.map(|(_, d, _)| d)
    }

    // Pass 1: bucket per (directory, normalized album title).
    let mut subs: Vec<Sub> = Vec::new();
    let mut sub_index: HashMap<(String, String), usize> = HashMap::new();
    for w in work {
        let dir = Path::new(&w.entry.path)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let tnorm = norm(&w.parsed.album);
        match sub_index.get(&(dir.clone(), tnorm.clone())) {
            Some(&si) => subs[si].tracks.push((w.entry, w.parsed, w.fresh)),
            None => {
                sub_index.insert((dir, tnorm.clone()), subs.len());
                subs.push(Sub {
                    tracks: vec![(w.entry, w.parsed, w.fresh)],
                    artist: None,
                    title_norm: tnorm,
                    title: String::new(),
                    year: None,
                });
            }
        }
    }

    // Pass 2: resolve consensus metadata per subgroup. Release artist falls
    // back to track-artist consensus ONLY when the folder tags no albumartist
    // AND all track artists agree — otherwise "The Singles" by Edguy (tagged)
    // and by Phil Collins (untagged) merged, since Pass 3 treats unknown as
    // matching anything. Mixed track artists stay None → Various Artists.
    for s in &mut subs {
        s.artist = consensus_ci(s.tracks.iter().map(|(_, p, _)| p.album_artist.clone()))
            .or_else(|| {
                let mut distinct: Vec<String> = Vec::new();
                for (_, p, _) in s.tracks.iter() {
                    if let Some(a) = &p.artist {
                        let n = norm(a);
                        if !distinct.contains(&n) {
                            distinct.push(n);
                        }
                    }
                }
                if distinct.len() == 1 {
                    consensus_ci(s.tracks.iter().map(|(_, p, _)| p.artist.clone()))
                } else {
                    None
                }
            });
        s.title = s.tracks[0].1.album.clone();
        s.year = consensus(s.tracks.iter().map(|(_, p, _)| p.year));
    }

    // Pass 3: merge subgroups sharing a normalized title when their release
    // artists agree (unknown matches anything) — multi-disc trees live in
    // sibling folders. Distinct known artists with the same title stay
    // separate; artist-less subgroups join the largest such group.
    struct AlbumGroup {
        artist: Option<String>,
        title: String,
        year: Option<i64>,
        tracks: Vec<(FileEntry, ParsedTrack, bool)>,
    }
    let mut groups: Vec<AlbumGroup> = Vec::new();
    let mut title_keys: Vec<String> = Vec::new();
    let mut buckets: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, s) in subs.iter().enumerate() {
        match buckets.entry(s.title_norm.clone()) {
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(vec![i]);
                title_keys.push(s.title_norm.clone());
            }
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.get_mut().push(i);
            }
        }
    }
    for key in title_keys {
        let members = &buckets[&key];
        let mut distinct: Vec<String> = Vec::new();
        for &i in members {
            if let Some(a) = &subs[i].artist {
                let n = norm(a);
                if !distinct.contains(&n) {
                    distinct.push(n);
                }
            }
        }
        if distinct.len() <= 1 {
            // One cluster: every subgroup shares this title and at most one
            // release artist (or none of them tagged one).
            let mut merged = AlbumGroup {
                artist: members.iter().find_map(|&i| subs[i].artist.clone()),
                title: subs[members[0]].title.clone(),
                year: consensus(members.iter().map(|&i| subs[i].year.clone())),
                tracks: Vec::new(),
            };
            for &i in members {
                merged.tracks.append(&mut subs[i].tracks);
            }
            groups.push(merged);
        } else {
            // Same title, several real artists: partition by artist; unknowns
            // attach to the largest partition (tie → first seen).
            let mut parts: Vec<(String, AlbumGroup)> = Vec::new();
            let mut unknown: Vec<usize> = Vec::new();
            for &i in members {
                match subs[i].artist.clone() {
                    Some(a) => {
                        let n = norm(&a);
                        match parts.iter_mut().find(|(pn, _)| *pn == n) {
                            Some((_, g)) => g.tracks.append(&mut subs[i].tracks),
                            None => parts.push((
                                n,
                                AlbumGroup {
                                    artist: Some(a),
                                    title: subs[i].title.clone(),
                                    year: subs[i].year,
                                    tracks: std::mem::take(&mut subs[i].tracks),
                                },
                            )),
                        }
                    }
                    None => unknown.push(i),
                }
            }
            if !unknown.is_empty() && !parts.is_empty() {
                let mut idx_max = 0;
                for (j, (_, g)) in parts.iter().enumerate() {
                    if g.tracks.len() > parts[idx_max].1.tracks.len() {
                        idx_max = j;
                    }
                }
                for &i in &unknown {
                    parts[idx_max].1.tracks.append(&mut subs[i].tracks);
                }
            } else if !unknown.is_empty() {
                for i in unknown {
                    groups.push(AlbumGroup {
                        artist: None,
                        title: subs[i].title.clone(),
                        year: subs[i].year,
                        tracks: std::mem::take(&mut subs[i].tracks),
                    });
                }
            }
            groups.extend(parts.into_iter().map(|(_, g)| g));
        }
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut artist_cache: HashMap<String, String> = HashMap::new();
    // Pre-seed Various Artists so it exists even before any compilation lands.
    tx.execute(
        "INSERT INTO artists(id, name, sort_name) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO NOTHING",
        rusqlite::params![VARIOUS_ARTISTS_ID, VARIOUS_ARTISTS_NAME, norm(VARIOUS_ARTISTS_NAME)],
    )
    .map_err(|e| e.to_string())?;

    for g in &groups {
        // Trust albumartist: if a release artist was resolved, differing
        // track artists are just guests/feats. Various Artists applies only
        // to albums with NO albumartist anywhere and mixed track artists.
        let artist_name: String = if let Some(name) = &g.artist {
            name.clone()
        } else {
            let mut distinct: Vec<(String, String)> = Vec::new(); // (norm, display)
            for (_, p, _) in &g.tracks {
                if let Some(a) = &p.artist {
                    let n = norm(a);
                    if !distinct.iter().any(|(x, _)| *x == n) {
                        distinct.push((n, a.clone()));
                    }
                }
            }
            match distinct.len() {
                0 => "Unknown Artist".to_string(),
                1 => distinct.remove(0).1,
                _ => VARIOUS_ARTISTS_NAME.to_string(),
            }
        };
        let artist_id = ensure_artist(&tx, &mut artist_cache, &artist_name)
            .map_err(|e| e.to_string())?;

        // Retag-adopt: a changed group whose (artist, normalized title)
        // matches an EXISTING album joins it regardless of year — year is
        // stored metadata, never identity. This is what lets a tag-editor
        // save merge a stray file into the compilation/album it belongs to
        // (the album's own files were skipped by the incremental scan and
        // are not part of this group).
        let adopted: Option<String> = {
            let mut stmt = tx
                .prepare(
                    "SELECT al.title, al.id FROM albums al
                     JOIN artists ar ON ar.id = al.artist_id
                     WHERE ar.sort_name = ?1",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([&norm(&artist_name)], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                })
                .map_err(|e| e.to_string())?;
            let mut hit = None;
            for row in rows {
                let (title, id) = row.map_err(|e| e.to_string())?;
                if norm(&title) == norm(&g.title) {
                    hit = Some(id);
                    break;
                }
            }
            hit
        };
        let album_id = adopted.unwrap_or_else(|| {
            stable_id(
                "al",
                &[
                    &norm(&artist_name),
                    &norm(&g.title),
                    &g.year.map(|y| y.to_string()).unwrap_or_default(),
                ],
            )
        });
        tx.execute(
            "INSERT INTO albums(id, artist_id, title, year) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO NOTHING",
            rusqlite::params![album_id, artist_id, g.title.trim(), g.year],
        )
        .map_err(|e| e.to_string())?;

        let mut rows: Vec<(String, String, i64, Option<i64>, String, f64)> = Vec::new();
        for (entry, p, fresh) in &g.tracks {
            let tid = stable_id("tr", &[&entry.path]);
            rows.push((
                tid,
                entry.path.clone(),
                p.disc,
                p.track,
                p.title.trim().to_string(),
                p.duration_sec,
            ));
            if *fresh {
                counts.added += 1;
            } else {
                counts.updated += 1;
            }
        }
        // Deterministic ordering within the album regardless of walk order.
        rows.sort_by_key(|(_, _, disc, track, _, _)| (*disc, track.unwrap_or(9_999)));
        for (tid, path, disc, track, title, dur) in &rows {
            let fe = g.tracks.iter().find(|(e, _, _)| e.path == *path);
            let (mtime_ns, size) = fe.map(|(e, _, _)| (e.mtime_ns, e.size)).unwrap_or((0, 0));
            tx.execute(
                "INSERT INTO tracks(id, album_id, disc, track, title, duration_sec, path, mtime_ns, size)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(path) DO UPDATE SET
                   album_id = excluded.album_id, disc = excluded.disc,
                   track = excluded.track, title = excluded.title,
                   duration_sec = excluded.duration_sec,
                   mtime_ns = excluded.mtime_ns, size = excluded.size",
                rusqlite::params![tid, album_id, disc, track, title, dur, path, mtime_ns, size],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // Removal sweep: paths not seen this run are GONE from disk — but their
    // rows are KEPT and flagged missing in the dump (existence check), so the
    // user can relink the file or remove the track explicitly (Step 2a
    // follow-up). Nothing is auto-deleted anymore.
    let removed_existing: Vec<String> = if only.is_some() {
        Vec::new()
    } else {
        existing.keys().filter(|p| !seen.contains(*p)).cloned().collect()
    };
    counts.missing = removed_existing.len();

    // Orphan cleanup after removals.
    tx.execute_batch(
        "DELETE FROM albums WHERE id NOT IN (SELECT DISTINCT album_id FROM tracks);
         DELETE FROM artists WHERE id NOT IN (SELECT DISTINCT artist_id FROM albums)
           AND id != 'ar-various';",
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_src() -> std::path::PathBuf {
        std::path::Path::new("fixtures/library").to_path_buf()
    }

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("songstress-m2-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Recursive copy so tests can mutate a private tree.
    fn copy_tree(src: &Path, dst: &Path) {
        for entry in walkdir::WalkDir::new(src) {
            let entry = entry.expect("walk");
            let rel = entry.path().strip_prefix(src).expect("prefix");
            let target = dst.join(rel);
            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&target).expect("mkdir");
            } else {
                std::fs::copy(entry.path(), &target).expect("copy");
            }
        }
    }

    fn album_tracks(conn: &Connection, title: &str) -> Vec<(i64, Option<i64>, String)> {
        let mut stmt = conn
            .prepare(
                "SELECT t.disc, t.track, t.title FROM tracks t
                 JOIN albums a ON a.id = t.album_id WHERE a.title = ?1
                 ORDER BY t.disc, t.track",
            )
            .expect("stmt");
        let rows = stmt
            .query_map([title], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .expect("map");
        rows.map(|r| r.expect("row")).collect()
    }

    // ---- staged-fixture helpers -------------------------------------------
    // Real-world grouping bugs need specific tag combinations; instead of
    // committing binary fixtures per case, copy a tagged fixture track and
    // retag it in place.

    fn set_tag(tag: &mut lofty::tag::Tag, key: ItemKey, value: Option<&str>) {
        tag.remove_key(&key);
        if let Some(v) = value {
            tag.insert_text(key, v.to_string());
        }
    }

    /// Copy a fixture track into `rel_dir` under the scenario root and retag.
    fn stage(
        root: &Path,
        src: &str,
        rel_dir: &str,
        file_name: &str,
        albumartist: Option<&str>,
        artist: &str,
        album: &str,
        title: &str,
        year: Option<&str>,
        disc_track: (&str, &str),
    ) {
        let dir = root.join(rel_dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let dest = dir.join(file_name);
        std::fs::copy(fixtures_src().join(src), &dest).expect("copy");
        let mut tagged = lofty::read_from_path(&dest).expect("read");
        let has_primary = tagged.primary_tag().is_some();
        let tag = if has_primary {
            tagged.primary_tag_mut().expect("tag")
        } else {
            tagged.first_tag_mut().expect("tag")
        };
        set_tag(tag, ItemKey::AlbumArtist, albumartist);
        set_tag(tag, ItemKey::TrackArtist, Some(artist));
        set_tag(tag, ItemKey::AlbumTitle, Some(album));
        set_tag(tag, ItemKey::TrackTitle, Some(title));
        // Purge both spellings so `year: None` really means no year.
        tag.remove_key(&ItemKey::RecordingDate);
        set_tag(tag, ItemKey::Year, year);
        set_tag(tag, ItemKey::DiscNumber, Some(disc_track.0));
        set_tag(tag, ItemKey::TrackNumber, Some(disc_track.1));
        tagged
            .save_to_path(&dest, lofty::config::WriteOptions::default())
            .expect("save");
    }

    fn album_artist(conn: &Connection, title: &str) -> Option<String> {
        conn.query_row(
            "SELECT ar.name FROM albums al JOIN artists ar ON ar.id = al.artist_id
             WHERE al.title = ?1",
            [title],
            |r| r.get(0),
        )
        .ok()
    }

    #[test]
    fn guest_tracks_stay_under_albumartist() {
        let root = temp_dir("guests");
        let src = "Various Artists/Synth Wars (2019)/02 - Chrome Sunset.flac";
        stage(&root, src, "Aurora Sky/Nightfall Sessions", "01 - Title Track.flac",
            Some("Aurora Sky"), "Aurora Sky", "Nightfall Sessions", "Title Track", Some("2024"), ("1", "1"));
        stage(&root, src, "Aurora Sky/Nightfall Sessions", "02 - Guest Spot.flac",
            Some("Aurora Sky"), "Guest Singer", "Nightfall Sessions", "Guest Spot", Some("2024"), ("1", "2"));

        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        let counts = run_scan(&mut conn, &root, |_, _| {}).expect("scan");
        assert_eq!(counts.errors, Vec::<String>::new());

        assert_eq!(album_tracks(&conn, "Nightfall Sessions").len(), 2, "one album, both tracks");
        assert_eq!(
            album_artist(&conn, "Nightfall Sessions").as_deref(),
            Some("Aurora Sky"),
            "albumartist wins over differing track artists"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn partial_albumartist_majority_wins() {
        let root = temp_dir("partial-aa");
        let src = "Various Artists/Synth Wars (2019)/02 - Chrome Sunset.flac";
        stage(&root, src, "Artist X/Lost Tracks", "01 - One.flac",
            Some("Artist X"), "Artist X", "Lost Tracks", "One", Some("2020"), ("1", "1"));
        stage(&root, src, "Artist X/Lost Tracks", "02 - Two.flac",
            Some("Artist X"), "Feat Guy", "Lost Tracks", "Two", None, ("1", "2"));
        stage(&root, src, "Artist X/Lost Tracks", "03 - Three.flac",
            None, "Another Guest", "Lost Tracks", "Three", None, ("1", "3"));

        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        run_scan(&mut conn, &root, |_, _| {}).expect("scan");

        assert_eq!(album_tracks(&conn, "Lost Tracks").len(), 3, "untagged track stays in the album");
        assert_eq!(
            album_artist(&conn, "Lost Tracks").as_deref(),
            Some("Artist X"),
            "consensus albumartist, not per-track fallback"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn albumartistless_mixed_artists_is_one_various_album() {
        let root = temp_dir("no-aa");
        let src = "Various Artists/Synth Wars (2019)/02 - Chrome Sunset.flac";
        stage(&root, src, "Comp", "01 - A.flac",
            None, "Singer One", "Mixed Bag", "A", None, ("1", "1"));
        stage(&root, src, "Comp", "02 - B.flac",
            None, "Singer Two", "Mixed Bag", "B", None, ("1", "2"));

        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        run_scan(&mut conn, &root, |_, _| {}).expect("scan");

        // Same folder ⇒ same album; no albumartist + mixed artists ⇒ Various.
        assert_eq!(album_tracks(&conn, "Mixed Bag").len(), 2);
        assert_eq!(
            album_artist(&conn, "Mixed Bag").as_deref(),
            Some(VARIOUS_ARTISTS_NAME)
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_or_odd_year_does_not_split() {
        let root = temp_dir("year");
        let src = "Various Artists/Synth Wars (2019)/02 - Chrome Sunset.flac";
        stage(&root, src, "Solo/Archive", "01 - Keep.flac",
            Some("Solo"), "Solo", "Archive", "Keep", Some("1987"), ("1", "1"));
        stage(&root, src, "Solo/Archive", "02 - Keep Too.flac",
            Some("Solo"), "Solo", "Archive", "Keep Too", None, ("1", "2"));

        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        run_scan(&mut conn, &root, |_, _| {}).expect("scan");

        assert_eq!(album_tracks(&conn, "Archive").len(), 2);
        let year: Option<i64> = conn
            .query_row("SELECT year FROM albums WHERE title = 'Archive'", [], |r| r.get(0))
            .expect("row");
        assert_eq!(year, Some(1987), "consensus year kept");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn disc_folders_merge_when_albumartist_partial() {
        let root = temp_dir("discs");
        let src = "Various Artists/Synth Wars (2019)/02 - Chrome Sunset.flac";
        stage(&root, src, "Duo/Merge Me/Disc 1", "01 - D1.flac",
            Some("Duo"), "Duo", "Merge Me", "D1", Some("2015"), ("1", "1"));
        stage(&root, src, "Duo/Merge Me/Disc 2", "01 - D2.flac",
            None, "Duo", "Merge Me", "D2", Some("2015"), ("2", "1"));

        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        run_scan(&mut conn, &root, |_, _| {}).expect("scan");

        assert_eq!(
            album_tracks(&conn, "Merge Me"),
            vec![(1, Some(1), "D1".into()), (2, Some(1), "D2".into())],
            "sibling disc folders merge despite the missing albumartist"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn same_title_untagged_folder_does_not_join_tagged_artists_album() {
        // "The Singles" regression: Edguy's folder tags albumartist, Phil
        // Collins' tags none — the untagged folder must NOT merge into the
        // tagged artist's album (track-artist fallback makes the artists
        // distinct, so Pass 3 partitions them).
        let root = temp_dir("singles-split");
        let src = "Various Artists/Synth Wars (2019)/02 - Chrome Sunset.flac";
        stage(&root, src, "Band A/The Singles", "01 - Alpha.flac",
            Some("Band A"), "Band A", "The Singles", "Alpha", Some("2008"), ("1", "1"));
        stage(&root, src, "Singer B/The Singles", "01 - Beta.flac",
            None, "Singer B", "The Singles", "Beta", Some("2016"), ("1", "1"));

        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        run_scan(&mut conn, &root, |_, _| {}).expect("scan");

        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM albums WHERE title = 'The Singles'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(n, 2, "same title, different release artists stay separate");
        let singer_b_has_it: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM albums al JOIN artists ar ON ar.id = al.artist_id
                 WHERE al.title = 'The Singles' AND ar.name = 'Singer B')",
                [],
                |r| r.get(0),
            )
            .expect("exists");
        assert!(singer_b_has_it, "untagged folder lands under its track artist");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn scans_fixtures_into_expected_shape() {
        let dbp = temp_dir("shape").join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        let counts = run_scan(&mut conn, &fixtures_src(), |_, _| {}).expect("scan");

        assert_eq!(counts.errors, Vec::<String>::new());
        assert_eq!(counts.added, 8, "all fixture files added");

        // Multi-disc ordering within one album.
        let gm = album_tracks(&conn, "Giants & Monsters");
        assert_eq!(
            gm,
            vec![
                (1, Some(1), "Silent Echoes".into()),
                (1, Some(2), "Throne of the Iron Vigil".into()),
                (2, Some(1), "Echoes of the Hollow Prophecy".into()),
            ]
        );

        // Compilation lands under Various Artists.
        let synth_artist: String = conn
            .query_row(
                "SELECT ar.name FROM albums al JOIN artists ar ON ar.id = al.artist_id
                 WHERE al.title = 'Synth Wars'",
                [],
                |r| r.get(0),
            )
            .expect("synth wars artist");
        assert_eq!(synth_artist, VARIOUS_ARTISTS_NAME);

        // Diacritic folding in sort_name.
        let bjork_sort: String = conn
            .query_row(
                "SELECT sort_name FROM artists WHERE name = 'Björk'",
                [],
                |r| r.get(0),
            )
            .expect("bjork");
        assert_eq!(bjork_sort, "bjork");

        // "The " prefix stripped for sort_name.
        let tbm_sort: String = conn
            .query_row(
                "SELECT sort_name FROM artists WHERE name = 'The Birthday Massacre'",
                [],
                |r| r.get(0),
            )
            .expect("tbm");
        assert_eq!(tbm_sort, "birthday massacre");

        // Sparse-tag wav falls back to filename-derived title.
        let wav_title: String = conn
            .query_row(
                "SELECT t.title FROM tracks t WHERE t.path LIKE '%untitled-song.wav'",
                [],
                |r| r.get(0),
            )
            .expect("wav");
        assert_eq!(wav_title, "untitled-song");

        // Durations are real (1s of silence each).
        let zero_dur: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks WHERE duration_sec <= 0", [], |r| r.get(0))
            .expect("durations");
        assert_eq!(zero_dur, 0);
        let _ = std::fs::remove_file(&dbp);
    }

    #[test]
    fn incremental_rescan_and_removal() {
        let root = temp_dir("incr");
        copy_tree(&fixtures_src(), &root);
        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");

        run_scan(&mut conn, &root, |_, _| {}).expect("first scan");
        let second = run_scan(&mut conn, &root, |_, _| {}).expect("second scan");
        assert_eq!((second.added, second.updated, second.removed), (0, 0, 0));
        assert_eq!(second.skipped, 8, "unchanged files skipped by mtime+size");

        // Touch one file's mtime → exactly one update on the next scan.
        let victim = root.join("Helloween/Giants & Monsters (2021)/01 - Silent Echoes.flac");
        let f = std::fs::File::options().append(true).open(&victim).expect("open");
        f.set_modified(std::time::SystemTime::now()).expect("touch");
        drop(f);
        let third = run_scan(&mut conn, &root, |_, _| {}).expect("third scan");
        assert_eq!((third.added, third.updated, third.removed, third.skipped), (0, 1, 0, 7));

        // Delete a file → its row is KEPT and flagged missing (relink or
        // explicit removal is the user's call); nothing auto-removed.
        std::fs::remove_file(root.join("sample/untitled-song.wav")).expect("rm");
        let fourth = run_scan(&mut conn, &root, |_, _| {}).expect("fourth scan");
        assert_eq!(fourth.removed, 0);
        assert_eq!(fourth.missing, 1, "vanished file counted as missing");
        let albums: i64 = conn
            .query_row("SELECT COUNT(*) FROM albums", [], |r| r.get(0))
            .expect("albums count");
        assert_eq!(albums, 5, "missing track keeps its album alive");
        let wav_still: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tracks WHERE path LIKE '%untitled-song.wav'",
                [],
                |r| r.get(0),
            )
            .expect("wav row");
        assert_eq!(wav_still, 1, "missing track row kept");

        let _ = std::fs::remove_dir_all(&root);
    }

    /// A retagged stray file must ADOPT the existing album with the same
    /// artist + title even though its own year differs (year is metadata,
    /// never identity) and even though the album's own files were skipped
    /// by the incremental scan. This is the tag-editor merge scenario.
    #[test]
    fn retagged_stray_adopts_existing_album_regardless_of_year() {
        let root = temp_dir("adopt");
        let src = "Various Artists/Synth Wars (2019)/02 - Chrome Sunset.flac";
        // The compilation: two tracks, albumartist "The Band", year 2019.
        stage(&root, src, "Comp", "01 - A.flac",
            Some("The Band"), "Singer One", "Anison no Kokoro", "A", Some("2019"), ("1", "1"));
        stage(&root, src, "Comp", "02 - B.flac",
            Some("The Band"), "Singer Two", "Anison no Kokoro", "B", Some("2019"), ("1", "2"));
        // The stray: own folder, album title matches, but a DIFFERENT
        // albumartist and year 2012 → first scan keys it apart (disagreeing
        // release artists never merge).
        stage(&root, src, "Stray", "01 - C.flac",
            Some("Wrong Band"), "Guest", "Anison no Kokoro", "C", Some("2012"), ("1", "1"));

        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        run_scan(&mut conn, &root, |_, _| {}).expect("first scan");
        assert_eq!(count_albums(&conn, "Anison no Kokoro"), 2, "split as staged");

        // Editor save: set the albumartist on the stray only. Its year
        // stays 2012 — adoption must not care.
        {
            let p = root.join("Stray/01 - C.flac");
            let mut tagged = lofty::read_from_path(&p).expect("read");
            let tag = tagged.primary_tag_mut().expect("tag");
            set_tag(tag, ItemKey::AlbumArtist, Some("The Band"));
            tagged
                .save_to_path(&p, lofty::config::WriteOptions::default())
                .expect("save");
        }
        run_scan(&mut conn, &root, |_, _| {}).expect("rescan");

        assert_eq!(count_albums(&conn, "Anison no Kokoro"), 1, "merged via adoption");
        assert_eq!(album_tracks(&conn, "Anison no Kokoro").len(), 3, "stray joined the compilation");
        let _ = std::fs::remove_dir_all(&root);
    }

    fn count_albums(conn: &Connection, title: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM albums WHERE title = ?1",
            [title],
            |r| r.get(0),
        )
        .expect("count")
    }

    /// Multi-root: the same album (same release artist + title) split across
    /// two library roots must unify into ONE album via Pass 3 — the scan walks
    /// all roots in one run and buckets merge across directories.
    #[test]
    fn multi_root_merge_unifies_across_roots() {
        let r1 = temp_dir("mr1");
        let r2 = temp_dir("mr2");
        let src = "Various Artists/Synth Wars (2019)/02 - Chrome Sunset.flac";
        stage(
            &r1, src, "The Artist/Merged Album", "01 - A.flac",
            Some("The Artist"), "The Artist", "Merged Album", "A", Some("2019"), ("1", "1"),
        );
        stage(
            &r2, src, "The Artist/Merged Album", "02 - B.flac",
            Some("The Artist"), "The Artist", "Merged Album", "B", Some("2019"), ("1", "2"),
        );

        let dbp = temp_dir("mr-db").join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        let counts =
            run_scan_roots(&mut conn, &[r1.clone(), r2.clone()], |_, _| {}, false).expect("scan");
        assert_eq!(counts.errors, Vec::<String>::new());

        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM albums WHERE title = 'Merged Album'",
                [],
                |r| r.get(0),
            )
            .expect("count");
        assert_eq!(n, 1, "same album across two roots merges into one");
        assert_eq!(
            album_tracks(&conn, "Merged Album").len(),
            2,
            "both tracks land under the single album"
        );
        let _ = std::fs::remove_dir_all(&r1);
        let _ = std::fs::remove_dir_all(&r2);
        let _ = std::fs::remove_file(&dbp);
    }

    /// Display/playback order: within an album disc, tracks with NO track
    /// number sort alphabetically BEFORE numbered tracks (user decision).
    #[test]
    fn unnumbered_tracks_sort_alphabetically_first() {
        let root = temp_dir("order");
        let src = "Various Artists/Synth Wars (2019)/02 - Chrome Sunset.flac";
        stage(&root, src, "Album", "01 - Zebra.flac",
            Some("The Band"), "The Band", "Order Test", "Zebra", Some("2019"), ("1", "1"));
        stage(&root, src, "Album", "02 - Mango.flac",
            Some("The Band"), "The Band", "Order Test", "Mango", Some("2019"), ("1", "2"));
        // Unnumbered: strip the track number after staging.
        for (file, title) in [("03 - Apple.flac", "Apple"), ("04 - Banana.flac", "Banana")] {
            stage(&root, src, "Album", file,
                Some("The Band"), "The Band", "Order Test", title, Some("2019"), ("1", "1"));
            let p = root.join("Album").join(file);
            let mut tagged = lofty::read_from_path(&p).expect("read");
            let tag = tagged.primary_tag_mut().expect("tag");
            set_tag(tag, ItemKey::TrackNumber, None);
            tagged
                .save_to_path(&p, lofty::config::WriteOptions::default())
                .expect("save");
        }

        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        run_scan(&mut conn, &root, |_, _| {}).expect("scan");

        let mut stmt = conn
            .prepare(
                "SELECT title FROM tracks WHERE album_id = (
                    SELECT id FROM albums WHERE title = 'Order Test')
                 ORDER BY disc, (track IS NOT NULL), track, title",
            )
            .expect("stmt");
        let titles: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .expect("q")
            .collect::<Result<_, _>>()
            .expect("rows");
        assert_eq!(
            titles,
            vec!["Apple", "Banana", "Zebra", "Mango"],
            "unnumbered alphabetical first, then numbered by track no"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}

