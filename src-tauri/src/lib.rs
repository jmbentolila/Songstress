use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use tauri::{Emitter, Manager};

mod eq;
mod library;
mod menu;
mod menu_dbus;
mod mpris;
mod mpv;
mod wayland_appmenu;
mod watcher;

// Shared DB connection. Commands run on the async runtime pool and take the
// mutex briefly — SQLite work here is microseconds (settings/lookups); the
// long scan job opens its OWN connection from db_path so it never blocks
// settings reads.
struct AppState {
    db_path: PathBuf,
    db: Mutex<rusqlite::Connection>,
}

/// Handle to the MPV playback engine (Phase 3), spawned once at setup.
pub struct Engine(pub std::sync::Arc<mpv::Mpv>);

/// One track of the library with its album context — the raw material for
/// play-order building.
struct PoolTrack {
    track_id: String,
    path: String,
    album_id: String,
    album_index: usize,
}

/// Load every track (with album context) for the given album ids, ordered by
/// album year then disc/track. `album_index` = position within its album.
fn pool_tracks(
    conn: &rusqlite::Connection,
    album_ids: &[String],
) -> Result<Vec<PoolTrack>, String> {
    let mut out = Vec::new();
    for album_id in album_ids {
        let rows: Vec<(String, String)> = {
            let mut stmt = conn
                .prepare(
                    "SELECT id, path FROM tracks WHERE album_id = ?1
                     ORDER BY disc, (track IS NOT NULL), track, title",
                )
                .map_err(|e| e.to_string())?;
            let mapped = stmt.query_map([album_id], |r| Ok((r.get(0)?, r.get(1)?)));
            match mapped {
                Ok(rows) => rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?,
                Err(e) => return Err(e.to_string()),
            }
        };
        for (album_index, (track_id, path)) in rows.into_iter().enumerate() {
            out.push(PoolTrack {
                track_id,
                path,
                album_id: album_id.clone(),
                album_index,
            });
        }
    }
    Ok(out)
}

/// Build the play order for a click on `pool[clicked]` under the current
/// shuffle stage: Off = natural order; Album/Artist/All = clicked track
/// first, remainder Fisher-Yates shuffled.
fn build_order(
    mut pool: Vec<PoolTrack>,
    clicked: usize,
    shuffle: mpv::ShuffleStage,
) -> (Vec<mpv::OrderItem>, usize) {
    if shuffle == mpv::ShuffleStage::Off {
        let order = pool
            .into_iter()
            .map(|t| mpv::OrderItem {
                track_id: t.track_id,
                path: t.path,
                album_id: t.album_id,
                album_index: t.album_index,
            })
            .collect();
        return (order, clicked);
    }
    let anchor = pool.remove(clicked);
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(7);
    mpv::Rng::shuffle(&mut pool, seed);
    let mut order = vec![mpv::OrderItem {
        track_id: anchor.track_id,
        path: anchor.path,
        album_id: anchor.album_id,
        album_index: anchor.album_index,
    }];
    order.extend(pool.into_iter().map(|t| mpv::OrderItem {
        track_id: t.track_id,
        path: t.path,
        album_id: t.album_id,
        album_index: t.album_index,
    }));
    (order, 0)
}

#[tauri::command]
async fn play_album(
    state: tauri::State<'_, AppState>,
    engine: tauri::State<'_, Engine>,
    album_id: String,
    track_index: usize,
) -> Result<(), String> {
    // Pool per shuffle stage: off = the album, album = the album (shuffled),
    // artist = everything by the album's artist, all = the whole library.
    // Read the stage BEFORE taking the DB lock — play_album's lock order is
    // engine→db nowhere / db-only, and inverting it against
    // playback_set_shuffle (engine→db) risks deadlock.
    let shuffle = engine.0.state.lock().unwrap().shuffle;
    let (pool, clicked) = {
        let conn = state.db.lock().unwrap();
        let album_ids: Vec<String> = match shuffle {
            mpv::ShuffleStage::Off | mpv::ShuffleStage::Album => vec![album_id.clone()],
            mpv::ShuffleStage::Artist => {
                let mut stmt = conn
                    .prepare(
                        "SELECT al.id FROM albums al
                         JOIN artists ar ON ar.id = al.artist_id
                         WHERE ar.id = (SELECT artist_id FROM albums WHERE id = ?1)
                         ORDER BY (al.year IS NULL), al.year, al.title",
                    )
                    .map_err(|e| e.to_string())?;
                let mapped = stmt.query_map([&album_id], |r| r.get::<_, String>(0));
                match mapped {
                    Ok(rows) => rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?,
                    Err(e) => return Err(e.to_string()),
                }
            }
            mpv::ShuffleStage::All => {
                let mut stmt = conn
                    .prepare(
                        "SELECT al.id FROM albums al
                         JOIN artists ar ON ar.id = al.artist_id
                         ORDER BY ar.sort_name, (al.year IS NULL), al.year, al.title",
                    )
                    .map_err(|e| e.to_string())?;
                let mapped = stmt.query_map([], |r| r.get::<_, String>(0));
                match mapped {
                    Ok(rows) => rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?,
                    Err(e) => return Err(e.to_string()),
                }
            }
        };
        let pool = pool_tracks(&conn, &album_ids)?;
        let clicked = match shuffle {
            mpv::ShuffleStage::Off => track_index,
            _ => pool.iter().position(|t| t.album_id == album_id && t.album_index == track_index)
                .ok_or("clicked track not in pool")?,
        };
        (pool, clicked)
    };
    if pool.is_empty() {
        return Err("unknown or empty album".into());
    }
    // A row can outlive its file (deleted outside the app, no rescan yet).
    // Refuse cleanly instead of ghost-playing: mpv would error-skip to the
    // next track while the UI still shows the clicked one as current.
    if !Path::new(&pool[clicked].path).exists() {
        return Err("this track's file is missing on disk — rescan the library".into());
    }
    let (order, start) = build_order(pool, clicked, shuffle);
    engine.0.play_order(order, start).await
}

/// Build a shuffle-anchored order for a MID-PLAYBACK stage change: the
/// current track first, then the new stage's pool shuffled. Returns None
/// when nothing is playing (nothing to anchor).
fn anchored_order(
    conn: &rusqlite::Connection,
    current_track_id: &str,
    current_album_id: &str,
    shuffle: mpv::ShuffleStage,
) -> Result<Option<(Vec<mpv::OrderItem>, usize)>, String> {
    let album_ids: Vec<String> = match shuffle {
        mpv::ShuffleStage::Album => vec![current_album_id.to_string()],
        mpv::ShuffleStage::Artist => {
            let mut stmt = conn
                .prepare(
                    "SELECT al.id FROM albums al
                     WHERE al.artist_id = (SELECT artist_id FROM albums WHERE id = ?1)
                     ORDER BY (al.year IS NULL), al.year, al.title",
                )
                .map_err(|e| e.to_string())?;
            let mapped = stmt.query_map([current_album_id], |r| r.get::<_, String>(0));
            match mapped {
                Ok(rows) => rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?,
                Err(e) => return Err(e.to_string()),
            }
        }
        mpv::ShuffleStage::All => {
            let mut stmt = conn
                .prepare(
                    "SELECT al.id FROM albums al
                     JOIN artists ar ON ar.id = al.artist_id
                     ORDER BY ar.sort_name, (al.year IS NULL), al.year, al.title",
                )
                .map_err(|e| e.to_string())?;
            let mapped = stmt.query_map([], |r| r.get::<_, String>(0));
            match mapped {
                Ok(rows) => rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?,
                Err(e) => return Err(e.to_string()),
            }
        }
        mpv::ShuffleStage::Off => {
            // Back to plain album order, positioned at the current track.
            vec![current_album_id.to_string()]
        }
    };
    let pool = pool_tracks(conn, &album_ids)?;
    let clicked = pool
        .iter()
        .position(|t| t.track_id == current_track_id)
        .ok_or("playing track vanished from the pool")?;
    Ok(Some(build_order(pool, clicked, shuffle)))
}

