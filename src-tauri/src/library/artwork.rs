//! Artwork pipeline (PHASE 2 implementation record in PLAN.md §6): per album
//! with missing art/colors, resolve the cover source (folder art > largest
//! embedded picture), decode
//! ONCE, write 96/256/512 WebP thumbnails into the app cache dir and store
//! panel colors in the albums row. Albums with neither source stay NULL and
//! the UI shows its placeholder.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lofty::prelude::*;
use rusqlite::Connection;

use crate::colors;

pub const THUMB_SIZES: &[u32] = &[96, 256, 512];

/// Filenames checked inside an album's dominant directory, in priority order.
const FOLDER_ART: &[&str] = &[
    "cover.jpg", "folder.jpg", "front.jpg", "cover.png", "folder.png", "front.png",
];

pub fn thumbs_dir(cache_dir: &Path) -> PathBuf {
    cache_dir.join("thumbs")
}

pub fn thumb_path(cache_dir: &Path, album_id: &str, size: u32) -> PathBuf {
    thumbs_dir(cache_dir).join(album_id).join(format!("{size}.webp"))
}

/// The dominant directory among an album's tracks (most tracks wins; ties →
/// first seen). Cover art lives in album directories.
pub(crate) fn dominant_dir(conn: &Connection, album_id: &str) -> Option<PathBuf> {
    let mut stmt = conn
        .prepare("SELECT path FROM tracks WHERE album_id = ?1")
        .ok()?;
    let rows = stmt.query_map([album_id], |r| r.get::<_, String>(0)).ok()?;
    let mut counts: HashMap<PathBuf, usize> = HashMap::new();
    for row in rows {
        let path = row.ok()?;
        if let Some(parent) = Path::new(&path).parent() {
            *counts.entry(parent.to_path_buf()).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(dir, n)| (*n, std::cmp::Reverse(dir.clone())))
        .map(|(dir, _)| dir)
}

pub(crate) fn folder_art(dir: &Path) -> Option<PathBuf> {
    folder_art_all(dir).into_iter().next()
}

/// Every existing folder-art file in the directory, in FOLDER_ART priority
/// order. The tag editor's candidate list shows each by name; `folder_art`
/// is just the first (the winner the cover pipeline resolves to).
pub(crate) fn folder_art_all(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    // Case-insensitive match (Cover.jpg, FOLDER.PNG, …) while preserving the
    // FOLDER_ART priority order. One read_dir per album directory.
    let mut by_lower: HashMap<String, PathBuf> = HashMap::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        by_lower
            .entry(name.to_ascii_lowercase())
            .or_insert_with(|| dir.join(name));
    }
    FOLDER_ART
        .iter()
        .filter_map(|name| by_lower.get(*name).cloned())
        .filter(|p| p.is_file())
        .collect()
}

/// Largest embedded picture among the given track files (lofty Picture data).
/// Public: the artwork picker calls it with ONE audio file the user chose,
/// so "pick the cover from the album's own mp3" needs no second extractor.
pub fn embedded_art(paths: &[PathBuf]) -> Option<Vec<u8>> {
    let mut best: Option<(usize, Vec<u8>)> = None;
    for path in paths {
        let Ok(tagged) = lofty::read_from_path(path) else {
            continue;
        };
        let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) else {
            continue;
        };
        for pic in tag.pictures() {
            if best.as_ref().is_none_or(|(n, _)| pic.data().len() > *n) {
                best = Some((pic.data().len(), pic.data().to_vec()));
            }
        }
    }
    best.map(|(_, data)| data)
}

struct Job {
    album_id: String,
    source: Option<Source>,
}

enum Source {
    Folder(PathBuf),
    /// Track files to scan for the largest embedded picture.
    Embedded(Vec<PathBuf>),
}

/// Resolve one album's cover source (folder art > largest embedded),
/// shared by the bulk `refresh` and the targeted `refresh_one`.
fn album_source(conn: &Connection, album_id: &str) -> Option<Source> {
    if let Some(path) = dominant_dir(conn, album_id).and_then(|dir| folder_art(&dir)) {
        return Some(Source::Folder(path));
    }
    let mut stmt = conn
        .prepare("SELECT path FROM tracks WHERE album_id = ?1")
        .ok()?;
    let rows = stmt
        .query_map([&album_id], |r| r.get::<_, String>(0))
        .ok()?;
    Some(Source::Embedded(
        rows.flatten().map(PathBuf::from).collect(),
    ))
}

/// Decode + thumbnail + color-extract for one album. DB-free by design so it
/// can run on rayon workers (rusqlite `Connection` is not `Sync`).
fn process_album(job: &Job, cache_dir: &Path) -> Option<(String, String, String)> {
    let img = match job.source.as_ref()? {
        Source::Folder(path) => image::open(path).ok(),
        Source::Embedded(paths) => embedded_art(paths).and_then(|d| image::load_from_memory(&d).ok()),
    }?;

    let out_dir = thumbs_dir(cache_dir).join(&job.album_id);
    std::fs::create_dir_all(&out_dir).ok()?;
    for size in THUMB_SIZES {
        let thumb = img.thumbnail(*size, *size);
        thumb.save(thumb_path(cache_dir, &job.album_id, *size)).ok()?;
    }

    let colors = colors::extract_image(&img).ok()?;
    let c1 = format!("{:02x}{:02x}{:02x}", colors[0], colors[1], colors[2]);
    let c2 = format!("{:02x}{:02x}{:02x}", colors[3], colors[4], colors[5]);
    // Cache-busting lives in the URL: `<size>-<hash8>` where hash8 hashes the
    // bytes just written. The thumb protocol parses the size before the dash,
    // so old hash-less URLs keep resolving; a cover that regenerates
    // identically keeps its URL (and its webview cache), an edited one does
    // not — which is what artwork editing requires.
    let bytes = std::fs::read(thumb_path(cache_dir, &job.album_id, 512)).ok()?;
    let h8 = blake3::hash(&bytes).to_hex()[..8].to_string();
    Some((
        format!("thumb://{}/512-{}.webp", job.album_id, h8),
        c1,
        c2,
    ))
}

/// Process every album that still lacks cover or colors. Returns how many
/// albums were filled. Per-album failures are skipped silently (the row just
/// stays NULL and the UI placeholders) — a later rescan retries them.
///
/// Three phases: serial prep (ALL database access), rayon-parallel decode/
/// thumbnails/colors, serial result writes. Progress is driven by an atomic
/// counter polled from a scoped thread, so the `FnMut` callback never crosses
/// a thread boundary itself.
pub fn refresh(
    conn: &Connection,
    cache_dir: &Path,
    mut progress: impl FnMut(usize, usize) + Send,
) -> Result<usize, String> {
    use rayon::prelude::*;

    // Phase 1 (serial): resolve sources while we still own the connection.
    let pending: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT id FROM albums WHERE cover IS NULL OR color_c1 IS NULL")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?
    };
    let total = pending.len();

    let mut jobs: Vec<Job> = Vec::new();
    for album_id in pending {
        if let Some(source) = album_source(conn, &album_id) {
            jobs.push(Job {
                album_id,
                source: Some(source),
            });
        }
    }

    // Phase 2 (parallel): decode, write thumbs, extract colors.
    use std::sync::atomic::{AtomicUsize, Ordering};
    let done = AtomicUsize::new(0);
    let done = &done; // shared across the poller thread and rayon workers
    let results: Vec<(String, Option<(String, String, String)>)> = std::thread::scope(|scope| {
        let progress = &mut progress;
        scope.spawn(move || {
            let mut last = 0usize;
            loop {
                let cur = done.load(Ordering::Relaxed);
                if cur != last {
                    last = cur;
                    progress(cur, total);
                }
                if cur >= total {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(80));
            }
        });
        jobs.into_par_iter()
            .map(|job| {
                let out = process_album(&job, cache_dir);
                done.fetch_add(1, Ordering::Relaxed);
                (job.album_id, out)
            })
            .collect()
    });

    // Phase 3 (serial): apply results to the database.
    let mut filled = 0;
    for (album_id, result) in results {
        let Some((cover, c1, c2)) = result else {
            continue;
        };
        if conn
            .execute(
                "UPDATE albums SET cover = ?2, color_c1 = ?3, color_c2 = ?4 WHERE id = ?1",
                rusqlite::params![album_id, cover, c1, c2],
            )
            .is_ok()
        {
            filled += 1;
        }
    }
    Ok(filled)
}