#[tauri::command]
async fn playback_set_shuffle(
    state: tauri::State<'_, AppState>,
    engine: tauri::State<'_, Engine>,
    stage: String,
) -> Result<(), String> {
    let stage = mpv::ShuffleStage::parse(&stage);
    let anchored = {
        let st = engine.0.state.lock().unwrap();
        match st.current() {
            Some(item) => {
                let conn = state.db.lock().unwrap();
                Some(anchored_order(&conn, &item.track_id, &item.album_id, stage)?)
            }
            None => None,
        }
    };
    engine.0.set_shuffle(stage, anchored.flatten()).await?;
    let conn = state.db.lock().unwrap();
    library::settings::set(&conn, "shuffleStage", &serde_json::json!(stage.as_str()).to_string())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn playback_set_repeat(
    state: tauri::State<'_, AppState>,
    engine: tauri::State<'_, Engine>,
    stage: String,
) -> Result<(), String> {
    let stage = mpv::RepeatStage::parse(&stage);
    engine.0.set_repeat(stage).await?;
    let conn = state.db.lock().unwrap();
    library::settings::set(&conn, "repeatStage", &serde_json::json!(stage.as_str()).to_string())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn playback_pause(engine: tauri::State<'_, Engine>, paused: bool) -> Result<(), String> {
    engine.0.set_paused(paused).await
}

/// Cycle pause on the CURRENT engine state (frontend mirrors it from events).
#[tauri::command]
async fn playback_toggle(engine: tauri::State<'_, Engine>) -> Result<bool, String> {
    let paused = !engine.0.state.lock().unwrap().paused;
    engine.0.set_paused(paused).await?;
    Ok(paused)
}

#[tauri::command]
async fn playback_jump(engine: tauri::State<'_, Engine>, delta: i64) -> Result<(), String> {
    engine.0.jump(delta).await
}

#[tauri::command]
async fn playback_seek(engine: tauri::State<'_, Engine>, sec: f64) -> Result<(), String> {
    engine.0.seek_to(sec).await
}

#[tauri::command]
async fn playback_volume(engine: tauri::State<'_, Engine>, vol: f64) -> Result<(), String> {
    engine.0.set_volume(vol).await
}

#[tauri::command]
async fn playback_stop(engine: tauri::State<'_, Engine>) -> Result<(), String> {
    engine.0.stop().await
}

/// Look up tracks by id (input order preserved, unknown ids skipped) as
/// queue items. album_index is the position within the track's OWN album
/// (event contract), so each affected album is re-listed in canonical order.
fn tracks_by_ids(
    conn: &rusqlite::Connection,
    track_ids: &[String],
) -> Result<Vec<mpv::OrderItem>, String> {
    let mut out = Vec::new();
    for id in track_ids {
        let row: Option<(String, String)> = {
            let mut stmt = conn
                .prepare("SELECT path, album_id FROM tracks WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            let mut mapped = stmt.query_map([id], |r| Ok((r.get(0)?, r.get(1)?))).map_err(|e| e.to_string())?;
            match mapped.next() {
                Some(r) => Some(r.map_err(|e| e.to_string())?),
                None => continue,
            }
        };
        let Some((_, album_id)) = row else { continue };
        let album = pool_tracks(conn, &[album_id])?;
        if let Some(t) = album.iter().find(|t| t.track_id == *id) {
            out.push(mpv::OrderItem {
                track_id: t.track_id.clone(),
                path: t.path.clone(),
                album_id: t.album_id.clone(),
                album_index: t.album_index,
            });
        }
    }
    Ok(out)
}

/// Queue tracks (Step 7a): front = "Play next", back = "Add to queue".
#[tauri::command]
async fn playback_queue(
    state: tauri::State<'_, AppState>,
    engine: tauri::State<'_, Engine>,
    track_ids: Vec<String>,
    front: bool,
) -> Result<(), String> {
    let items = {
        let conn = state.db.lock().unwrap();
        tracks_by_ids(&conn, &track_ids)?
    };
    if items.is_empty() {
        return Err("no such tracks".into());
    }
    engine.0.queue_tracks(items, front).await
}

#[tauri::command]
async fn playback_queue_remove(
    engine: tauri::State<'_, Engine>,
    pos: usize,
) -> Result<(), String> {
    engine.0.queue_remove(pos).await
}

/// Play the queued entry at `pos` now (entries before it are discarded).
#[tauri::command]
async fn playback_queue_jump(engine: tauri::State<'_, Engine>, pos: usize) -> Result<(), String> {
    engine.0.queue_jump(pos).await
}

/// Clear the entire user queue (popover "Clear").
#[tauri::command]
async fn playback_queue_clear(engine: tauri::State<'_, Engine>) -> Result<(), String> {
    engine.0.queue_clear().await
}

/// Apply the equalizer (Step 6): push the lavfi chain onto mpv's `af` and
/// persist the (clamped) state in settings — Rust re-applies it at launch,
/// so this is the single write path. Returns the normalized state so the UI
/// round-trips exactly what took effect.
#[tauri::command]
async fn playback_eq(
    state: tauri::State<'_, AppState>,
    engine: tauri::State<'_, Engine>,
    eq: eq::Eq,
) -> Result<eq::Eq, String> {
    let mut eq = eq;
    eq.preamp_db = eq::clamp_db(eq.preamp_db);
    for g in &mut eq.gains {
        *g = eq::clamp_db(*g);
    }
    engine.0.set_af(eq::af_chain(&eq)).await?;
    let conn = state.db.lock().unwrap();
    library::settings::set(
        &conn,
        "equalizer",
        &serde_json::to_string(&eq).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(eq)
}

static SCAN_RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Step 7d: set by the file watcher's debounce callback, consumed (and
/// cleared) by the watch-scan loop. Events arriving DURING a scan keep it
/// set, so one more scan runs after the in-flight one settles.
static WATCH_DIRTY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Minimum spacing between watcher-triggered rescans (bulk copies fire a
/// stream of debounced bursts — don't thrash the scanner).
const WATCH_MIN_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

#[derive(serde::Serialize)]
struct ScanSummary {
    added: usize,
    updated: usize,
    removed: usize,
    missing: usize,
    skipped: usize,
    errors: Vec<String>,
}

// --- Scanner (Phase 2 M2) + import staging (Step 2a) ------------------------

/// Shared scan pipeline: multi-root scan + artwork post-pass + events.
/// Callers must NOT hold SCAN_RUNNING (this takes and releases it).
async fn run_library_scan(
    app: tauri::AppHandle,
    db_path: PathBuf,
    cache_dir: PathBuf,
    roots: Vec<PathBuf>,
    full: bool,
) -> Result<ScanSummary, String> {
    use std::sync::atomic::Ordering;
    if SCAN_RUNNING.swap(true, Ordering::SeqCst) {
        return Err("scan already running".into());
    }
    let emitter = app.clone();
    // JoinHandle awaited WITHOUT `?` before the flag reset — a panicking scan
    // task must not leave SCAN_RUNNING stuck true (every later scan would
    // then fail instantly with "scan already running").
    let result = tauri::async_runtime::spawn_blocking(move || {
        let mut conn = library::db::open(&db_path).map_err(|e| e.to_string())?;
        let t_files = std::time::Instant::now();
        let counts = library::scan::run_scan_roots(&mut conn, &roots, |done, total| {
            let _ = emitter.emit("scan-progress", serde_json::json!({ "done": done, "total": total }));
        }, full)?;
        let files_elapsed = t_files.elapsed();
        // M3 post-pass: thumbs + colors for albums still missing them.
        // Shares the progress channel; cheap when everything is filled.
        let t_art = std::time::Instant::now();
        library::artwork::refresh(&conn, &cache_dir, |done, total| {
            let _ = emitter.emit("scan-progress", serde_json::json!({ "phase": "artwork", "done": done, "total": total }));
        })?;
        eprintln!(
            "[scan] files {:?} (added {} updated {} removed {} skipped {}) · artwork {:?}",
            files_elapsed, counts.added, counts.updated, counts.removed, counts.skipped,
            t_art.elapsed()
        );
        eprintln!(
            "[scan] missing {} (kept for relink/removal)",
            counts.missing
        );
        Ok::<library::scan::ScanCounts, String>(counts)
    })
    .await;

    SCAN_RUNNING.store(false, Ordering::SeqCst);

    let counts = result.map_err(|e| e.to_string())??;
    let summary = ScanSummary {
        added: counts.added,
        updated: counts.updated,
        removed: counts.removed,
        missing: counts.missing,
        skipped: counts.skipped,
        errors: counts.errors.clone(),
    };
    let _ = app.emit(
        "scan-finished",
        serde_json::to_value(&summary).unwrap_or_default(),
    );
    Ok(summary)
}

/// Step 7c: all library roots. `musicDirs` (JSON array) is AUTHORITATIVE
/// once present (an empty list = no roots = empty library); before the
/// one-time setup migration writes it, the legacy single `musicDir` (incl.
/// the JSON "null" footgun: null/empty = unset) falls back to ~/Music.
fn roots_from_settings(all: &std::collections::HashMap<String, String>) -> Vec<PathBuf> {
    if let Some(v) = all.get("musicDirs") {
        return serde_json::from_str::<Vec<String>>(v)
            .unwrap_or_default()
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .map(PathBuf::from)
            .collect();
    }
    let legacy = all
        .get("musicDir")
        .cloned()
        .and_then(|v| serde_json::from_str::<Option<String>>(&v).ok().flatten())
        .filter(|s| !s.trim().is_empty());
    vec![legacy.map(PathBuf::from).unwrap_or_else(|| dirs_home().join("Music"))]
}

fn music_roots(state: &AppState) -> Vec<PathBuf> {
    roots_from_settings(&library::settings::all(&state.db.lock().unwrap()).ok().unwrap_or_default())
}

/// Primary root: import-save target, kdialog start dir, first watch root.
fn music_root(state: &AppState) -> PathBuf {
    music_roots(state)
        .into_iter()
        .next()
        .unwrap_or_else(|| dirs_home().join("Music"))
}

/// Scan roots: ALL library dirs (Step 7c) plus the import staging area
/// (when present, and not itself a library root).
fn scan_roots(state: &AppState, cache_dir: &Path, override_root: Option<String>) -> Vec<PathBuf> {
    let mut roots = match override_root {
        Some(r) => vec![r.into()],
        None => music_roots(state),
    };
    let import = library::import::import_dir(cache_dir);
    if import.is_dir() && !roots.iter().any(|r| r == &import) {
        roots.push(import);
    }
    roots
}

#[tauri::command]
// `full` = reparse every file even if (mtime, size) is unchanged — needed
// after grouping/tag-consensus logic changes (skipped files never regroup).
async fn scan_library(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    root: Option<String>,
    full: Option<bool>,
) -> Result<ScanSummary, String> {
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?;
    let roots = scan_roots(&state, &cache_dir, root);
    run_library_scan(app, state.db_path.clone(), cache_dir, roots, full.unwrap_or(false)).await
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

// Full-res cover decode is expensive (some art is 3000px); cache per path so
// repeated panel opens/switches don't re-decode.
static COLOR_CACHE: LazyLock<Mutex<HashMap<PathBuf, [u8; 6]>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) mod colors {
    use std::collections::HashMap;

    use image::GenericImageView;

    type Rgb = [u8; 3];

    fn dominant_pair(img: &image::DynamicImage) -> Option<(Rgb, Rgb)> {
        let thumb = img.thumbnail(48, 48);
        let mut wsum = 0u64;
        let mut sr = 0u64;
        let mut sg = 0u64;
        let mut sb = 0u64;
        let mut counts: HashMap<Rgb, u64> = HashMap::new();
        for (_x, _y, p) in thumb.pixels() {
            let [r, g, b, a] = p.0;
            if a < 128 {
                continue;
            }
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let chroma = max - min;
            // Every pixel counts so the primary color represents the whole
            // artwork; vivid regions simply pull harder than dark fill.
            let w = 4u64 + (chroma as u64) * (chroma as u64) / 16;
            wsum += w;
            sr += w * r as u64;
            sg += w * g as u64;
            sb += w * b as u64;
            let key = [r >> 4, g >> 4, b >> 4];
            *counts.entry(key).or_insert(0) += w;
        }
        if wsum == 0 {
            return None;
        }
        let c1: Rgb = [(sr / wsum) as u8, (sg / wsum) as u8, (sb / wsum) as u8];

        let to_rgb = |k: Rgb| {
            [
                ((u16::from(k[0]) * 17 + 8).min(255)) as u8,
                ((u16::from(k[1]) * 17 + 8).min(255)) as u8,
                ((u16::from(k[2]) * 17 + 8).min(255)) as u8,
            ]
        };
        let dist = |a: Rgb, b: Rgb| -> u32 {
            a.iter()
                .zip(b.iter())
                .map(|(x, y)| (i32::from(*x) - i32::from(*y)).unsigned_abs())
                .sum()
        };

        // Merge near-identical buckets so spread-out regions (e.g. shaded
        // hair across many buckets) count as one significant color family.
        let mut buckets: Vec<(Rgb, u64)> =
            counts.into_iter().map(|(k, w)| (to_rgb(k), w)).collect();
        buckets.sort_by(|a, b| b.1.cmp(&a.1));
        let mut clusters: Vec<(Rgb, u64)> = Vec::new();
        for (color, weight) in buckets {
            if let Some(entry) = clusters
                .iter_mut()
                .find(|(c, _)| dist(*c, color) < 64)
            {
                let total = entry.1 + weight;
                entry.0 = [
                    ((entry.0[0] as u64 * entry.1 + color[0] as u64 * weight) / total) as u8,
                    ((entry.0[1] as u64 * entry.1 + color[1] as u64 * weight) / total) as u8,
                    ((entry.0[2] as u64 * entry.1 + color[2] as u64 * weight) / total) as u8,
                ];
                entry.1 = total;
            } else {
                clusters.push((color, weight));
            }
        }

        let top_weight = clusters.first().map(|(_, w)| *w).unwrap_or(0);

        // A busy artwork can average out to near-gray (e.g. blue night sky in
        // the top half, white snow in the bottom). If the whole-artwork
        // average lost its chroma, anchor the gradient's primary on the
        // heaviest significant color family instead — the one the eye calls
        // "the color" of this artwork.
        let chroma = |c: Rgb| -> u8 {
            let max = c[0].max(c[1]).max(c[2]);
            let min = c[0].min(c[1]).min(c[2]);
            max - min
        };
        let mut c1_final = c1;
        if chroma(c1) < 32 {
            if let Some((c, _)) = clusters
                .iter()
                .filter(|(c, w)| *w * 4 >= top_weight && chroma(*c) >= 48)
                .max_by_key(|(_, w)| *w)
            {
                c1_final = *c;
            }
        }

        // Accent: among significant clusters (>= 25% of the heaviest),
        // take the one perceptually farthest from the primary.
        let mut best: Option<Rgb> = None;
        let mut best_dist = 60u32;
        for (c, w) in &clusters {
            if *w * 4 >= top_weight {
                let d = dist(*c, c1_final);
                if d > best_dist {
                    best_dist = d;
                    best = Some(*c);
                }
            }
        }
        let c2 = best.unwrap_or_else(|| [
                c1_final[0].saturating_add(48),
                c1_final[1].saturating_add(44),
                c1_final[2].saturating_add(56),
            ]);
        Some((c1_final, c2))
    }

    pub fn extract(path: &std::path::Path) -> Result<[u8; 6], String> {
        let img = image::open(path).map_err(|e| e.to_string())?;
        extract_image(&img)
    }

    /// Same algorithm on an already-decoded image — the scanner (M3) feeds
    /// decoded cover sources here so thumbs and colors share one decode.
    pub fn extract_image(img: &image::DynamicImage) -> Result<[u8; 6], String> {
        match dominant_pair(img) {
            Some((c1, c2)) => Ok([c1[0], c1[1], c1[2], c2[0], c2[1], c2[2]]),
            None => Err("no usable pixels".into()),
        }
    }
}

fn covers_dir(app: &tauri::AppHandle) -> PathBuf {
    if cfg!(debug_assertions) {
        std::env::current_dir()
            .map(|p| p.join("../public/covers"))
            .expect("cwd")
    } else {
        app.path()
            .resource_dir()
            .expect("resource dir")
            .join("covers")
    }
}

// Async so the blocking JPEG decode runs on the runtime pool instead of the
// main thread — a sync command here stalled the whole UI on album switches.
#[tauri::command]
async fn album_colors(app: tauri::AppHandle, file: String) -> Result<[u8; 6], String> {
    if file.contains('/') || file.contains('\\') || file.contains("..") {
        return Err("invalid file name".into());
    }
    let path = covers_dir(&app).join(file);
    if let Some(cached) = COLOR_CACHE.lock().unwrap().get(&path) {
        return Ok(*cached);
    }
    let decode_path = path.clone();
    let out = tauri::async_runtime::spawn_blocking(move || colors::extract(&decode_path))
        .await
        .map_err(|e| e.to_string())??;
    COLOR_CACHE.lock().unwrap().insert(path, out);
    Ok(out)
}

// Self-healing for the WebKitGTK stale-mapping glitch: the frontend calls
// this when its viewport size disagrees with the actual window size. A 1px
// shrink+restore forces the reconfigure that remaps the webview correctly.
#[tauri::command]
fn fix_viewport(app: tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if let Ok(size) = win.inner_size() {
            let _ = win.set_size(tauri::PhysicalSize::new(
                size.width,
                size.height.saturating_sub(1),
            ));
            std::thread::sleep(std::time::Duration::from_millis(80));
            let _ = win.set_size(size);
        }
    }
}

// --- KDE decoration-aware titlebar -----------------------------------------
// The app draws its own decorations, so it must mirror what the user's KWin
// decoration would have done: button layout from kwinrc ([org.kde.kdecoration2]
// ButtonsOnLeft/Right) and button appearance from Klassy's config when present.
// Everything degrades to built-in defaults if the files/keys don't exist.

#[derive(serde::Serialize, Clone, Copy, PartialEq, Debug)]
struct DecoButton {
    normal: [u8; 3],
    hover: [u8; 3],
}

#[derive(serde::Serialize)]
struct WindowDecoration {
    /// Raw KWin button letters ("XIA"), left side. Frontend filters to the
    /// subset it can honor (X/I/A); Shade etc. have no CSD equivalent.
    buttons_left: String,
    buttons_right: String,
    close: DecoButton,
    minimize: DecoButton,
    maximize: DecoButton,
    /// Background opacity in percent, as configured in the decoration theme.
    bg_opacity_active: u8,
    bg_opacity_inactive: u8,
    /// Geometry, so the cluster is not merely coloured like the user's
    /// decoration but sized and spaced like it: visible dot, gap between dots,
    /// left margin of the titlebar, window corner radius.
    dot_size: u8,
    button_gap: u8,
    margin_left: u8,
    corner_radius: u8,
}

/// The button square Klassy lays out for each standard icon tier (KDE's
/// StandardGuiSizes). The small circle a shape draws fills the rect minus 1px —
/// calibrated against the user's own render: at screen scale 1.6 Konsole's
/// Klassy dots measure 24x24 px = 15 logical, on a 42px = 26 logical pitch,
/// which is rect 16 + `ButtonSpacingLeft=10`.
fn klassy_rect(icon_size: Option<&String>) -> u8 {
    match icon_size.map(|s| s.as_str()) {
        Some("IconTiny") => 12,
        Some("IconMedium") => 24,
        Some("IconLarge") => 32,
        Some("IconHuge") => 48,
        _ => 16, // IconSmall, and the fallback for an unknown tier
    }
}

#[cfg(test)]
mod deco_geometry {
    use super::klassy_rect;
    fn tier(t: &str) -> Option<String> {
        Some(t.to_string())
    }
    #[test]
    fn button_rect_follows_the_configured_icon_tier() {
        assert_eq!(klassy_rect(tier("IconTiny").as_ref()), 12);
        assert_eq!(klassy_rect(tier("IconMedium").as_ref()), 24);
        assert_eq!(klassy_rect(tier("IconLarge").as_ref()), 32);
        // IconSmall, and anything unrecognised, keeps the calibrated default.
        assert_eq!(klassy_rect(None), 16);
        assert_eq!(klassy_rect(tier("IconNonsense").as_ref()), 16);
    }
    #[test]
    fn small_circle_and_visible_gap_match_the_render() {
        // Konsole under this config: 24px dots at 1.6 scale = 15 logical, on a
        // 26 logical pitch = rect 16 + ButtonSpacingLeft 10.
        let rect = klassy_rect(tier("IconSmall").as_ref());
        let dot = rect - 1;
        assert_eq!(dot, 15);
        assert_eq!(dot + 10 + (rect - dot), 26);
    }
}

fn u8_field(group: Option<&HashMap<String, String>>, key: &str, fallback: u8) -> u8 {
    group
        .and_then(|g| g.get(key))
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(fallback)
}

const DEFAULT_CLOSE: DecoButton = DecoButton {
    normal: [255, 95, 87],
    hover: [195, 63, 69],
};
const DEFAULT_MINIMIZE: DecoButton = DecoButton {
    normal: [254, 188, 46],
    hover: [218, 165, 5],
};
const DEFAULT_MAXIMIZE: DecoButton = DecoButton {
    normal: [88, 251, 63],
    hover: [38, 148, 62],
};

fn read_ini(path: &std::path::Path) -> std::collections::HashMap<String, std::collections::HashMap<String, String>> {
    std::fs::read_to_string(path)
        .map(|text| parse_ini(&text))
        .unwrap_or_default()
}

fn parse_ini(
    text: &str,
) -> std::collections::HashMap<String, std::collections::HashMap<String, String>> {
    let mut out = std::collections::HashMap::new();
    let mut section = String::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].to_string();
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            out.entry(section.clone())
                .or_insert_with(std::collections::HashMap::new)
                .insert(key.trim().to_string(), val.trim().to_string());
        }
    }
    out
}

#[derive(serde::Deserialize)]
struct KlassyButtonOverride {
    #[serde(rename = "BackgroundNormal")]
    background_normal: Option<[u8; 3]>,
    #[serde(rename = "BackgroundHover")]
    background_hover: Option<[u8; 3]>,
}

fn klassy_button(raw: Option<&String>, fallback: DecoButton) -> DecoButton {
    let Some(raw) = raw else { return fallback };
    match serde_json::from_str::<KlassyButtonOverride>(raw) {
        Ok(o) => DecoButton {
            normal: o.background_normal.unwrap_or(fallback.normal),
            hover: o.background_hover.unwrap_or(fallback.hover),
        },
        Err(_) => fallback,
    }
}

fn opacity_percent(raw: Option<&String>) -> u8 {
    raw.and_then(|v| v.parse::<u8>().ok())
        .map(|v| v.clamp(10, 100))
        .unwrap_or(100)
}

fn config_dir() -> std::path::PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return std::path::PathBuf::from(xdg);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    std::path::PathBuf::from(home).join(".config")
}

#[tauri::command]
fn kde_window_decoration() -> WindowDecoration {
    let kwin = read_ini(&config_dir().join("kwinrc"));
    let deco = kwin.get("org.kde.kdecoration2");
    let get = |key: &str| deco.and_then(|s| s.get(key));

    let klassy = read_ini(&config_dir().join("klassy").join("klassyrc"));
    let colors = klassy.get("ButtonColors");
    let color = |key: &str| colors.and_then(|s| s.get(key));
    let sizing = klassy.get("ButtonSizing");
    let spacing = klassy.get("TitleBarSpacing");
    let windeco = klassy.get("Windeco");

    // Klassy spaces the button RECTS; the circle inside each rect is 1px
    // smaller, so the visible gap is the configured spacing plus that slack.
    let rect = klassy_rect(windeco.and_then(|g| g.get("IconSize")));
    let dot = rect.saturating_sub(1);
    let spacing_left = u8_field(sizing, "ButtonSpacingLeft", 10);

    WindowDecoration {
        buttons_left: get("ButtonsOnLeft").cloned().unwrap_or_else(|| "XIA".into()),
        buttons_right: get("ButtonsOnRight").cloned().unwrap_or_default(),
        close: klassy_button(color("ButtonOverrideColorsActiveClose"), DEFAULT_CLOSE),
        minimize: klassy_button(color("ButtonOverrideColorsActiveMinimize"), DEFAULT_MINIMIZE),
        maximize: klassy_button(color("ButtonOverrideColorsActiveMaximize"), DEFAULT_MAXIMIZE),
        bg_opacity_active: opacity_percent(color("ButtonBackgroundOpacityActive")),
        bg_opacity_inactive: opacity_percent(color("ButtonBackgroundOpacityInactive")),
        dot_size: dot,
        button_gap: spacing_left + (rect - dot),
        margin_left: u8_field(spacing, "TitleBarLeftMargin", 15),
        corner_radius: u8_field(windeco, "WindowCornerRadius", 14),
    }
}