/// Re-run the artwork pipeline for ONE album regardless of what the DB
/// believes it already has — the path an artwork edit (or a cleared one)
/// takes. Old thumbs are deleted so stale sizes cannot linger; an album left
/// with no art source at all has its cover/colors NULLed, and the UI
/// placeholder returns honestly.
pub fn refresh_one(conn: &Connection, cache_dir: &Path, album_id: &str) -> Result<(), String> {
    let known: bool = conn
        .query_row("SELECT 1 FROM albums WHERE id = ?1", [album_id], |_| Ok(true))
        .unwrap_or(false);
    if !known {
        return Err(format!("unknown album {album_id}"));
    }
    let _ = std::fs::remove_dir_all(thumbs_dir(cache_dir).join(album_id));
    let job = Job {
        album_id: album_id.to_string(),
        source: album_source(conn, album_id),
    };
    let (cover, c1, c2) = match process_album(&job, cache_dir) {
        Some(v) => (Some(v.0), Some(v.1), Some(v.2)),
        None => (None, None, None),
    };
    conn.execute(
        "UPDATE albums SET cover = ?2, color_c1 = ?3, color_c2 = ?4 WHERE id = ?1",
        rusqlite::params![album_id, cover, c1, c2],
    )
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Delete per-album thumbnail dirs that no longer belong to any album row.
/// Albums die — tracks discarded, retags that resolve to a different key,
/// dropped staged imports — and the scan's orphan cleanup handles the DB
/// side; the dirs under `thumbs/` were the leak. The thumbs dir is OURS:
/// every unknown child is a dead album or junk, so all of them go. Cheap
/// enough to run at the end of every scan.
pub fn prune_orphan_thumbs(conn: &Connection, cache_dir: &Path) {
    let dir = thumbs_dir(cache_dir);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    let live: std::collections::HashSet<String> = conn
        .prepare("SELECT id FROM albums")
        .and_then(|mut s| {
            s.query_map([], |r| r.get::<_, String>(0))
                .map(|rows| rows.filter_map(Result::ok).collect())
        })
        .unwrap_or_default();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if !live.contains(&name) {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{db, scan};
    use std::path::PathBuf;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("songstress-m3-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        dir
    }

    #[test]
    fn refresh_fills_folder_embedded_and_leaves_placeholders() {
        let root = temp_dir("root");
        // Private copy of the fixture tree.
        for entry in walkdir::WalkDir::new("fixtures/library") {
            let entry = entry.expect("walk");
            let rel = entry.path().strip_prefix("fixtures/library").expect("prefix");
            let target = root.join(rel);
            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&target).expect("mkdir");
            } else {
                std::fs::copy(entry.path(), &target).expect("copy");
            }
        }

        let dbp = root.join("t.db");
        let mut conn = db::open(&dbp).expect("open");
        scan::run_scan(&mut conn, &root, |_, _| {}).expect("scan");

        let cache = temp_dir("cache");
        let filled = refresh(&conn, &cache, |_, _| {}).expect("refresh");
        assert_eq!(filled, 2, "folder-art album + embedded-art album filled");

        // Folder art album: cover URL + colors + real files on disk.
        let (cover, c1): (String, String) = conn
            .query_row(
                "SELECT cover, color_c1 FROM albums WHERE title = 'Giants & Monsters'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("folder row");
        assert!(cover.starts_with("thumb://al-"), "cover = {cover}");
        assert!(
            cover.contains("/512-") && cover.ends_with(".webp"),
            "hashed cache-busting URL: {cover}"
        );
        assert_eq!(c1.len(), 6);
        let album_id = cover
            .trim_start_matches("thumb://")
            .split('/')
            .next()
            .expect("id in url")
            .to_string();
        for size in super::THUMB_SIZES {
            assert!(
                super::thumb_path(&cache, &album_id, *size).is_file(),
                "missing {size}.webp"
            );
        }

        // Embedded art album (Synth Wars) also filled.
        let synth_cover: String = conn
            .query_row(
                "SELECT cover FROM albums WHERE title = 'Synth Wars'",
                [],
                |r| r.get(0),
            )
            .expect("synth row");
        assert!(synth_cover.starts_with("thumb://al-"));

        // Albums with no art source stay NULL (placeholder path).
        let nulls: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM albums WHERE cover IS NULL AND color_c1 IS NULL",
                [],
                |r| r.get(0),
            )
            .expect("nulls");
        assert_eq!(nulls, 3, "Debut + TBM + wav's Unknown Album stay NULL");

        // Idempotent: a second refresh finds nothing to do.
        let again = refresh(&conn, &cache, |_, _| {}).expect("refresh 2");
        assert_eq!(again, 0);

        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&cache);
    }

    const ONE_BY_ONE_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
        0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
        0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78,
        0x9C, 0x62, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
        0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn folder_art_matches_case_insensitively() {
        let dir = temp_dir("case");

        // Only an uppercase Cover.JPG exists → found.
        std::fs::write(dir.join("Cover.JPG"), ONE_BY_ONE_PNG).expect("write");
        assert_eq!(
            folder_art(&dir),
            Some(dir.join("Cover.JPG")),
            "uppercase name matches the lowercase candidate"
        );

        // A lower-priority exact-name candidate must NOT outrank it
        // (cover.jpg has priority over folder.jpg regardless of case).
        std::fs::write(dir.join("folder.jpg"), ONE_BY_ONE_PNG).expect("write");
        assert_eq!(folder_art(&dir), Some(dir.join("Cover.JPG")));

        // A same-key duplicate with different casing is equally valid —
        // whichever read_dir lists first wins; both are "cover.jpg".

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn refresh_finds_uppercase_cover_jpg() {
        let root = temp_dir("upper");
        for entry in walkdir::WalkDir::new("fixtures/library") {
            let entry = entry.expect("walk");
            let rel = entry.path().strip_prefix("fixtures/library").expect("prefix");
            let target = root.join(rel);
            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&target).expect("mkdir");
            } else {
                std::fs::copy(entry.path(), &target).expect("copy");
            }
        }
        // Replace the album's folder.jpg with a differently-cased Cover.JPG —
        // real libraries have both spellings.
        let gm = root.join("Helloween/Giants & Monsters (2021)");
        std::fs::remove_file(gm.join("folder.jpg")).expect("rm folder art");
        std::fs::copy(
            "fixtures/library/Helloween/Giants & Monsters (2021)/folder.jpg",
            gm.join("Cover.JPG"),
        )
        .expect("write Cover.JPG");

        let dbp = root.join("t.db");
        let mut conn = db::open(&dbp).expect("open");
        scan::run_scan(&mut conn, &root, |_, _| {}).expect("scan");

        let cache = temp_dir("upper-cache");
        let filled = refresh(&conn, &cache, |_, _| {}).expect("refresh");
        assert_eq!(filled, 2, "Cover.JPG counts as folder art");

        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&cache);
    }

    #[test]
    fn prune_removes_dirs_of_dead_albums_only() {
        let root = temp_dir("prune-root");
        for entry in walkdir::WalkDir::new("fixtures/library") {
            let entry = entry.expect("walk");
            let rel = entry.path().strip_prefix("fixtures/library").expect("prefix");
            let target = root.join(rel);
            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&target).expect("mkdir");
            } else {
                std::fs::copy(entry.path(), &target).expect("copy");
            }
        }
        let dbp = root.join("t.db");
        let mut conn = db::open(&dbp).expect("open");
        scan::run_scan(&mut conn, &root, |_, _| {}).expect("scan");
        let cache = temp_dir("prune-cache");
        refresh(&conn, &cache, |_, _| {}).expect("refresh");

        let live: String = conn
            .query_row("SELECT id FROM albums WHERE cover IS NOT NULL", [], |r| {
                r.get(0)
            })
            .expect("a thumb-bearing album");
        let ghost = thumbs_dir(&cache).join("al-deadbeef");
        std::fs::create_dir_all(&ghost).expect("ghost dir");
        std::fs::write(ghost.join("512.webp"), b"x").expect("ghost file");
        assert!(thumbs_dir(&cache).join(&live).exists(), "live thumbs on disk");

        prune_orphan_thumbs(&conn, &cache);
        assert!(
            thumbs_dir(&cache).join(&live).exists(),
            "live album thumbs survive"
        );
        assert!(!ghost.exists(), "dead album dir pruned");

        // The leak itself: rows die (discards, retags to a new key), the
        // dir must go with them on the next prune.
        conn.execute("DELETE FROM tracks WHERE album_id = ?1", [&live])
            .expect("drop tracks");
        conn.execute("DELETE FROM albums WHERE id = ?1", [&live])
            .expect("drop album");
        prune_orphan_thumbs(&conn, &cache);
        assert!(
            !thumbs_dir(&cache).join(&live).exists(),
            "pruned once the row dies"
        );

        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&cache);
    }
}