// --- Settings (Phase 2 M1) --------------------------------------------------
// JSON-text key/value in SQLite; the frontend hydrates from here and writes
// through (debounced). init_settings seeds first-run localStorage migration.

// --- Menu model (Step 3): Rust-owned; see menu.rs ---------------------------

#[tauri::command]
fn get_menu() -> Vec<menu::Menu> {
    menu::build()
}

/// Registration state of the Global Menu: 0 trying, 1 ok, 2 failed. The
/// frontend assumes the Global Menu (titlebar bar hidden) until a definitive
/// failure, so the menubar never flashes during startup.
#[tauri::command]
fn appmenu_state() -> u8 {
    wayland_appmenu::state()
}

#[tauri::command]
fn set_menu_state(app: tauri::AppHandle, state: menu::MenuState) -> Result<(), String> {
    menu::set_state(state);
    let _ = app.emit("menu-changed", menu::build());
    menu_dbus::notify_changed();
    Ok(())
}

/// One activation path for every renderer: playback ids are handled natively
/// by the engine, everything else is re-emitted to the frontend.
#[tauri::command]
async fn menu_activate(
    app: tauri::AppHandle,
    engine: tauri::State<'_, Engine>,
    id: String,
) -> Result<(), String> {
    if !menu::activate(&id, &engine.0).await {
        let _ = app.emit("menu-action", &id);
    }
    Ok(())
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> Result<HashMap<String, String>, String> {
    library::settings::all(&state.db.lock().unwrap()).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_setting(state: tauri::State<AppState>, key: String, value: String) -> Result<(), String> {
    library::settings::set(&state.db.lock().unwrap(), &key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
fn init_settings(
    state: tauri::State<AppState>,
    values: HashMap<String, String>,
) -> Result<(), String> {
    library::settings::init_missing(&state.db.lock().unwrap(), &values)
        .map_err(|e| e.to_string())
}

// --- Library dump (Phase 2 M4) ----------------------------------------------

/// Native KDE folder picker via kdialog — the GTK chooser tauri-plugin-dialog
/// spawns looks out of place on this Plasma-only, personal-use app.
/// kdialog directory picker (native KDE style — never GTK dialogs); cancel
/// or empty output → None. `start` seeds the dialog location.
async fn pick_directory(start: PathBuf, title: &str) -> Result<Option<String>, String> {
    let title = title.to_string();
    tauri::async_runtime::spawn_blocking(move || {
        let out = std::process::Command::new("kdialog")
            .args([
                "--getexistingdirectory",
                &start.to_string_lossy(),
                "--title",
                &title,
            ])
            .output()
            .map_err(|e| format!("kdialog: {e}"))?;
        if !out.status.success() || out.stdout.is_empty() {
            return Ok(None);
        }
        let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok(if path.is_empty() { None } else { Some(path) })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// A new root must not sit inside an existing one or contain it — nested
/// roots double-walk files and make "which root owns this track" ambiguous.
fn validate_new_root(roots: &[PathBuf], path: &Path) -> Result<(), String> {
    for r in roots {
        if r == path {
            return Err("that folder is already a music folder".into());
        }
        if path.starts_with(r) {
            return Err(format!(
                "that folder is inside an existing music folder ({})",
                r.display()
            ));
        }
        if r.starts_with(path) {
            return Err(format!(
                "that folder contains an existing music folder ({})",
                r.display()
            ));
        }
    }
    Ok(())
}

fn persist_roots(state: &AppState, roots: &[PathBuf]) -> Result<(), String> {
    let list: Vec<String> = roots.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    library::settings::set(
        &state.db.lock().unwrap(),
        "musicDirs",
        &serde_json::to_string(&list).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

/// Delete library rows whose path lives under `root` (component-boundary
/// safe via LIKE with an escaped prefix + trailing separator, so `/music/b`
/// never matches `/music/bc/...`). Used when a music folder is REMOVED: the
/// user explicitly stopped tracking it, so the DB rows drop — files on disk
/// are NEVER touched. The following scan's orphan cleanup reaps any now-empty
/// album/artist rows.
fn delete_tracks_under_root(conn: &rusqlite::Connection, root: &Path) -> rusqlite::Result<usize> {
    let escaped = root
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("{escaped}/%");
    conn.execute(
        "DELETE FROM tracks WHERE path LIKE ?1 ESCAPE '\\'",
        rusqlite::params![pattern],
    )
}

/// Re-arm the file watcher over the current roots (Step 7d + 7c).
fn rewatch(state: &AppState) {
    let handle = watcher::spawn(music_roots(state), || {
        WATCH_DIRTY.store(true, std::sync::atomic::Ordering::SeqCst);
    });
    *WATCH_HANDLE.lock().unwrap() = Some(handle);
}

static WATCH_HANDLE: std::sync::Mutex<Option<watcher::WatchHandle>> =
    std::sync::Mutex::new(None);

/// Current music folders (effective roots, Step 7c).
#[tauri::command]
fn get_music_folders(state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    Ok(music_roots(&state)
        .into_iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect())
}

/// Add a music folder and rescan. `path: None` opens the kdialog picker
/// (starting at the most recent root). Returns the updated folder list, or
/// None when the user cancelled the picker.
#[tauri::command]
async fn add_music_folder(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    path: Option<String>,
) -> Result<Option<Vec<String>>, String> {
    let picked = match path {
        Some(p) => Some(p),
        None => pick_directory(music_root(&state), "Add Music Folder").await?,
    };
    let Some(picked) = picked else {
        return Ok(None); // picker cancelled — no changes, no rescan
    };
    let new_root = PathBuf::from(&picked);
    if !new_root.is_dir() {
        return Err("that folder does not exist".into());
    }
    {
        let roots = music_roots(&state);
        validate_new_root(&roots, &new_root)?;
        let mut next = roots;
        next.push(new_root);
        persist_roots(&state, &next)?;
    }
    rewatch(&state);
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let roots = scan_roots(&state, &cache_dir, None);
    run_library_scan(app, state.db_path.clone(), cache_dir, roots, false).await?;
    Ok(Some(
        music_roots(&state).into_iter().map(|p| p.to_string_lossy().into_owned()).collect(),
    ))
}

/// Remove a music folder and rescan — the deletion sweep drops tracks whose
/// path no longer sits under ANY root. FILES ON DISK ARE NEVER TOUCHED.
/// Removing the last folder empties the library (EmptyState returns).
#[tauri::command]
async fn remove_music_folder(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    path: String,
) -> Result<Vec<String>, String> {
    // Capture the removed root, then drop it from the persisted list.
    let removed = {
        let roots = music_roots(&state);
        let target = PathBuf::from(&path);
        let Some(pos) = roots.iter().position(|r| r == &target) else {
            return Err("that folder is not a music folder".into());
        };
        let mut next = roots;
        next.remove(pos);
        persist_roots(&state, &next)?;
        target
    };
    // Step 7c contract: removing a root DROPS its tracks from the library
    // (files on disk are NEVER touched). The scan's orphan cleanup then reaps
    // any now-empty album/artist rows; tracks under other roots are untouched.
    {
        let conn = state.db.lock().unwrap();
        delete_tracks_under_root(&conn, &removed).map_err(|e| e.to_string())?;
    }
    rewatch(&state);
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let roots = scan_roots(&state, &cache_dir, None);
    run_library_scan(app, state.db_path.clone(), cache_dir, roots, false).await?;
    Ok(music_roots(&state).into_iter().map(|p| p.to_string_lossy().into_owned()).collect())
}

// --- Import staging (PLAN.md Step 2a) ----------------------------------------
// Two-step flow: import_music COPIES into the staging area (playable right
// away, outside the library dir); save_imports copies staged files into
// <musicDir> and deletes the staged copies; discard_imports just deletes.

/// Native KDE multi-file picker (kdialog --getopenfilename --multiple).
/// Returns None on cancel. kdialog separates multiple hits with " \n"? —
/// it actually emits one path per line when --multiple is used with a
/// trailing newline; we split on newlines and tolerate spaces.
#[tauri::command]
async fn choose_import_files(state: tauri::State<'_, AppState>) -> Result<Option<Vec<String>>, String> {
    let start = music_root(&state);
    tauri::async_runtime::spawn_blocking(move || {
        let out = std::process::Command::new("kdialog")
            .args([
                "--getopenfilename",
                &start.to_string_lossy(),
                // KDE filter syntax "globs|Label"; MIME globs (audio/*) miss
                // files whose mime info is missing, extensions never do.
                "*.mp3 *.flac *.m4a *.aif *.aiff *.ogg *.opus *.wav|Audio Files\n*|All Files",
                "--multiple",
                "--title",
                "Add Music Files",
                "--separate-output",
            ])
            .output()
            .map_err(|e| format!("kdialog: {e}"))?;
        if !out.status.success() || out.stdout.is_empty() {
            return Ok(None);
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let files: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        Ok(if files.is_empty() { None } else { Some(files) })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Native KDE folder picker for IMPORT staging (unlike add_music_folder
/// this does NOT touch the music folders setting). Returns None on cancel.
#[tauri::command]
async fn choose_import_folder(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let start = music_root(&state);
    pick_directory(start, "Choose Import Folder").await
}

/// Native KDE file picker for re-linking a missing track (Step 2a follow-up).
#[tauri::command]
async fn choose_relink_file(start: String) -> Result<Option<String>, String> {
    let start = if start.is_empty() || !Path::new(&start).exists() {
        dirs_home().join("Music")
    } else {
        PathBuf::from(start)
    };
    tauri::async_runtime::spawn_blocking(move || {
        let out = std::process::Command::new("kdialog")
            .args([
                "--getopenfilename",
                &start.to_string_lossy(),
                "*.mp3 *.flac *.m4a *.aif *.aiff *.ogg *.opus *.wav|Audio Files",
                "--title",
                "Locate the missing track",
            ])
            .output()
            .map_err(|e| format!("kdialog: {e}"))?;
        if !out.status.success() || out.stdout.is_empty() {
            return Ok(None);
        }
        let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok(if path.is_empty() { None } else { Some(path) })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Point a missing track's row at a file the user located. Files outside the
/// library dir are COPIED into `<musicDir>/<Artist>/<Album>/` first (the
/// scan only walks the library + staging roots, so an external path would
/// be swept as an orphan on the next rescan). mtime is zeroed so the next
/// scan re-parses the file's tags through the real grouping path.
#[tauri::command]
async fn relink_track(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    track_id: String,
    new_path: String,
) -> Result<(), String> {
    let new_path = PathBuf::from(new_path);
    if !new_path.is_file() {
        return Err("that file does not exist".into());
    }
    if !library::scan::is_supported(&new_path) {
        return Err("that file type is not supported".into());
    }
    let music_dir = music_root(&state);
    let import_root = app
        .path()
        .app_cache_dir()
        .map(|d| library::import::import_dir(&d))
        .map_err(|e| e.to_string())?;

    let final_path = if new_path.starts_with(&music_dir) || new_path.starts_with(&import_root) {
        new_path.clone()
    } else {
        let (artist, album): (String, String) = {
            let conn = state.db.lock().unwrap();
            conn.query_row(
                "SELECT ar.name, al.title FROM tracks t
                 JOIN albums al ON al.id = t.album_id
                 JOIN artists ar ON ar.id = al.artist_id
                 WHERE t.id = ?1",
                [&track_id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .map_err(|_| "unknown track".to_string())?
        };
        let name = new_path
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();
        let dest = music_dir
            .join(library::import::sanitize_path(&artist))
            .join(library::import::sanitize_path(&album))
            .join(name);
        match library::import::resolve_collision(&new_path, &dest)? {
            Some(d) => {
                if let Some(p) = d.parent() {
                    std::fs::create_dir_all(p).map_err(|e| format!("mkdir {p:?}: {e}"))?;
                }
                std::fs::copy(&new_path, &d).map_err(|e| format!("copy: {e}"))?;
                d
            }
            None => return Err("the library already has an identical file".into()),
        }
    };

    let conn = state.db.lock().unwrap();
    let changed = conn
        .execute(
            "UPDATE tracks SET path = ?1, mtime_ns = 0 WHERE id = ?2",
            rusqlite::params![final_path.to_string_lossy(), track_id],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                "that file is already linked to another track".to_string()
            } else {
                e.to_string()
            }
        })?;
    if changed == 0 {
        return Err("unknown track".into());
    }
    Ok(())
}

/// Remove a missing track's row (and any orphaned album/artist) from the
/// library. The user's explicit choice — vanished files are otherwise kept
/// and flagged missing so they can be relinked.
#[tauri::command]
async fn delete_track(state: tauri::State<'_, AppState>, track_id: String) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    conn.execute("DELETE FROM tracks WHERE id = ?1", [&track_id])
        .map_err(|e| e.to_string())?;
    conn.execute_batch(
        "DELETE FROM albums WHERE id NOT IN (SELECT DISTINCT album_id FROM tracks);
         DELETE FROM artists WHERE id NOT IN (SELECT DISTINCT artist_id FROM albums)
           AND id != 'ar-various';",
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Remove EVERY missing track (optionally scoped to one album) in one go.
/// Returns how many rows were removed.
#[tauri::command]
async fn delete_missing_tracks(
    state: tauri::State<'_, AppState>,
    album_id: Option<String>,
) -> Result<usize, String> {
    let conn = state.db.lock().unwrap();
    let ids: Vec<String> = match &album_id {
        Some(id) => {
            let mut stmt = conn
                .prepare("SELECT id, path FROM tracks WHERE album_id = ?1")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([id], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                })
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok())
                .filter(|(_, p)| !Path::new(p).exists())
                .map(|(id, _)| id)
                .collect()
        }
        None => {
            let mut stmt = conn
                .prepare("SELECT id, path FROM tracks")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok())
                .filter(|(_, p)| !Path::new(p).exists())
                .map(|(id, _)| id)
                .collect()
        }
    };
    let n = ids.len();
    for id in &ids {
        conn.execute("DELETE FROM tracks WHERE id = ?1", [id])
            .map_err(|e| e.to_string())?;
    }
    if n > 0 {
        conn.execute_batch(
            "DELETE FROM albums WHERE id NOT IN (SELECT DISTINCT album_id FROM tracks);
             DELETE FROM artists WHERE id NOT IN (SELECT DISTINCT artist_id FROM albums)
               AND id != 'ar-various';",
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(n)
}

#[tauri::command]
async fn import_music(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    paths: Vec<String>,
) -> Result<ScanSummary, String> {
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?;
    let paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let emitter = app.clone();
    let cache = cache_dir.clone();
    let db_path = state.db_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Skip anything we already hold (library rows + staged files) so
        // "add file, then add its folder" can never duplicate tracks.
        let mut known = library::import::KnownFiles::from_db(
            &library::db::open(&db_path).map_err(|e| e.to_string())?,
        );
        for (name, size) in library::import::KnownFiles::from_dir(&library::import::import_dir(&cache)).0 {
            known.0.insert((name, size));
        }
        library::import::import_paths(&cache, &paths, &known, |done, total| {
            let _ = emitter.emit(
                "scan-progress",
                serde_json::json!({ "phase": "import", "done": done, "total": total }),
            );
        })
    })
    .await
    .map_err(|e| e.to_string())??;

    let roots = scan_roots(&state, &cache_dir, None);
    run_library_scan(app, state.db_path.clone(), cache_dir, roots, false).await
}

/// Save staged music into the library dir. `album_id` scopes to one album,
/// `track_id` to one track; neither saves EVERYTHING staged (the global
/// "save imported music" action).
#[tauri::command]
async fn save_imports(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    album_id: Option<String>,
    track_id: Option<String>,
) -> Result<library::import::SaveReport, String> {
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?;
    let db_path = state.db_path.clone();
    let cache = cache_dir.clone();
    let staged = tauri::async_runtime::spawn_blocking(move || {
        let conn = library::db::open(&db_path).map_err(|e| e.to_string())?;
        library::import::staged_tracks(&conn, &cache, album_id.as_deref(), track_id.as_deref())
    })
    .await
    .map_err(|e| e.to_string())??;
    if staged.is_empty() {
        return Ok(Default::default());
    }

    let music_dir = music_root(&state);
    // Where a staged album may land. The staging dir is deliberately NOT one of
    // these: merging into a copy that is about to be deleted is how an album ends
    // up pointing at nothing.
    let adopt_roots = music_roots(&state);
    let emitter = app.clone();
    let cache = cache_dir.clone();
    let db_path = state.db_path.clone();
    let staged_clone = staged.clone();
    let saved = tauri::async_runtime::spawn_blocking(move || {
        let conn = library::db::open(&db_path).map_err(|e| e.to_string())?;
        library::import::save_to_library(
            &conn,
            &cache,
            &adopt_roots,
            &music_dir,
            &staged_clone,
            |done, total| {
                let _ = emitter.emit(
                    "scan-progress",
                    serde_json::json!({ "phase": "import", "done": done, "total": total }),
                );
            },
        )
    })
    .await
    .map_err(|e| e.to_string())??;

    let roots = scan_roots(&state, &cache_dir, None);
    run_library_scan(app, state.db_path.clone(), cache_dir, roots, false).await?;
    Ok(saved)
}

/// Discard staged music: delete the staged files (library untouched).
#[tauri::command]
async fn discard_imports(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    album_id: Option<String>,
    track_id: Option<String>,
) -> Result<usize, String> {
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?;
    let db_path = state.db_path.clone();
    let cache = cache_dir.clone();
    let staged = tauri::async_runtime::spawn_blocking(move || {
        let conn = library::db::open(&db_path).map_err(|e| e.to_string())?;
        library::import::staged_tracks(&conn, &cache, album_id.as_deref(), track_id.as_deref())
    })
    .await
    .map_err(|e| e.to_string())??;
    if staged.is_empty() {
        return Ok(0);
    }
    let n = staged.len();
    let db_path = state.db_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = library::db::open(&db_path).map_err(|e| e.to_string())?;
        library::import::discard(&conn, &staged)
    })
    .await
    .map_err(|e| e.to_string())??;

    let roots = scan_roots(&state, &cache_dir, None);
    run_library_scan(app, state.db_path.clone(), cache_dir, roots, false).await?;
    Ok(n)
}

/// The staged albums and the folder each would move to, for the manage-imports
/// modal: the destination is what the decision is about, so it has to be on
/// screen before Apply, not in a dialog afterwards.
#[tauri::command]
async fn staged_import_plan(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<library::import::StagedAlbum>, String> {
    let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let roots = music_roots(&state);
    let music_dir = music_root(&state);
    let staging = library::import::import_dir(&cache_dir);
    let db = state.db_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = library::db::open(&db).map_err(|e| e.to_string())?;
        library::import::staged_plan(&conn, &roots, &music_dir, &staging)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(serde::Serialize)]
struct LibraryDump {
    artists: Vec<serde_json::Value>,
    albums: Vec<serde_json::Value>,
    tracks: Vec<serde_json::Value>,
}

/// Full library dump for frontend hydration. Personal-library scale makes a
/// single payload the right call (see PHASE2.md §7); revisit only if it
/// exceeds ~20 MB.
#[tauri::command]
fn get_library(app: tauri::AppHandle, state: tauri::State<AppState>) -> Result<LibraryDump, String> {
    let conn = state.db.lock().unwrap();
    let import_root = app
        .path()
        .app_cache_dir()
        .map(|d| library::import::import_dir(&d))
        .unwrap_or_default();

    let mut artists = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, name, sort_name FROM artists ORDER BY sort_name")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(serde_json::json!({
                    "id": r.get::<_, String>(0)?,
                    "name": r.get::<_, String>(1)?,
                    "sortName": r.get::<_, String>(2)?,
                }))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            artists.push(row.map_err(|e| e.to_string())?);
        }
    }
    let mut albums = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, artist_id, title, year, cover, color_c1, color_c2
                 FROM albums ORDER BY title",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                Ok(serde_json::json!({
                    "id": r.get::<_, String>(0)?,
                    "artistId": r.get::<_, String>(1)?,
                    "title": r.get::<_, String>(2)?,
                    "year": r.get::<_, Option<i64>>(3)?,
                    "cover": r.get::<_, Option<String>>(4)?,
                    "colorC1": r.get::<_, Option<String>>(5)?,
                    "colorC2": r.get::<_, Option<String>>(6)?,
                }))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            albums.push(row.map_err(|e| e.to_string())?);
        }
    }
    let mut tracks = Vec::new();
    let mut staged_albums: std::collections::HashSet<String> = Default::default();
    {
        let mut stmt = conn
            .prepare(
                "SELECT id, album_id, disc, track, title, duration_sec, path
                  FROM tracks
                  ORDER BY album_id, disc, (track IS NOT NULL), track, title",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                let id: String = r.get(0)?;
                let album_id: String = r.get(1)?;
                let path: String = r.get(6)?;
                let staged = !import_root.as_os_str().is_empty()
                    && Path::new(&path).starts_with(&import_root);
                if staged {
                    staged_albums.insert(album_id.clone());
                }
                let missing = !Path::new(&path).exists();
                Ok(serde_json::json!({
                    "id": id,
                    "albumId": album_id,
                    "disc": r.get::<_, i64>(2)?,
                    "track": r.get::<_, Option<i64>>(3)?,
                    "title": r.get::<_, String>(4)?,
                    "durationSec": r.get::<_, f64>(5)?,
                    "staged": staged,
                    "missing": missing,
                }))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            tracks.push(row.map_err(|e| e.to_string())?);
        }
    }
    for album in &mut albums {
        let id = album["id"].as_str().unwrap_or_default().to_string();
        album["staged"] = serde_json::json!(staged_albums.contains(&id));
    }
    Ok(LibraryDump {
        artists,
        albums,
        tracks,
    })
}

// --- Tag editor (PLAN.md Step 1) --------------------------------------------
// Reads hit FILES (not the DB) so the editor shows what a rescan would see;
// saves rewrite tags and let edited mtimes drive the incremental scan. Each
// command opens its own connection so blocking file IO never holds AppState.

#[tauri::command]
async fn get_track_tags(
    state: tauri::State<'_, AppState>,
    track_id: String,
) -> Result<library::tags::TrackTags, String> {
    let db_path = state.db_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = library::db::open(&db_path).map_err(|e| e.to_string())?;
        library::tags::get_track_tags(&conn, &track_id)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_album_tags(
    state: tauri::State<'_, AppState>,
    album_id: String,
) -> Result<library::tags::AlbumTags, String> {
    let db_path = state.db_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = library::db::open(&db_path).map_err(|e| e.to_string())?;
        library::tags::get_album_tags(&conn, &album_id)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn save_track_tags(
    state: tauri::State<'_, AppState>,
    track_id: String,
    tags: library::tags::TrackTags,
) -> Result<(), String> {
    let db_path = state.db_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = library::db::open(&db_path).map_err(|e| e.to_string())?;
        library::tags::save_track_tags(&conn, &track_id, &tags)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn save_album_tags(
    state: tauri::State<'_, AppState>,
    album_id: String,
    shared: library::tags::TrackTags,
) -> Result<usize, String> {
    let db_path = state.db_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let conn = library::db::open(&db_path).map_err(|e| e.to_string())?;
        library::tags::save_album_tags(&conn, &album_id, &shared)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .register_uri_scheme_protocol("thumb", |ctx, request| {
            // thumb://<album_id>/<size>.webp — served from the app cache dir
            // with a size fallback chain (512 → 256 → 96).
            let response = |status: u16, body: Vec<u8>| tauri::http::Response::builder()
                .status(status)
                .header("Content-Type", "image/webp")
                .header("Cache-Control", "max-age=31536000, immutable")
                .body(body)
                .unwrap();
            let uri = request.uri().to_string();
            let path = uri
                .strip_prefix("thumb://")
                .map(|p| p.trim_start_matches("localhost"))
                .unwrap_or("");
            let mut segments = path.split('/').filter(|s| !s.is_empty());
            let (Some(album_id), Some(file)) = (segments.next(), segments.next()) else {
                return response(400, Vec::new());
            };
            if album_id.contains("..") || file.contains("..") {
                return response(400, Vec::new());
            }
            let Ok(cache_dir) = ctx.app_handle().path().app_cache_dir() else {
                return response(500, Vec::new());
            };
            let size = file.trim_end_matches(".webp");
            let chain: &[u32] = &[512, 256, 96];
            let mut sizes: Vec<u32> = vec![size.parse().unwrap_or(512)];
            sizes.extend(chain.iter().copied());
            for candidate in sizes {
                let path =
                    library::artwork::thumb_path(&cache_dir, album_id, candidate);
                if let Ok(bytes) = std::fs::read(&path) {
                    return response(200, bytes);
                }
            }
            response(404, Vec::new())
        })
        .setup(|app| {
            // WebKitGTK sometimes maps its view at a stale size inside the
            // correctly-sized GTK window (playbar mid-screen, dead glass
            // below); activation doesn't heal it. A 1px shrink+restore forces
            // a reconfigure that does. Harmless if sizing already succeeded.
            let win = app.get_webview_window("main").expect("main window");
            std::thread::spawn(move || {
                for delay_ms in [600u64, 2500] {
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                    if let Ok(size) = win.inner_size() {
                        let _ = win
                            .set_size(tauri::PhysicalSize::new(
                                size.width,
                                size.height.saturating_sub(1),
                            ));
                        std::thread::sleep(std::time::Duration::from_millis(80));
                        let _ = win.set_size(size);
                    }
                }
            });
            // Phase 2 M1: open (and migrate) the library DB for the shared
            // app state before any command can run.
            let db_dir = app
                .path()
                .app_data_dir()
                .expect("app data dir");
            std::fs::create_dir_all(&db_dir).expect("create data dir");
            let conn = library::db::open(&db_dir.join("songstress.db"))
                .expect("open library database");
            // Persisted volume (JSON number in settings) drives the mpv spawn
            // flag — the first track must not blare at the default 80.
            let volume: f64 = library::settings::all(&conn)
                .ok()
                .and_then(|m| m.get("volume").cloned())
                .and_then(|v| serde_json::from_str::<f64>(&v).ok())
                .unwrap_or(80.0);
            app.manage(AppState {
                db_path: db_dir.join("songstress.db"),
                db: Mutex::new(conn),
            });

            // Phase 3: spawn the mpv engine once; commands talk to it through
            // the managed handle. Events reach the frontend via APP handle.
            mpv::set_app_handle(app.handle().clone());
            let play_state = std::sync::Arc::new(Mutex::new(mpv::PlayState::default()));
            let engine = match tauri::async_runtime::block_on(mpv::Mpv::spawn(
                play_state.clone(),
                volume,
            )) {
                Ok(e) => e,
                Err(e) => {
                    // No mpv binary / no socket: playback commands fail with a
                    // clean error instead of taking the app down.
                    eprintln!("[mpv] engine unavailable, playback disabled: {e}");
                    mpv::Mpv::detached(play_state)
                }
            };
            app.manage(Engine(engine.clone()));

            // Restore the persisted shuffle/repeat stages (Step 5a): modes
            // must survive launches. Best-effort — a detached engine or a
            // missing row just leaves the defaults.
            {
                let app_state = app.state::<AppState>();
                let stages: (String, String) = {
                    let conn = app_state.db.lock().unwrap();
                    let all = library::settings::all(&conn).ok().unwrap_or_default();
                    let get = |k: &str| {
                        all.get(k)
                            .and_then(|v| serde_json::from_str::<String>(v).ok())
                            .unwrap_or_default()
                    };
                    (get("shuffleStage"), get("repeatStage"))
                };
                let sh = mpv::ShuffleStage::parse(&stages.0);
                let rp = mpv::RepeatStage::parse(&stages.1);
                engine.state.lock().unwrap().shuffle = sh;
                engine.state.lock().unwrap().repeat = rp;
                if rp == mpv::RepeatStage::Track {
                    let _ = tauri::async_runtime::block_on(engine.set_repeat(rp));
                }
            }

            // Restore the persisted equalizer (Step 6): applied straight to
            // mpv's `af` before the first track loads. Best-effort like the
            // stages — a bad/absent row just leaves it off.
            {
                let app_state = app.state::<AppState>();
                let saved = {
                    let conn = app_state.db.lock().unwrap();
                    library::settings::all(&conn)
                        .ok()
                        .and_then(|m| m.get("equalizer").cloned())
                        .and_then(|v| serde_json::from_str::<eq::Eq>(&v).ok())
                };
                if let Some(saved) = saved {
                    let _ = tauri::async_runtime::block_on(engine.set_af(eq::af_chain(&saved)));
                }
            }

            // Step 7c one-time migration: write `musicDirs` (authoritative
            // JSON array) from the legacy single `musicDir` — the effective
            // root list, so the fallback (~/Music) becomes explicit and
            // removable. Best-effort; roots_from_settings handles both shapes.
            {
                let st = app.state::<AppState>();
                let needs_migration = {
                    let conn = st.db.lock().unwrap();
                    library::settings::all(&conn)
                        .ok()
                        .map(|m| !m.contains_key("musicDirs"))
                        .unwrap_or(false)
                };
                if needs_migration {
                    let current = music_roots(&st);
                    let _ = persist_roots(&st, &current);
                }
            }

            // Step 7d: live library watching — inotify over ALL music roots
            // (Step 7c: re-armed by add/remove_music_folder), debounced 2s;
            // a polling async loop turns the dirty flag into incremental
            // rescans (waiting out SCAN_RUNNING and the minimum interval).
            {
                use std::sync::atomic::Ordering;
                rewatch(&app.state::<AppState>());
                let app_h = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let mut last: Option<std::time::Instant> = None;
                    loop {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        if !WATCH_DIRTY.load(Ordering::SeqCst) {
                            continue;
                        }
                        if SCAN_RUNNING.load(Ordering::SeqCst) {
                            continue; // dirty stays set → rescan after it settles
                        }
                        if let Some(t) = last {
                            if t.elapsed() < WATCH_MIN_INTERVAL {
                                continue; // dirty stays set → scan when due
                            }
                        }
                        WATCH_DIRTY.store(false, Ordering::SeqCst);
                        last = Some(std::time::Instant::now());
                        let cache_dir = app_h.path().app_cache_dir().unwrap_or_default();
                        let (roots, db_path) = {
                            let st = app_h.state::<AppState>();
                            (scan_roots(&st, &cache_dir, None), st.db_path.clone())
                        };
                        eprintln!("[watch] library changed on disk — rescanning");
                        let _ =
                            run_library_scan(app_h.clone(), db_path, cache_dir, roots, false).await;
                    }
                });
            }

            // Phase 4: MPRIS on the session bus. Swallows its own errors —
            // a missing/busy bus name must not take playback down.
            // thumb_path() appends "thumbs" itself — pass the cache root.
            let cache_dir = app.path().app_cache_dir().expect("app cache dir");
            mpris::serve(
                engine.clone(),
                db_dir.join("songstress.db"),
                cache_dir,
                app.handle().clone(),
            );

            // Step 3: Plasma Global Menu — dbusmenu object + KWin Wayland
            // registration (best-effort; titlebar menus always work).
            menu_dbus::serve(engine.clone(), app.handle().clone());

            // Phase 2 groundwork note: color extraction will move into the
            // library scan pipeline (computed once per cover during scan,
            // stored with the album row). No startup warm-up here on purpose —
            // album_colors' path-keyed COLOR_CACHE already dedupes per session.
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            album_colors,
            fix_viewport,
            kde_window_decoration,
            staged_import_plan,
            get_settings,
            set_menu_state,
            get_menu,
            appmenu_state,
            menu_activate,
            set_setting,
            init_settings,
            scan_library,
            get_library,
            get_music_folders,
            add_music_folder,
            remove_music_folder,
            choose_import_files,
            choose_import_folder,
            import_music,
            save_imports,
            discard_imports,
            relink_track,
            choose_relink_file,
            delete_track,
            delete_missing_tracks,
            play_album,
            playback_pause,
            playback_toggle,
            playback_jump,
            playback_set_shuffle,
            playback_set_repeat,
            playback_seek,
            playback_volume,
            playback_stop,
            playback_queue,
            playback_queue_remove,
            playback_queue_jump,
            playback_queue_clear,
            playback_eq,
            get_track_tags,
            get_album_tags,
            save_track_tags,
            save_album_tags
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // mpv is a detached child — without this it keeps playing after
            // the window closes.
            if let tauri::RunEvent::Exit = event {
                mpv::kill_engine();
            }
        });
}

#[cfg(test)]
mod tests {
    use super::{colors, klassy_button, opacity_percent, parse_ini};
    use super::{DEFAULT_CLOSE, DEFAULT_MAXIMIZE, DEFAULT_MINIMIZE};

    #[test]
    fn extracts_from_every_cover_without_overflow() {
        let dir = std::path::Path::new("../public/covers");
        let mut count = 0;
        for entry in std::fs::read_dir(dir).expect("covers dir") {
            let path = entry.expect("entry").path();
            if !matches!(path.extension().and_then(|e| e.to_str()), Some("jpg") | Some("png")) {
                continue;
            }
            let out = colors::extract(&path)
                .unwrap_or_else(|e| panic!("extraction failed for {:?}: {e}", path));
            assert_eq!(out.len(), 6);
            count += 1;
        }
        assert!(count >= 10, "expected the full cover set, got {count}");
    }

    #[test]
    fn anison_accent_is_the_blue_hair() {
        let out = colors::extract(std::path::Path::new(
            "../public/covers/various-artists-anison-no-kokoro.jpg",
        ))
        .expect("extraction");
        let c2 = [out[3], out[4], out[5]];
        assert!(
            c2[2] > c2[0] + 20,
            "accent should lean blue, got {c2:?}"
        );
    }

    #[test]
    fn ini_sections_and_comments_parse() {
        let ini = parse_ini(
            "[org.kde.kdecoration2]\nButtonsOnLeft=XIA\n# comment\n; also comment\n\n[Other]\nKey=1\n",
        );
        let deco = &ini["org.kde.kdecoration2"];
        assert_eq!(deco["ButtonsOnLeft"], "XIA");
        assert_eq!(ini["Other"]["Key"], "1");
    }

    #[test]
    fn klassy_overrides_apply_with_fallback() {
        let good = r#"{"BackgroundHover":[195,63,69],"BackgroundNormal":[253,82,86]}"#;
        let b = klassy_button(Some(&good.to_string()), DEFAULT_CLOSE);
        assert_eq!(b.normal, [253, 82, 86]);
        assert_eq!(b.hover, [195, 63, 69]);

        // Partial override keeps the untouched channel from the fallback.
        let partial = r#"{"BackgroundNormal":[10,20,30]}"#;
        let b = klassy_button(Some(&partial.to_string()), DEFAULT_MINIMIZE);
        assert_eq!(b.normal, [10, 20, 30]);
        assert_eq!(b.hover, DEFAULT_MINIMIZE.hover);

        // Garbage JSON and absent keys fall back wholesale.
        assert_eq!(
            klassy_button(Some(&"not json".to_string()), DEFAULT_CLOSE),
            DEFAULT_CLOSE
        );
        assert_eq!(klassy_button(None, DEFAULT_MAXIMIZE), DEFAULT_MAXIMIZE);
    }

    #[test]
    fn opacity_clamps_to_sane_range() {
        assert_eq!(opacity_percent(Some(&"85".to_string())), 85);
        assert_eq!(opacity_percent(Some(&"5".to_string())), 10);
        assert_eq!(opacity_percent(Some(&"300".to_string())), 100);
        assert_eq!(opacity_percent(Some(&"abc".to_string())), 100);
        assert_eq!(opacity_percent(None), 100);
    }

    // --- Step 7c: multi-library roots ---------------------------------------

    #[test]
    fn roots_from_settings_migration_and_multi_root() {
        let mut legacy = std::collections::HashMap::new();
        legacy.insert(
            "musicDir".to_string(),
            serde_json::to_string(&Some("/home/u/Music".to_string())).unwrap(),
        );
        // Legacy single musicDir is wrapped to a one-element list.
        assert_eq!(
            super::roots_from_settings(&legacy),
            vec![std::path::PathBuf::from("/home/u/Music")]
        );

        // Legacy JSON null / "" = unset → ~/Music fallback (single primary).
        let mut null_legacy = std::collections::HashMap::new();
        null_legacy.insert("musicDir".to_string(), "null".to_string());
        let r = super::roots_from_settings(&null_legacy);
        assert_eq!(r.len(), 1);
        assert!(r[0].ends_with("Music"), "{:?}", r[0]);

        let mut empty_legacy = std::collections::HashMap::new();
        empty_legacy.insert("musicDir".to_string(), "\"\"".to_string());
        let r = super::roots_from_settings(&empty_legacy);
        assert_eq!(r.len(), 1);
        assert!(r[0].ends_with("Music"));

        // musicDirs (JSON array) is authoritative once present.
        let mut multi = std::collections::HashMap::new();
        multi.insert(
            "musicDirs".to_string(),
            serde_json::to_string(&vec!["/music/a".to_string(), "/music/b".to_string()]).unwrap(),
        );
        assert_eq!(
            super::roots_from_settings(&multi),
            vec![
                std::path::PathBuf::from("/music/a"),
                std::path::PathBuf::from("/music/b"),
            ]
        );

        // Empty musicDirs → empty library (no fallback).
        let mut empty_multi = std::collections::HashMap::new();
        empty_multi.insert("musicDirs".to_string(), "[]".to_string());
        assert!(super::roots_from_settings(&empty_multi).is_empty());

        // musicDirs wins over a stale musicDir even when it's empty.
        let mut both = std::collections::HashMap::new();
        both.insert("musicDirs".to_string(), "[]".to_string());
        both.insert("musicDir".to_string(), "/stale".to_string());
        assert!(super::roots_from_settings(&both).is_empty());

        // Garbage musicDirs → empty (must NOT silently resurrect the legacy root).
        let mut bad = std::collections::HashMap::new();
        bad.insert("musicDirs".to_string(), "not json".to_string());
        assert!(super::roots_from_settings(&bad).is_empty());

        // Missing both keys → ~/Music fallback.
        assert_eq!(super::roots_from_settings(&std::collections::HashMap::new()).len(), 1);
    }

    #[test]
    fn validate_new_root_rejects_overlap() {
        use std::path::Path;
        let roots = vec![std::path::PathBuf::from("/music/a")];
        // duplicate
        assert!(super::validate_new_root(&roots, Path::new("/music/a")).is_err());
        // candidate nested inside an existing root
        assert!(super::validate_new_root(&roots, Path::new("/music/a/sub")).is_err());
        // existing root nested inside the candidate
        assert!(super::validate_new_root(&roots, Path::new("/music")).is_err());
        // disjoint candidates pass
        assert!(super::validate_new_root(&roots, Path::new("/music/b")).is_ok());
        assert!(super::validate_new_root(&roots, Path::new("/other/c")).is_ok());
        // empty root list → anything is ok
        let empty: Vec<std::path::PathBuf> = vec![];
        assert!(super::validate_new_root(&empty, Path::new("/music/anything")).is_ok());
    }

    #[test]
    fn persist_roots_round_trips_through_settings() {
        let dir = std::env::temp_dir().join("songstress-m1-tests");
        std::fs::create_dir_all(&dir).expect("dir");
        let dbp = dir.join(format!("roots-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&dbp);
        let conn = crate::library::db::open(&dbp).expect("open");
        let state = super::AppState {
            db_path: dbp.clone(),
            db: std::sync::Mutex::new(conn),
        };
        let roots = vec![
            std::path::PathBuf::from("/music/a"),
            std::path::PathBuf::from("/other/b"),
        ];
        super::persist_roots(&state, &roots).expect("persist");
        // Round-trips: roots_from_settings reads musicDirs back.
        let got = super::roots_from_settings(
            &crate::library::settings::all(&state.db.lock().unwrap()).unwrap(),
        );
        assert_eq!(got, roots);
        let _ = std::fs::remove_file(&dbp);
    }

    #[test]
    fn root_removal_drops_only_tracks_under_it() {
        let dir = std::env::temp_dir().join("songstress-m1-tests");
        std::fs::create_dir_all(&dir).expect("dir");
        let dbp = dir.join(format!("rm-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&dbp);
        let conn = crate::library::db::open(&dbp).expect("open");
        conn.execute_batch(
            "INSERT INTO artists VALUES ('ar-x','X','x');
             INSERT INTO albums VALUES ('al-1','ar-x','A',2020,NULL,NULL,NULL);
             INSERT INTO albums VALUES ('al-2','ar-x','B',2021,NULL,NULL,NULL);
             INSERT INTO tracks VALUES ('tr-1','al-1',1,1,'t1',10.0,'/music/a/track.flac',1,123);
             INSERT INTO tracks VALUES ('tr-2','al-2',1,1,'t2',10.0,'/music/b/track.flac',1,123);
             INSERT INTO tracks VALUES ('tr-3','al-2',1,1,'t3',10.0,'/music/bc/other.flac',1,456);",
        )
        .expect("seed");
        let n = super::delete_tracks_under_root(&conn, std::path::Path::new("/music/b"))
            .expect("delete");
        assert_eq!(n, 1, "only /music/b/* drops, not /music/bc/* or /music/a/*");
        let left: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |r| r.get(0))
            .expect("count");
        assert_eq!(left, 2, "/music/a and /music/bc tracks survive");
        let _ = std::fs::remove_file(&dbp);
    }
}

