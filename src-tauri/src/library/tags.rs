//! Tag editor backend (PLAN.md Step 1): read/write file tags via lofty.
//!
//! Reads always hit the FILES (not the DB) — the editor shows what a rescan
//! would see. Saves rewrite tags and let the edited mtimes drive the existing
//! incremental scan through the real grouping path; there is deliberately NO
//! parallel DB-update code here.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::picture::{MimeType, Picture, PictureType};
use lofty::tag::{Accessor, ItemKey, Tag};
use rusqlite::Connection;

/// Full per-file tag surface of the editor (MusicBee Tags-tab field set;
/// conductor/lyrics/ratings excluded by decision, artwork editing deferred).
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackTags {
    #[serde(default)] pub title: String,
    #[serde(default)] pub artist: String,
    #[serde(default)] pub album_artist: String,
    #[serde(default)] pub album: String,
    #[serde(default)] pub year: Option<i64>,
    #[serde(default)] pub track_no: Option<u32>,
    #[serde(default)] pub track_total: Option<u32>,
    #[serde(default)] pub disc_no: Option<u32>,
    #[serde(default)] pub disc_total: Option<u32>,
    #[serde(default)] pub genre: String,
    #[serde(default)] pub composer: String,
    #[serde(default)] pub label: String,
    #[serde(default)] pub comment: String,
    #[serde(default)] pub grouping: String,
}

/// One distinct non-empty value of a disputed album field, with how many of
/// the album's files carry it. The `•` in the UI expands to these — the
/// disagreement is shown, not just announced.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueCount {
    pub value: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldConflict {
    /// camelCase field name, matching the editor's Editable keys.
    pub field: String,
    /// Distinct non-empty values, most-frequent first (ties keep file order).
    /// EMPTY when every file is silent on this field; length ≥2 is what the
    /// UI treats as a dispute (the `*_disputed` booleans say the same).
    pub values: Vec<ValueCount>,
    /// Files carrying any non-empty value — the census's remainder is
    /// "no value", which the frontend needs to diff a CLEAR precisely.
    pub present: u32,
}

/// Album-level view: consensus values ("first non-empty wins") with
/// disagreement flags and the competing values themselves. Per-track fields
/// are edited track-by-track only.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumTags {
    #[serde(default)] pub album_artist: String,
    #[serde(default)] pub album_artist_disputed: bool,
    #[serde(default)] pub album: String,
    #[serde(default)] pub album_disputed: bool,
    #[serde(default)] pub year: Option<i64>,
    #[serde(default)] pub year_disputed: bool,
    #[serde(default)] pub genre: String,
    #[serde(default)] pub genre_disputed: bool,
    #[serde(default)] pub composer: String,
    #[serde(default)] pub composer_disputed: bool,
    #[serde(default)] pub label: String,
    #[serde(default)] pub label_disputed: bool,
    #[serde(default)] pub grouping: String,
    #[serde(default)] pub grouping_disputed: bool,
    #[serde(default)] pub comment: String,
    #[serde(default)] pub comment_disputed: bool,
    #[serde(default)] pub track_total: Option<u32>,
    #[serde(default)] pub track_total_disputed: bool,
    #[serde(default)] pub disc_total: Option<u32>,
    #[serde(default)] pub disc_total_disputed: bool,
    /// The disputed fields as data (every shared field gets a census, even
    /// quiet ones — the frontend diffs against it). Supersedes the
    /// `*_disputed` booleans, which stay for compatibility.
    #[serde(default)]
    pub conflicts: Vec<FieldConflict>,
    /// Files in the album — the denominator of "writes 4 of 18 files".
    pub file_count: usize,
}

fn clean(v: Option<Cow<'_, str>>) -> String {
    v.map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_default()
}

fn text(tag: Option<&Tag>, key: &ItemKey) -> String {
    clean(tag.and_then(|t| t.get_string(key)).map(Into::into))
}

fn read_file(path: &Path) -> Result<TrackTags, String> {
    Ok(read_file_full(path)?.0)
}

/// TrackTags + the blake3 hashes of every picture the file carries (across
/// all its tags) — the fingerprint the artwork diff compares against.
fn read_file_full(path: &Path) -> Result<(TrackTags, Vec<String>), String> {
    let tagged = lofty::read_from_path(path).map_err(|e| format!("{path:?}: {e}"))?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let pics: Vec<String> = tagged
        .tags()
        .iter()
        .flat_map(|t| t.pictures())
        .map(|p| blake3::hash(p.data()).to_hex().to_string())
        .collect();
    Ok((
        TrackTags {
            title: clean(tag.and_then(|t| t.title())),
            artist: clean(tag.and_then(|t| t.artist())),
            album_artist: text(tag, &ItemKey::AlbumArtist),
            album: clean(tag.and_then(|t| t.album())),
            year: tag.and_then(|t| t.year()).map(i64::from),
            track_no: tag.and_then(|t| t.track()),
            track_total: tag.and_then(|t| t.track_total()),
            disc_no: tag.and_then(|t| t.disk()),
            disc_total: tag.and_then(|t| t.disk_total()),
            genre: clean(tag.and_then(|t| t.genre())),
            composer: text(tag, &ItemKey::Composer),
            label: text(tag, &ItemKey::Label),
            comment: clean(tag.and_then(|t| t.comment())),
            grouping: text(tag, &ItemKey::ContentGroup),
        },
        pics,
    ))
}

pub fn get_track_tags(conn: &Connection, track_id: &str) -> Result<TrackTags, String> {
    let path: String = conn
        .query_row("SELECT path FROM tracks WHERE id = ?1", [track_id], |r| r.get(0))
        .map_err(|_| "unknown track".to_string())?;
    read_file(Path::new(&path))
}

/// The track modal's identity row (Phase C): WHICH file this editor is
/// editing — name, human folder, full path for the tooltip — whether it is
/// still pending (the stays-behind caution reads this), and the album the
/// row currently points at (the stepper's sibling list). Paths travel out
/// as DISPLAY strings only; revealing them stays `reveal_container`'s job.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackFile {
    pub file: String,
    pub folder: String,
    pub path: String,
    pub staged: bool,
    pub album_id: String,
}

pub fn track_file(conn: &Connection, track_id: &str) -> Result<TrackFile, String> {
    let (path, staged, album_id): (String, i64, String) = conn
        .query_row(
            "SELECT path, staged, album_id FROM tracks WHERE id = ?1",
            [track_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map_err(|_| "unknown track".to_string())?;
    let p = PathBuf::from(&path);
    let folder_raw = p
        .parent()
        .map(|d| d.display().to_string())
        .unwrap_or_default();
    // Home reads as "~": the folder line answers WHERE the file is, and an
    // absolute /home/<user> prefix is the part the eye skips.
    let folder = match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && folder_raw.starts_with(&home) => {
            format!("~{}", &folder_raw[home.len()..])
        }
        _ => folder_raw,
    };
    Ok(TrackFile {
        file: file_name(&p),
        folder,
        path,
        staged: staged != 0,
        album_id,
    })
}

/// First non-empty value + "tracks disagree" flag (trimmed exact compare).
fn consensus(values: impl Iterator<Item = String>) -> (String, bool) {
    let mut first = String::new();
    let mut disputed = false;
    for v in values {
        let v = v.trim();
        if v.is_empty() {
            continue;
        }
        if first.is_empty() {
            first = v.to_string();
        } else if v != first {
            disputed = true;
        }
    }
    (first, disputed)
}

fn consensus_opt<T: PartialEq + Copy>(
    values: impl Iterator<Item = Option<T>>,
) -> (Option<T>, bool) {
    let mut first: Option<T> = None;
    let mut disputed = false;
    for v in values.flatten() {
        match first {
            None => first = Some(v),
            Some(f) if v != f => disputed = true,
            _ => {}
        }
    }
    (first, disputed)
}

/// The census of one album field: distinct non-empty values with counts
/// (most frequent first, ties keep file order) and how many files carry any
/// value at all. Absence is never a contender — but it IS counted, so a
/// clear knows exactly how many files it touches.
fn conflicts_for(field: &str, values: impl Iterator<Item = String>) -> Option<FieldConflict> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut present = 0u32;
    for v in values {
        let v = v.trim().to_string();
        if v.is_empty() {
            continue;
        }
        present += 1;
        let e = counts.entry(v.clone()).or_default();
        if *e == 0 {
            order.push(v.clone());
        }
        *e += 1;
    }
    // Stable sort: equal counts keep their first-seen (file) order.
    order.sort_by_key(|v| std::cmp::Reverse(counts[v]));
    Some(FieldConflict {
        field: field.to_string(),
        values: order
            .into_iter()
            .map(|v| ValueCount {
                count: counts[&v],
                value: v,
            })
            .collect(),
        present,
    })
}

// ── Editor save semantics (Tag Editor Redesign spec, PLAN.md) ─────────────

/// Which shared fields the user actually edited. **Fields that were not
/// touched are never written** — the promise the whole save is built on.
/// Older callers pass `all()` (every field), which keeps the pre-redesign
/// behavior while still enjoying the per-file diff.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TouchedFields {
    pub album_artist: bool,
    pub album: bool,
    pub year: bool,
    pub genre: bool,
    pub composer: bool,
    pub label: bool,
    pub comment: bool,
    pub grouping: bool,
    pub track_total: bool,
    pub disc_total: bool,
}

impl TouchedFields {
    pub fn all() -> Self {
        Self {
            album_artist: true,
            album: true,
            year: true,
            genre: true,
            composer: true,
            label: true,
            comment: true,
            grouping: true,
            track_total: true,
            disc_total: true,
        }
    }
}

/// What to do with the files' embedded picture(s) — and (album mode) the
/// folder-art file — during a save. Externally tagged so the webview sends
/// plain JSON: `"keep"`, `"clear"`, `{"hash": "<blake3>"}` or
/// `{"upload": {"image": "<base64>", "mime": "image/jpeg"}}`.
#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ArtChange {
    #[default]
    Keep,
    Clear,
    /// Use an image the album already owns, identified by its blake3 hash
    /// (bytes come from wherever they live — a sibling file or the folder
    /// art — so multi-megabyte images never travel back over IPC).
    /// NEWTYPE on purpose: the wire shape the webview sends is
    /// `{"hash":"<hex>"}` (see src/lib/artChange.ts). As a STRUCT variant
    /// serde would demand the doubly-nested `{"hash":{"hash":"…"}}` and
    /// every candidate-pick from the UI died with `invalid args 'art':
    /// invalid type: string` — measured 2026-09-05 picking a cover on the
    /// ASMR compilation, where each of the 10 files carries its own
    /// picture so the candidate pile is the whole point of the modal.
    Hash(String),
    Upload {
        image: String,
        #[serde(default)]
        mime: String,
    },
}

/// A resolved artwork target: real bytes, sniffed (not claimed) mime, hash.
#[derive(Debug, Clone)]
struct ResolvedArt {
    data: Vec<u8>,
    mime: String,
    hash: String,
}

enum ArtTarget {
    Keep,
    Clear,
    Set(ResolvedArt),
}

/// The outcome of an album save — the modal prints its blast radius from
/// this, and the folder-art lines in `receipt` are where a user's own
/// `folder.jpg` being replaced or deleted gets said out loud.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveReport {
    pub written: usize,
    pub total: usize,
    pub receipt: Vec<String>,
}

fn validate(tags: &TrackTags) -> Result<(), String> {
    if let Some(y) = tags.year {
        if !(0..=9999).contains(&y) {
            return Err(format!("year must be a number up to 4 digits (got {y})"));
        }
    }
    Ok(())
}

/// Sniff the real format (never trusting a claimed mime), reject anything
/// undecodable or absurdly large, fingerprint the bytes.
fn resolve_bytes(data: Vec<u8>) -> Result<ResolvedArt, String> {
    if data.len() > 25 * 1024 * 1024 {
        return Err("cover image is larger than 25 MB".into());
    }
    let reader = image::ImageReader::new(std::io::Cursor::new(&data))
        .with_guessed_format()
        .map_err(|e| format!("cover image: {e}"))?;
    let mime = match reader.format() {
        Some(image::ImageFormat::Jpeg) => "image/jpeg",
        Some(image::ImageFormat::Png) => "image/png",
        Some(image::ImageFormat::WebP) => "image/webp",
        Some(image::ImageFormat::Gif) => "image/gif",
        Some(image::ImageFormat::Tiff) => "image/tiff",
        _ => return Err("not a readable image (jpeg, png, webp, gif or tiff)".into()),
    };
    Ok(ResolvedArt {
        hash: blake3::hash(&data).to_hex().to_string(),
        mime: mime.to_string(),
        data,
    })
}

fn to_jpeg(data: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(data).map_err(|e| format!("cover image: {e}"))?;
    let rgb = img.to_rgb8();
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 90)
        .encode(
            rgb.as_raw().as_slice(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("cover image: {e}"))?;
    Ok(out)
}

/// A file's picture list becomes exactly the target: cleared, or one
/// CoverFront frame. (Other tags of the file keep their pictures — the
/// reader-priority tag is the one the UI shows and the one we own.)
fn set_pictures(tag: &mut Tag, target: Option<&ResolvedArt>) {
    while !tag.pictures().is_empty() {
        tag.remove_picture(0);
    }
    if let Some(r) = target {
        tag.push_picture(Picture::new_unchecked(
            PictureType::CoverFront,
            Some(MimeType::from_str(&r.mime)),
            None,
            r.data.clone(),
        ));
    }
}

fn art_differs(pics: &[String], target: &ArtTarget) -> bool {
    match target {
        ArtTarget::Keep => false,
        ArtTarget::Clear => !pics.is_empty(),
        // "Already equal" means EXACTLY one picture with these bytes — a
        // file carrying a stray second frame differs, and the save
        // normalizes it (the MusicBee-style cleanup the spec decided on).
        ArtTarget::Set(r) => !(pics.len() == 1 && pics[0] == r.hash),
    }
}

fn fields_differ(a: &TrackTags, b: &TrackTags, t: &TouchedFields) -> bool {
    (t.album_artist && a.album_artist != b.album_artist)
        || (t.album && a.album != b.album)
        || (t.year && a.year != b.year)
        || (t.genre && a.genre != b.genre)
        || (t.composer && a.composer != b.composer)
        || (t.label && a.label != b.label)
        || (t.comment && a.comment != b.comment)
        || (t.grouping && a.grouping != b.grouping)
        || (t.track_total && a.track_total != b.track_total)
        || (t.disc_total && a.disc_total != b.disc_total)
}

/// Turn an ArtChange into concrete bytes (or the explicit Clear). `Hash`
/// looks through the album's files first, then its folder-art candidates —
/// the same inventory the candidate list is built from.
fn resolve_art(conn: &Connection, album_id: &str, art: &ArtChange) -> Result<ArtTarget, String> {
    match art {
        ArtChange::Keep => Ok(ArtTarget::Keep),
        ArtChange::Clear => Ok(ArtTarget::Clear),
        ArtChange::Upload { image, .. } => {
            use base64::Engine as _;
            let data = base64::engine::general_purpose::STANDARD
                .decode(image)
                .map_err(|e| format!("cover data: {e}"))?;
            Ok(ArtTarget::Set(resolve_bytes(data)?))
        }
        ArtChange::Hash(hash) => {
            let RowPaths(rows) = album_rows(conn, album_id)?;
            for (_, p) in &rows {
                let Ok(tagged) = lofty::read_from_path(p) else {
                    continue;
                };
                for pic in tagged.tags().iter().flat_map(|t| t.pictures()) {
                    if blake3::hash(pic.data()).to_hex().to_string() == *hash {
                        return Ok(ArtTarget::Set(resolve_bytes(pic.data().to_vec())?));
                    }
                }
            }
            if let Some(dir) = crate::library::artwork::dominant_dir(conn, album_id) {
                for path in crate::library::artwork::folder_art_all(&dir) {
                    if let Ok(bytes) = std::fs::read(&path) {
                        if blake3::hash(&bytes).to_hex().to_string() == *hash {
                            return Ok(ArtTarget::Set(resolve_bytes(bytes)?));
                        }
                    }
                }
            }
            Err(format!("selected artwork ({hash}) is no longer in this album"))
        }
    }
}

fn file_name(p: &Path) -> String {
    p.file_name().and_then(|s| s.to_str()).unwrap_or_default().to_string()
}

struct RowPaths(Vec<(String, PathBuf)>); // (track_id, path)

/// One image the album owns: every distinct embedded picture (count = files
/// carrying it) and/or a folder-art file by name. `preview` is a 256px
/// thumb:// URL written on demand; `current` marks what the cover pipeline
/// resolves today (the folder winner, else the largest embedded — the same
/// precedence the grid shows).
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtCandidate {
    pub hash: String,
    pub label: String,
    /// Files this image is embedded in (0 when it only exists as folder art).
    pub count: u32,
    /// Filename when a folder-art copy exists (the tile says which).
    pub folder: Option<String>,
    pub preview: String,
    /// 512px variant for the lightbox (same file the webview already caches
    /// for the tile, one size up — originals live in the files, not in cache).
    pub full: String,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtInventory {
    pub candidates: Vec<ArtCandidate>,
    pub current: Option<String>,
    /// Filename of the folder-art winner, when one exists (the modal says
    /// which file outranks the embedded pictures).
    pub folder_art: Option<String>,
    /// Files carrying at least one picture — "clear artwork" writes to
    /// exactly these, and the blast-radius line says so.
    pub files_with_art: u32,
    /// Hashes of the asked-about track's own pictures, in tag order (empty
    /// when no track was named) — the track modal's "this file carries" mark.
    pub track_pics: Vec<String>,
}

/// Everything the artwork selector shows. Previews (256px webp) are written
/// into the thumb cache under `art-<hash8>/` as a side effect — cheap once,
/// then the webview caches them by URL.
pub fn list_art_candidates(
    conn: &Connection,
    album_id: &str,
    track_id: Option<&str>,
    cache_dir: &Path,
) -> Result<ArtInventory, String> {
    let RowPaths(rows) = album_rows(conn, album_id)?;

    #[derive(Default)]
    struct Entry {
        count: u32,
        data: Vec<u8>,
        folder: Option<String>,
    }
    let mut by_hash: HashMap<String, Entry> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut largest_embedded: Option<(usize, String)> = None;
    let mut track_pics: Vec<String> = Vec::new();
    let mut files_with_art = 0usize;

    for (tid, path) in &rows {
        let tagged = match lofty::read_from_path(path) {
            Ok(t) => t,
            Err(_) => continue, // unreadable file contributes no candidates (and no error: browsing is not saving)
        };
        let mut seen: std::collections::HashSet<String> = Default::default();
        for pic in tagged.tags().iter().flat_map(|t| t.pictures()) {
            let hash = blake3::hash(pic.data()).to_hex().to_string();
            if tid.as_str() == track_id.unwrap_or("") {
                track_pics.push(hash.clone());
            }
            let len = pic.data().len();
            if largest_embedded.as_ref().is_none_or(|(n, _)| len > *n) {
                largest_embedded = Some((len, hash.clone()));
            }
            if !seen.insert(hash.clone()) {
                continue; // one census per file
            }
            let e = by_hash.entry(hash.clone()).or_default();
            if e.count == 0 {
                order.push(hash);
            }
            e.count += 1;
            if e.data.is_empty() {
                e.data = pic.data().to_vec();
            }
        }
        if !seen.is_empty() {
            files_with_art += 1;
        }
    }

    let mut folder_winner: Option<(PathBuf, String)> = None;
    if let Some(dir) = crate::library::artwork::dominant_dir(conn, album_id) {
        for path in crate::library::artwork::folder_art_all(&dir) {
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let hash = blake3::hash(&bytes).to_hex().to_string();
            if folder_winner.is_none() {
                folder_winner = Some((path.clone(), hash.clone()));
            }
            let e = by_hash.entry(hash.clone()).or_default();
            if e.count == 0 && e.folder.is_none() {
                order.push(hash);
            }
            if e.data.is_empty() {
                e.data = bytes;
            }
            if e.folder.is_none() {
                e.folder = Some(file_name(&path));
            }
        }
    }

    let mut candidates: Vec<ArtCandidate> = Vec::new();
    for hash in order {
        let Some(entry) = by_hash.get(&hash) else { continue };
        let Ok(img) = image::load_from_memory(&entry.data) else {
            continue; // undecodable bytes show no tile (and write no thumb)
        };
        let h8 = hash[..8].to_string();
        let dir = thumbs_preview_dir(cache_dir, &h8);
        let url = format!("thumb://art-{h8}/256.webp");
        if !dir.join("256.webp").is_file() {
            let thumb = img.thumbnail(256, 256);
            let _ = std::fs::create_dir_all(&dir);
            let _ = thumb.save(dir.join("256.webp"));
        }
        if !dir.join("512.webp").is_file() {
            let thumb = img.thumbnail(512, 512);
            let _ = std::fs::create_dir_all(&dir);
            let _ = thumb.save(dir.join("512.webp"));
        }
        let label = match (&entry.folder, entry.count) {
            (Some(name), 0) => name.clone(),
            (Some(name), n) => format!("{name} · in {n} files"),
            (None, 1) => "in 1 file".into(),
            (None, n) => format!("in {n} files"),
        };
        candidates.push(ArtCandidate {
            hash,
            label,
            count: entry.count,
            folder: entry.folder.clone(),
            preview: url,
            // Content-addressed: a URL whose bytes can never change carries
            // `immutable` honestly, and a response cached by an older build
            // (which once mislabeled 256 bytes under this URL) can never be
            // handed back — the hash moves with the pixels.
            full: format!("thumb://art-{h8}/512-{h8}.webp"),
        });
    }
    candidates.sort_by_key(|c| std::cmp::Reverse(c.count));

    let current = folder_winner
        .as_ref()
        .map(|(_, h)| h.clone())
        .or_else(|| largest_embedded.map(|(_, h)| h));
    Ok(ArtInventory {
        candidates,
        current,
        folder_art: folder_winner.map(|(p, _)| file_name(&p)),
        files_with_art: files_with_art as u32,
        track_pics,
    })
}

fn thumbs_preview_dir(cache_dir: &Path, hash8: &str) -> PathBuf {
    crate::library::artwork::thumbs_dir(cache_dir).join(format!("art-{hash8}"))
}

fn album_rows(conn: &Connection, album_id: &str) -> Result<RowPaths, String> {
    let mut stmt = conn
        .prepare("SELECT id, path, disc, track, title FROM tracks WHERE album_id = ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([album_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut ordered: Vec<(i64, Option<i64>, String, String, PathBuf)> = Vec::new();
    for row in rows {
        let (id, p, disc, track, title) = row.map_err(|e| e.to_string())?;
        ordered.push((disc, track, title, id, PathBuf::from(p)));
    }
    // Same rule as every serving path: unnumbered tracks alphabetically
    // (sort_key-folded) before numbered ones — SQLite would split case.
    ordered.sort_by_key(|r| super::track_order_key(r.0, r.1, &r.2));
    let out: Vec<(String, PathBuf)> =
        ordered.into_iter().map(|(_, _, _, id, p)| (id, p)).collect();
    if out.is_empty() {
        return Err("unknown or empty album".into());
    }
    Ok(RowPaths(out))
}

pub fn get_album_tags(conn: &Connection, album_id: &str) -> Result<AlbumTags, String> {
    let RowPaths(rows) = album_rows(conn, album_id)?;
    let files: Vec<TrackTags> = rows
        .iter()
        .map(|(_, p)| read_file(p))
        .collect::<Result<_, _>>()?;

    let (album_artist, aa_d) = consensus(files.iter().map(|f| f.album_artist.clone()));
    let (album, al_d) = consensus(files.iter().map(|f| f.album.clone()));
    let (year, y_d) = consensus_opt(files.iter().map(|f| f.year));
    let (genre, g_d) = consensus(files.iter().map(|f| f.genre.clone()));
    let (composer, c_d) = consensus(files.iter().map(|f| f.composer.clone()));
    let (label, l_d) = consensus(files.iter().map(|f| f.label.clone()));
    let (grouping, gr_d) = consensus(files.iter().map(|f| f.grouping.clone()));
    let (comment, co_d) = consensus(files.iter().map(|f| f.comment.clone()));
    let (track_total, tt_d) = consensus_opt(files.iter().map(|f| f.track_total));
    let (disc_total, dt_d) = consensus_opt(files.iter().map(|f| f.disc_total));

    // The same disagreements, as data: which values compete and how many
    // files carry each. The UI shows these instead of a bare `•`.
    let mut conflicts: Vec<FieldConflict> = Vec::new();
    for (field, vals) in [
        ("albumArtist", files.iter().map(|f| f.album_artist.clone()).collect::<Vec<_>>()),
        ("album", files.iter().map(|f| f.album.clone()).collect::<Vec<_>>()),
        ("genre", files.iter().map(|f| f.genre.clone()).collect::<Vec<_>>()),
        ("composer", files.iter().map(|f| f.composer.clone()).collect::<Vec<_>>()),
        ("label", files.iter().map(|f| f.label.clone()).collect::<Vec<_>>()),
        ("grouping", files.iter().map(|f| f.grouping.clone()).collect::<Vec<_>>()),
        ("comment", files.iter().map(|f| f.comment.clone()).collect::<Vec<_>>()),
        (
            "year",
            files
                .iter()
                .map(|f| f.year.map(|y| y.to_string()).unwrap_or_default())
                .collect::<Vec<_>>(),
        ),
        (
            "trackTotal",
            files
                .iter()
                .map(|f| f.track_total.map(|n| n.to_string()).unwrap_or_default())
                .collect::<Vec<_>>(),
        ),
        (
            "discTotal",
            files
                .iter()
                .map(|f| f.disc_total.map(|n| n.to_string()).unwrap_or_default())
                .collect::<Vec<_>>(),
        ),
    ] {
        if let Some(c) = conflicts_for(field, vals.into_iter()) {
            conflicts.push(c);
        }
    }

    Ok(AlbumTags {
        album_artist,
        album_artist_disputed: aa_d,
        album,
        album_disputed: al_d,
        year,
        year_disputed: y_d,
        genre,
        genre_disputed: g_d,
        composer,
        composer_disputed: c_d,
        label,
        label_disputed: l_d,
        grouping,
        grouping_disputed: gr_d,
        comment,
        comment_disputed: co_d,
        track_total,
        track_total_disputed: tt_d,
        disc_total,
        disc_total_disputed: dt_d,
        conflicts,
        file_count: rows.len(),
    })
}

/// The primary-or-first tag, creating one when the file has none at all.
fn ensure_tag(tagged: &mut lofty::file::TaggedFile) -> &mut Tag {
    if tagged.primary_tag().is_some() {
        return tagged.primary_tag_mut().expect("primary");
    }
    if tagged.first_tag().is_some() {
        return tagged.first_tag_mut().expect("first");
    }
    let tag_type = tagged.file_type().primary_tag_type();
    tagged.insert_tag(Tag::new(tag_type));
    // insert_tag pushes to the end; first_tag_mut now finds it.
    tagged.first_tag_mut().expect("inserted tag")
}

fn set_text(tag: &mut Tag, key: ItemKey, value: &str) {
    tag.remove_key(&key);
    let v = value.trim();
    if !v.is_empty() {
        tag.insert_text(key, v.to_string());
    }
}

/// Album-level fields ONLY — safe to stamp on every file of an album
/// (per-track fields are never touched here; empty strings still clear).
/// A field is written iff `sel` says it was touched — "fields you did not
/// edit are never written" is enforced here, at the tag level.
fn apply_shared(tag: &mut Tag, t: &TrackTags, sel: &TouchedFields) {
    if sel.album_artist {
        set_text(tag, ItemKey::AlbumArtist, &t.album_artist);
    }
    if sel.album {
        set_text(tag, ItemKey::AlbumTitle, &t.album);
    }
    if sel.genre {
        set_text(tag, ItemKey::Genre, &t.genre);
    }
    if sel.composer {
        set_text(tag, ItemKey::Composer, &t.composer);
    }
    if sel.label {
        set_text(tag, ItemKey::Label, &t.label);
    }
    if sel.comment {
        set_text(tag, ItemKey::Comment, &t.comment);
    }
    if sel.grouping {
        set_text(tag, ItemKey::ContentGroup, &t.grouping);
    }
    // NOTE: the year() accessor falls back to RecordingDate (and set_year
    // rewrites an existing RecordingDate), so clearing must purge BOTH or
    // the old value keeps reading back.
    if sel.year {
        if let Some(y) = t.year {
            tag.set_year(u32::try_from(y.max(0)).unwrap_or(u32::MAX));
        } else {
            tag.remove_key(&ItemKey::Year);
            tag.remove_key(&ItemKey::RecordingDate);
        }
    }
    if sel.track_total {
        tag.remove_key(&ItemKey::TrackTotal);
        if let Some(n) = t.track_total {
            tag.set_track_total(n);
        }
    }
    if sel.disc_total {
        tag.remove_key(&ItemKey::DiscTotal);
        if let Some(n) = t.disc_total {
            tag.set_disk_total(n);
        }
    }
}

/// Per-track fields: title, artist, number and disc (album-mode rows carry
/// the first and last two; artist is single-track-mode only).
fn apply_per_track(
    tag: &mut Tag,
    title: &str,
    artist: Option<&str>,
    track_no: Option<u32>,
    disc_no: Option<u32>,
) {
    set_text(tag, ItemKey::TrackTitle, title);
    if let Some(a) = artist {
        set_text(tag, ItemKey::TrackArtist, a);
    }
    tag.remove_key(&ItemKey::TrackNumber);
    if let Some(n) = track_no {
        tag.set_track(n);
    }
    tag.remove_key(&ItemKey::DiscNumber);
    if let Some(n) = disc_no {
        tag.set_disk(n);
    }
}

fn write_file(path: &Path, f: impl FnOnce(&mut Tag)) -> Result<(), String> {
    let mut tagged = lofty::read_from_path(path).map_err(|e| format!("{path:?}: {e}"))?;
    let tag = ensure_tag(&mut tagged);
    f(tag);
    let expected = tag.clone();
    // TEMP + RENAME, and a CATCHED PANIC. lofty 0.22.4's write_id3v1
    // truncates the 30-byte legacy field with a byte slice and PANICS on
    // a field whose 28th byte lands inside a multi-byte char (measured:
    // 'í' at 27..29, album Road To The Unknown, artwork removal 2026-09-05
    // — the panic escaped the save task as a red banner and the retry
    // fails forever on the same file). Saving into a sibling temp keeps
    // the ORIGINAL untouched through such a panic, and catch_unwind
    // downgrades it from "task died, whole save aborted, file possibly
    // half-written" to one honest per-file line in the report. The
    // upstream fix lives in lofty ≥0.23; upgrading the API across
    // tags/scan/artwork is a dedicated session — until then the writer
    // is armored.
    let tmp = path.with_extension("songstress-tmp");
    // lofty's save_to_path EDITS its target in place (it never creates),
    // so the temp starts as a byte-copy of the original; the panic can
    // then only ever damage the copy.
    std::fs::copy(path, &tmp).map_err(|e| format!("{path:?}: temp: {e}"))?;
    // Fix the known panic at its source when we can: lofty's write_id3v1
    // truncates each legacy field with a BYTE slice, so any field longer
    // than its budget whose cut lands inside a multi-byte char panics.
    // Char-boundary-trim every ID3v1 text field to its exact budget and
    // the writer can no longer fall in that hole.
    if let Some(v1) = tagged.tag_mut(lofty::tag::TagType::Id3v1) {
        for key in [
            ItemKey::TrackTitle,
            ItemKey::TrackArtist,
            ItemKey::AlbumTitle,
            ItemKey::Comment,
        ] {
            if let Some(s) = v1.get_string(&key) {
                let budget = if key == ItemKey::Comment { 28 } else { 30 };
                if s.len() > budget {
                    let mut cut = budget;
                    while !s.is_char_boundary(cut) {
                        cut -= 1;
                    }
                    let trimmed = s[..cut].to_string();
                    v1.remove_key(&key);
                    v1.insert_text(key, trimmed);
                }
            }
        }
    }
    let mut save = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tagged.save_to_path(&tmp, lofty::config::WriteOptions::default())
    }));
    if matches!(save, Err(_)) {
        // Something in the legacy tag still bites lofty's writer (an
        // unmapped genre string, a field we do not touch): retry ONCE
        // with the ID3v1 tag gone. ID3v1 is a 1997-era duplicate of the
        // v2 fields we just wrote — dropping it costs the user nothing
        // modern, and a file that refuses the writer forever costs
        // everything.
        let _ = std::fs::remove_file(&tmp);
        std::fs::copy(path, &tmp).map_err(|e| format!("{path:?}: temp: {e}"))?;
        tagged.remove(lofty::tag::TagType::Id3v1);
        save = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tagged.save_to_path(&tmp, lofty::config::WriteOptions::default())
        }));
    }
    match save {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            let _ = std::fs::remove_file(&tmp);
            // "No format could be determined" is lofty's PROBE refusing
            // the file, not a write failure — measured 2026-09-05 on the
            // owner's "19 Flash.mp3": a fat ID3v2.3 block (387 KB APIC)
            // followed at its DECLARED end by a second ID3v2.4 block. The
            // merged view reads fine (extension hint), but the write
            // probe cannot place the audio, so NOTHING can be saved to
            // any path. The repair strips the stacked headers by raw size
            // math first (cut_leading_tags), which provably returns the
            // file to lofty's sight (verified on /tmp copies of the
            // incident file and its mutants). The verify path below can
            // never be reached for such a file — no save ever returned
            // Ok — so this is the single extra door.
            if e.to_string().contains("No format could be determined") {
                return repair_stacked_tags(path, &expected).map_err(|re| {
                    format!("{path:?}: {e} (stacked-tag repair: {re})")
                });
            }
            return Err(format!("{path:?}: {e}"));
        }
        Err(_) => {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("{path:?}: tag writer panicked twice (id3v1?)"));
        }
    }
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{path:?}: rename: {e}")
    })?;

    // Verify the edit actually landed. Some files (Lavf52-era muxers) carry
    // STACKED ID3v2 tags: the reader merges them, but the writer only
    // rewrites the first block, so a later block's stale frames win on
    // reread and the save silently vanishes (Ok(()) and all). When that
    // happens, strip every tag of this type straight from the file and
    // rewrite the merged+edited tag as the only one.
    let reread = lofty::read_from_path(path).map_err(|e| format!("{path:?}: {e}"))?;    let landed = reread
        .primary_tag()
        .or_else(|| reread.first_tag())
        .is_some_and(|t| {
            expected.items().all(|item| {
                t.get_string(item.key()).map(str::to_string)
                    == item.value().text().map(str::to_string)
            })
        });
    if !landed {
        repair_stacked_tags(path, &expected)?;
    }
    Ok(())
}

/// Remove every leading ID3v2 block from a file using ONLY the header's
/// own size field — no tag library involved, so it works on exactly the
/// files lofty's probe goes blind on (adjacent stacked ID3v2 blocks, the
/// "No format could be determined" save failure — owner's "19 Flash.mp3",
/// 2026-09-05: ID3v2.3 covering a 387 KB APIC, and an ID3v2.4 starting
/// EXACTLY at the v2.3's declared end, before the MPEG frames).
/// Anything after the last header (audio, or a tag format the caller can
/// handle) is preserved byte-for-byte; a file that does not start with
/// ID3 is untouched. Returns the number of bytes removed.
fn cut_leading_tags(path: &Path) -> Result<usize, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path:?}: cut: {e}"))?;
    let mut off = 0usize;
    while bytes[off..].starts_with(b"ID3") && off + 10 <= bytes.len() {
        let ver = bytes[off + 3];
        let size = match ver {
            2 => u32::from_be_bytes([0, bytes[off + 6], bytes[off + 7], bytes[off + 8]]) as usize,
            3 | 4 => {
                ((bytes[off + 6] as usize) << 21)
                    | ((bytes[off + 7] as usize) << 14)
                    | ((bytes[off + 8] as usize) << 7)
                    | bytes[off + 9] as usize
            }
            // Unknown version: trust nothing, cut nothing further.
            _ => break,
        };
        let next = off + 10 + size;
        if next > bytes.len() {
            // A header claiming more than the file holds is noise, not a
            // tag worth cutting; stop rather than eat the audio.
            break;
        }
        off = next;
    }
    if off == 0 {
        return Ok(0);
    }
    std::fs::write(path, &bytes[off..]).map_err(|e| format!("{path:?}: cut write: {e}"))?;
    Ok(off)
}

/// Strip every on-disk tag of `expected`'s type, then write `expected` back
/// as the single tag. Lossless for stacked-ID3v2 files: the merged view the
/// reader produced already contains the union of all stacked blocks.
///
/// The copy is first run through `cut_leading_tags` — raw arithmetic, no
/// lofty — because a fat stacked-ID3v2 file makes lofty's WRITE probe
/// blind ("No format could be determined", measured 2026-09-05), and the
/// repair may therefore be reaching a file lofty cannot even look at;
/// the strip loop below needs the probe to work.
fn repair_stacked_tags(path: &Path, expected: &Tag) -> Result<(), String> {
    let tt = expected.tag_type();
    // Same armor as write_file, one notch stronger: lofty's writer panics
    // LEAVE THE TARGET HALF-WRITTEN (measured 2026-09-04: the pre-armor
    // in-place save died inside write_id3v1 and the file carried a
    // garbage tail until the next scan re-read it). So the repair works
    // on a sibling copy — SAME EXTENSION, lofty probes the format by
    // extension — and the original is replaced only by rename once the
    // rewrite has verified. The id3v1 fields are char-trimmed by the
    // same rule as write_file, and a panic anywhere is caught, not
    // allowed to escape the save task.
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("{path:?}: no file name"))?;
    let tmp = path.with_file_name(format!(".songstress-repair-{name}"));
    let res = (|| -> Result<(), String> {
        std::fs::copy(path, &tmp).map_err(|e| format!("{path:?}: temp: {e}"))?;
        cut_leading_tags(&tmp)?;
        for attempt in 1..=16 {
            let present = lofty::read_from_path(&tmp)
                .map_err(|e| format!("{path:?}: {e}"))?
                .contains_tag_type(tt);
            if !present {
                break;
            }
            let removed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                tt.remove_from_path(&tmp)
            }));
            match removed {
                Err(_) => {
                    return Err(format!("{path:?}: tag writer panicked (strip {attempt})"))
                }
                Ok(r) => r.map_err(|e| format!("{path:?}: strip {attempt}: {e}"))?,
            }
        }
        let mut tagged = lofty::read_from_path(&tmp).map_err(|e| format!("{path:?}: {e}"))?;
        tagged.insert_tag(expected.clone());
        let saved = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tagged.save_to_path(&tmp, lofty::config::WriteOptions::default())
        }))
        .map_err(|_| format!("{path:?}: tag writer panicked (rewrite)"))?;
        saved.map_err(|e| format!("{path:?}: rewrite: {e}"))?;
        Ok(())
    })();
    if let Err(e) = res {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        format!("{path:?}: rename: {e}")
    })?;

    // The repair must land; a second failure is a hard error, never silence.
    let check = lofty::read_from_path(path).map_err(|e| format!("{path:?}: {e}"))?;
    let ok = check
        .primary_tag()
        .or_else(|| check.first_tag())
        .is_some_and(|t| {
            expected.items().all(|item| {
                t.get_string(item.key()).map(str::to_string)
                    == item.value().text().map(str::to_string)
            })
        });
    if !ok {
        return Err(format!("{path:?}: tag rewrite did not stick"));
    }
    Ok(())
}

pub fn save_track_tags(
    conn: &Connection,
    track_id: &str,
    tags: &TrackTags,
    art: &ArtChange,
) -> Result<(), String> {
    validate(tags)?;
    let (path, album_id): (String, String) = conn
        .query_row(
            "SELECT path, album_id FROM tracks WHERE id = ?1",
            [track_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| "unknown track".to_string())?;
    let target = resolve_art(conn, &album_id, art)?;
    let tags = tags.clone();
    // A track save always writes its one file — the modal shows every field
    // of it, and a one-file blast radius needs no diff to state it.
    write_file(Path::new(&path), |tag| {
        apply_shared(tag, &tags, &TouchedFields::all());
        apply_per_track(
            tag,
            &tags.title,
            Some(&tags.artist),
            tags.track_no,
            tags.disc_no,
        );
        match &target {
            ArtTarget::Keep => {}
            ArtTarget::Clear => set_pictures(tag, None),
            ArtTarget::Set(r) => set_pictures(tag, Some(r)),
        }
    })
}

/// Stamp the touched shared fields + the artwork decision onto the files
/// that actually differ — the write set, not the whole album. Folder art is
/// replaced/created/removed alongside (folder art outranks embedded, so the
/// editor must not pretend otherwise) and every folder-level act lands in
/// the receipt. Every file is attempted even if some fail; errors are
/// joined, receipts kept out of them because a receipt is not an error.
pub fn save_album_tags(
    conn: &Connection,
    album_id: &str,
    shared: &TrackTags,
    touched: &TouchedFields,
    art: &ArtChange,
) -> Result<SaveReport, String> {
    validate(shared)?;
    let RowPaths(rows) = album_rows(conn, album_id)?;
    let total = rows.len();
    let shared = shared.clone();
    let target = resolve_art(conn, album_id, art)?;

    let mut written = 0usize;
    let mut errors: Vec<String> = Vec::new();
    let mut receipt: Vec<String> = Vec::new();
    for (_, path) in &rows {
        let (current, pics) = match read_file_full(path) {
            Ok(v) => v,
            Err(e) => {
                errors.push(e);
                continue;
            }
        };
        if !fields_differ(&current, &shared, touched) && !art_differs(&pics, &target) {
            continue; // already says exactly this — do not touch its mtime
        }
        let result = write_file(path, |tag| {
            apply_shared(tag, &shared, touched);
            match &target {
                ArtTarget::Keep => {}
                ArtTarget::Clear => set_pictures(tag, None),
                ArtTarget::Set(r) => set_pictures(tag, Some(r)),
            }
        });
        match result {
            Ok(()) => written += 1,
            Err(e) => errors.push(e),
        }
    }

    // Folder art moves with the album's decision. Encoding to JPEG here (not
    // storing a PNG under .jpg) keeps the file honest for every other
    // reader in the wild; a replaced winner keeps its name whatever its
    // bytes, because the user's folder is where they left it.
    if !matches!(target, ArtTarget::Keep) {
        if let Some(dir) = crate::library::artwork::dominant_dir(conn, album_id) {
            match &target {
                ArtTarget::Clear => {
                    if let Some(w) = crate::library::artwork::folder_art(&dir) {
                        match std::fs::remove_file(&w) {
                            Ok(()) => receipt.push(format!("removed {}", file_name(&w))),
                            Err(e) => errors.push(format!("{}: {e}", w.display())),
                        }
                    }
                }
                ArtTarget::Set(r) => {
                    match to_jpeg(&r.data) {
                        Ok(jpeg) => {
                            let (path, verb) =
                                match crate::library::artwork::folder_art(&dir) {
                                    Some(w) => (w, "replaced"),
                                    None => (dir.join("cover.jpg"), "wrote"),
                                };
                            match std::fs::write(&path, &jpeg) {
                                Ok(()) => {
                                    receipt.push(format!("{} {}", verb, file_name(&path)))
                                }
                                Err(e) => errors.push(format!("{}: {e}", path.display())),
                            }
                        }
                        Err(e) => errors.push(e),
                    }
                }
                ArtTarget::Keep => unreachable!(),
            }
        }
    }

    if errors.is_empty() {
        // The albums ROW must follow a year edit now, not someday: the
        // scan that re-reads these files only ever INSERTs the album row
        // (ON CONFLICT DO NOTHING, and retag-adopt matches on
        // artist+title), so without this block clearing Year in the modal
        // wrote every file and updated NOTHING — the grid and the panel
        // kept printing the fossil year (owner report 2026-09-05, the
        // 218-file Anison compilation). Title and artist stay the SCAN's
        // to re-key (identity); year is pure stored metadata, so a
        // successful album save mirrors it into the row directly. The
        // TRACK modal deliberately does not: one file's year is not an
        // album consensus.
        if touched.year {
            conn.execute(
                "UPDATE albums SET year = ?2 WHERE id = ?1",
                rusqlite::params![album_id, shared.year],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(SaveReport {
            written,
            total,
            receipt,
        })
    } else {
        Err(errors.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixtures_src() -> PathBuf {
        Path::new("fixtures/library").to_path_buf()
    }

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("songstress-tags-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

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

    fn scanned_db(root: &Path) -> Connection {
        let dbp = root.join("t.db");
        let mut conn = crate::library::db::open(&dbp).expect("open");
        crate::library::scan::run_scan(&mut conn, root, |_, _| {}).expect("scan");
        conn
    }

    fn track_id(conn: &Connection, title: &str) -> String {
        conn.query_row(
            "SELECT id FROM tracks WHERE title = ?1",
            [title],
            |r| r.get(0),
        )
        .expect("track id")
    }

    fn album_id(conn: &Connection, title: &str, artist_name: &str) -> String {
        conn.query_row(
            "SELECT al.id FROM albums al JOIN artists ar ON ar.id = al.artist_id
             WHERE al.title = ?1 AND ar.name = ?2",
            [title, artist_name],
            |r| r.get(0),
        )
        .expect("album id")
    }

    fn count_albums(conn: &Connection, title: &str) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM albums WHERE title = ?1",
            [title],
            |r| r.get(0),
        )
        .expect("count")
    }

    #[test]
    fn track_file_names_the_file_and_flags_pending() {
        let root = temp_dir("track-file");
        copy_tree(&fixtures_src(), &root);
        let conn = scanned_db(&root);
        let tid = track_id(&conn, "Silent Echoes");
        let album = conn
            .query_row(
                "SELECT album_id FROM tracks WHERE id = ?1",
                [&tid],
                |r| r.get::<_, String>(0),
            )
            .expect("album id");
        let info = track_file(&conn, &tid).expect("track file");
        // The identity row shows a real file name; the stepper reads the id.
        assert!(
            info.file.starts_with("01 - Silent Echoes.") && info.file.contains("."),
            "got {:?}",
            info.file
        );
        assert_eq!(info.album_id, album);
        assert!(!info.staged);
        assert!(info.path.contains(&info.file));
        // The stays-behind caution's gate: a pending file reads as pending.
        conn.execute("UPDATE tracks SET staged = 1 WHERE id = ?1", [&tid])
            .expect("flag");
        assert!(track_file(&conn, &tid).expect("staged").staged);
    }

    #[test]
    fn save_read_roundtrip_incl_clearing() {
        let root = temp_dir("roundtrip");
        copy_tree(&fixtures_src(), &root);
        let conn = scanned_db(&root);
        let tid = track_id(&conn, "Silent Echoes");

        // Baseline read works on real fixture files.
        let before = get_track_tags(&conn, &tid).expect("read");
        assert_eq!(before.title, "Silent Echoes");
        assert_eq!(before.artist, "Helloween");

        let full = TrackTags {
            title: "Renamed Echo".into(),
            artist: "Helloween".into(),
            album_artist: "Helloween".into(),
            album: "Giants & Monsters".into(),
            year: Some(1985),
            track_no: Some(7),
            track_total: Some(12),
            disc_no: Some(2),
            disc_total: Some(3),
            genre: "Power Metal".into(),
            composer: "Kai Hansen".into(),
            label: "Noise Records".into(),
            comment: "edited by the tag editor test".into(),
            grouping: "Ween Era".into(),
        };
        save_track_tags(&conn, &tid, &full, &ArtChange::Keep).expect("save");

        // Read straight from the FILE (no rescan involved).
        let reread = get_track_tags(&conn, &tid).expect("reread");
        assert_eq!(reread, full);

        // Clearing: empty strings / None must remove keys, not write "".
        let mut cleared = full.clone();
        cleared.composer.clear();
        cleared.label.clear();
        cleared.comment.clear();
        cleared.grouping.clear();
        cleared.year = None;
        cleared.track_total = None;
        cleared.disc_total = None;
        save_track_tags(&conn, &tid, &cleared, &ArtChange::Keep).expect("clear-save");
        let after = get_track_tags(&conn, &tid).expect("post-clear read");
        assert_eq!(after, cleared);

        // Edited mtime ⇒ incremental rescan re-parses through real grouping.
        drop(conn);
        let mut conn = crate::library::db::open(&root.join("t.db")).expect("reopen");
        let counts = crate::library::scan::run_scan(&mut conn, &root, |_, _| {}).expect("rescan");
        assert_eq!(counts.updated, 1, "the edited file was re-parsed");
        let db_title: String = conn
            .query_row("SELECT title FROM tracks WHERE id = ?1", [&tid], |r| r.get(0))
            .expect("db title");
        assert_eq!(db_title, "Renamed Echo");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Minimal untagged MP3: two MPEG-1 Layer III frames (128 kbps,
    /// 44.1 kHz, 417 B each, zero payload). One frame fails lofty's
    /// validation — it peeks at the next header.
    fn write_tagless_mp3(path: &Path) {

        let mut one = vec![0xFFu8, 0xFB, 0x90, 0x00];
        one.resize(417, 0);
        let mut data = one.clone();
        data.extend_from_slice(&one);
        std::fs::write(path, data).expect("write mp3");
    }

    #[test]
    fn tagless_file_gets_a_created_tag() {
        let root = temp_dir("tagless");
        write_tagless_mp3(&root.join("bare-song.mp3"));
        let conn = scanned_db(&root);

        let tid: String = conn
            .query_row(
                "SELECT id FROM tracks WHERE path LIKE '%bare-song.mp3'",
                [],
                |r| r.get(0),
            )
            .expect("scanned tagless file");

        let before = get_track_tags(&conn, &tid).expect("read tagless");
        assert_eq!(
            before,
            TrackTags::default(),
            "no containers anywhere ⇒ all fields empty"
        );

        let tags = TrackTags {
            title: "Born Empty".into(),
            artist: "Nobody".into(),
            album: "Blank Slate".into(),
            year: Some(2026),
            track_no: Some(1),
            ..Default::default()
        };
        save_track_tags(&conn, &tid, &tags, &ArtChange::Keep).expect("save onto tagless file");
        let reread = get_track_tags(&conn, &tid).expect("reread");
        assert_eq!(reread, tags);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn fixing_albumartist_merges_split_albums_on_rescan() {
        let root = temp_dir("merge");
        let src = fixtures_src()
            .join("Various Artists/Synth Wars (2019)/02 - Chrome Sunset.flac");

        // Same album title+year, two DIFFERENT known release artists ⇒ the
        // scanner keeps them apart. This is the misbehaving-library case.
        let stage = |dir: &str, aa: &str| {
            let d = root.join(dir);
            std::fs::create_dir_all(&d).unwrap();
            let dest = d.join("01 - Half.flac");
            std::fs::copy(&src, &dest).unwrap();
            let mut tagged = lofty::read_from_path(&dest).unwrap();
            let tag = tagged.primary_tag_mut().unwrap();
            tag.remove_key(&ItemKey::AlbumArtist);
            tag.insert_text(ItemKey::AlbumArtist, aa.to_string());
            tag.remove_key(&ItemKey::AlbumTitle);
            tag.insert_text(ItemKey::AlbumTitle, "Split Me".to_string());
            tag.remove_key(&ItemKey::Year);
            tag.insert_text(ItemKey::Year, "2000".to_string());
            tag.set_track(1);
            tagged
                .save_to_path(&dest, lofty::config::WriteOptions::default())
                .unwrap();
        };
        stage("Band/First Half", "The Real Band");
        stage("Other/Second Half", "Wrong Artist");
        let conn = scanned_db(&root);
        assert_eq!(count_albums(&conn, "Split Me"), 2, "split as staged");

        // Editor fixes BOTH halves to the same albumartist. Shared struct
        // carries ONLY album-level fields (per-track fields are never touched
        // by album-mode saves); album+year must match on both halves or the
        // scanner would still key them apart.
        let shared = TrackTags {
            album_artist: "The Real Band".into(),
            album: "Split Me".into(),
            year: Some(2000),
            ..Default::default()
        };
        for artist_name in ["The Real Band", "Wrong Artist"] {
            let aid = album_id(&conn, "Split Me", artist_name);
            save_album_tags(
                &conn,
                &aid,
                &shared,
                &TouchedFields::all(),
                &ArtChange::Keep,
            )
            .expect("save album half");
        }

        // Rescan through the REAL grouping path: now one album, two tracks.
        drop(conn);
        let mut conn = crate::library::db::open(&root.join("t.db")).expect("reopen");
        crate::library::scan::run_scan(&mut conn, &root, |_, _| {}).expect("rescan");
        assert_eq!(count_albums(&conn, "Split Me"), 1, "merged after retag");
        let n: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM tracks t JOIN albums a ON a.id = t.album_id
                 WHERE a.title = 'Split Me'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 2);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Regression for the silent-vanish save: some real MP3s (Lavf52-era
    /// muxers) carry STACKED ID3v2 tags. The reader merges them, but a plain
    /// `save_to_path` only rewrites the first block — a later block's stale
    /// frames win on reread and the edit silently disappears. write_file must
    /// detect that and collapse the file to a single merged tag.
    #[test]
    fn stacked_id3v2_save_actually_lands() {
        let root = temp_dir("stacked");
        // Two MPEG-1 Layer III frames (see write_tagless_mp3 for why two).
        let mut one = vec![0xFFu8, 0xFB, 0x90, 0x00];
        one.resize(417, 0);
        let mut audio = one.clone();
        audio.extend_from_slice(&one);

        // Build two ID3v2 blocks by hand: #1 holds only the compilation flag,
        // #2 (minimal, hand-rolled — dump_to pads too much for the probe's
        // junk search) holds the "real" tags. Concatenated = the pathological
        // layout the real Lavf52-era file has.
        use lofty::tag::TagExt;
        let mut t1 = Tag::new(lofty::tag::TagType::Id3v2);
        t1.insert_text(ItemKey::FlagCompilation, "1".into());
        let mut bytes = Vec::new();
        t1.dump_to(&mut bytes, lofty::config::WriteOptions::default())
            .unwrap();
        // Minimal ID3v2.4 tag: header + one TALB frame ("BEST of STEREOPONY").
        let album: &[u8] = b"BEST of STEREOPONY";
        let frame_data: Vec<u8> = std::iter::once(0u8) // Latin-1 encoding
            .chain(album.iter().copied())
            .collect();
        let frame_size = frame_data.len() as u32;
        let mut t2 = Vec::new();
        t2.extend_from_slice(b"ID3\x04\x00\x00");
        let sz = frame_data.len() + 10; // frame header (10) + data
        t2.extend_from_slice(&[
            ((sz >> 21) & 0x7F) as u8,
            ((sz >> 14) & 0x7F) as u8,
            ((sz >> 7) & 0x7F) as u8,
            (sz & 0x7F) as u8,
        ]);
        t2.extend_from_slice(b"TALB");
        t2.extend_from_slice(&frame_size.to_be_bytes());
        t2.extend_from_slice(&[0, 0]); // frame flags
        t2.extend_from_slice(&frame_data);
        bytes.extend_from_slice(&t2);
        bytes.extend_from_slice(&audio);
        let path = root.join("stacked.mp3");
        std::fs::write(&path, &bytes).unwrap();

        // Sanity: lofty really does merge the two blocks into one view...
        let merged = lofty::read_from_path(&path).unwrap();
        assert_eq!(
            merged.primary_tag().unwrap().get_string(&ItemKey::AlbumTitle),
            Some("BEST of STEREOPONY")
        );

        // ...and a plain save of an edited tag loses the edit (pre-fix bug).
        {
            let mut tagged = lofty::read_from_path(&path).unwrap();
            let tag = tagged.primary_tag_mut().unwrap();
            tag.remove_key(&ItemKey::AlbumTitle);
            tag.insert_text(ItemKey::AlbumTitle, "Anison no Kokoro".into());
            tagged
                .save_to_path(&path, lofty::config::WriteOptions::default())
                .unwrap();
        }
        let after_plain = lofty::read_from_path(&path).unwrap();
        assert_eq!(
            after_plain.primary_tag().unwrap().get_string(&ItemKey::AlbumTitle),
            Some("BEST of STEREOPONY"),
            "demonstrates the silent no-op our repair exists for"
        );

        // The real path: scan + save_track_tags must land the edit.
        let conn = crate::library::db::open(&root.join("t.db")).expect("open");
        let mut conn = conn;
        crate::library::scan::run_scan(&mut conn, &root, |_, _| {}).expect("scan");
        let tid: String = conn
            .query_row("SELECT id FROM tracks LIMIT 1", [], |r| r.get(0))
            .unwrap();
        save_track_tags(
            &conn,
            &tid,
            &TrackTags {
                album: "Anison no Kokoro".into(),
                album_artist: "The Survivors".into(),
                ..Default::default()
            },
            &ArtChange::Keep,
        )
        .expect("save");

        let final_read = lofty::read_from_path(&path).unwrap();
        let tag = final_read.primary_tag().unwrap();
        assert_eq!(
            tag.get_string(&ItemKey::AlbumTitle),
            Some("Anison no Kokoro"),
            "edit must survive the stacked-tag collapse"
        );
        assert_eq!(tag.get_string(&ItemKey::AlbumArtist), Some("The Survivors"));
        assert_eq!(final_read.tags().len(), 1, "collapsed to a single ID3v2");
        let _ = std::fs::remove_dir_all(&root);
    }

    // ── Tag Editor Redesign, Phase A ──────────────────────────────────────

    fn png_solid(color: [u8; 3]) -> Vec<u8> {
        let buf = image::RgbaImage::from_pixel(8, 8, image::Rgba([color[0], color[1], color[2], 255]));
        let img = image::DynamicImage::ImageRgba8(buf);
        let mut out = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    fn b64(bytes: &[u8]) -> String {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    fn genre_of(path: &Path) -> String {
        read_file(path).unwrap().genre
    }

    fn pics_of(path: &Path) -> Vec<String> {
        read_file_full(path).unwrap().1
    }

    fn giants(root: &Path) -> (Connection, String, Vec<PathBuf>) {
        copy_tree(&fixtures_src(), root);
        let conn = scanned_db(root);
        let aid = album_id(&conn, "Giants & Monsters", "Helloween");
        let files = conn
            .prepare_cached("SELECT path FROM tracks WHERE album_id = ?1")
            .unwrap()
            .query_map([&aid], |r| r.get::<_, String>(0))
            .unwrap()
            .map(|r| PathBuf::from(r.unwrap()))
            .collect();
        (conn, aid, files)
    }

    #[test]
    fn write_file_survives_id3v1_multibyte_truncation() {
        // lofty 0.22.4's write_id3v1 cuts each legacy field with
        // split_at(BYTES), so a field whose cut lands inside a multi-byte
        // char PANICS the whole writer (owner report 2026-09-04, clearing
        // artwork on the Road To The Unknown compilation — Walla Walla's
        // v1 comment carried cp1251 cyrillic, the cut at 28 landed inside
        // a decoded char). write_file must still save the file.
        let root = temp_dir("id3v1panic");
        copy_tree(&fixtures_src(), &root);
        let file = root.join("Helloween/Giants & Monsters (2021)/02 - Throne of the Iron Vigil.mp3");
        // A hand-built legacy tag at EOF — the real-world shape, which
        // lofty's own writer cannot produce (writing it is the panic).
        let mut v1 = b"TAG".to_vec();
        let mut title = b"Throne of the Iron Vigil".to_vec();
        title.resize(30, 0);
        v1.extend(title);
        v1.extend([0u8; 60]); // artist + album blank
        v1.extend(b"2021");
        let mut comment = b"Collected by ".to_vec();
        comment.extend(std::iter::repeat(0xD0).take(17)); // 30 bytes,
        // first non-ASCII byte at 13 (odd) — the decoded comment's split
        // at 28 lands inside a char, exactly like the real Walla Walla.
        v1.extend(comment);
        v1.push(0xFF); // genre: none
        assert_eq!(v1.len(), 128);
        let mut bytes = std::fs::read(&file).unwrap();
        bytes.extend(v1);
        std::fs::write(&file, bytes).unwrap();

        // The exact edit from the incident: clear pictures, touch genre.
        super::write_file(&file, |tag| {
            while !tag.pictures().is_empty() {
                tag.remove_picture(0);
            }
            tag.insert_text(ItemKey::Genre, "Power Metal".into());
        })
        .expect("save must not die on the id3v1 truncation");

        let after = lofty::read_from_path(&file).unwrap();
        let v2 = after.tag(lofty::tag::TagType::Id3v2).expect("v2 written");
        assert_eq!(v2.get_string(&ItemKey::Genre).unwrap(), "Power Metal");
        // The legacy tag SURVIVES, its panic-cut field char-trimmed.
        let v1 = after.tag(lofty::tag::TagType::Id3v1).expect("v1 kept");
        let c = v1.get_string(&ItemKey::Comment).unwrap();
        // lofty's v1 READER decodes one char per byte, so compare chars:
        // the on-disk FIELD is the char count (must sit inside the 28).
        assert!(c.chars().count() <= 28, "comment trimmed to the field budget: {c:?}");
        assert!(c.starts_with("Collected by"), "head kept: {c:?}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn conflicts_report_values_with_counts() {
        let root = temp_dir("conflicts");
        let (conn, aid, files) = giants(&root);
        let before = get_album_tags(&conn, &aid).unwrap();
        assert!(before.genre.is_empty(), "fixtures carry no genre");
        let g0 = before.conflicts.iter().find(|c| c.field == "genre").expect("census");
        assert!(g0.values.is_empty() && g0.present == 0, "census covers quiet fields too");

        // Two files start disagreeing; the third stays silent. Absence is
        // not a contender — exactly two values must show, count 1 each.
        let mut tag_file = |p: &Path, g: &str| {
            let mut tagged = lofty::read_from_path(p).unwrap();
            tagged.primary_tag_mut().unwrap().set_genre(g.to_string());
            tagged.save_to_path(p, Default::default()).unwrap();
        };
        tag_file(&files[0], "Folk");
        tag_file(&files[1], "Metal");

        let at = get_album_tags(&conn, &aid).unwrap();
        let c = at.conflicts.iter().find(|c| c.field == "genre").expect("genre conflict");
        assert!(at.genre_disputed, "legacy flag still set");
        assert_eq!(c.values.len(), 2, "the silent file does not contend");
        assert_eq!(c.present, 2, "…but the census still knows it is silent");
        assert_eq!(c.values[0].value, "Folk", "first-seen order on equal counts");
        assert_eq!(c.values[0].count, 1);
        assert_eq!(c.values[1].value, "Metal");
        assert_eq!(c.values[1].count, 1);

        // A majority reads first:
        tag_file(&files[2], "Metal");
        let at = get_album_tags(&conn, &aid).unwrap();
        let c = at.conflicts.iter().find(|c| c.field == "genre").unwrap();
        assert_eq!(c.values[0].value, "Metal");
        assert_eq!(c.values[0].count, 2);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn album_save_writes_only_files_that_differ() {
        let root = temp_dir("wrideset");
        let (conn, aid, files) = giants(&root);
        let touched = TouchedFields { genre: true, ..Default::default() };

        // The fixtures have no genre: stamping one reaches every file…
        let shared = TrackTags { genre: "X".into(), ..Default::default() };
        let rep = save_album_tags(&conn, &aid, &shared, &touched, &ArtChange::Keep).unwrap();
        assert_eq!(rep.written, files.len(), "empty differs from X");
        for f in &files {
            assert_eq!(genre_of(f), "X");
        }

        // …saving the same value again writes NOTHING — no mtime bumps, no
        // re-parse, an honest zero the modal can print.
        let rep = save_album_tags(&conn, &aid, &shared, &touched, &ArtChange::Keep).unwrap();
        assert_eq!(rep.written, 0, "untouched-equal save is a no-op");
        assert_eq!(rep.total, files.len());

        // One file drifts; the next save is a one-file write set, and only
        // the touched field moved anywhere.
        let mut tagged = lofty::read_from_path(&files[1]).unwrap();
        tagged.primary_tag_mut().unwrap().set_genre("Folk".into());
        tagged.save_to_path(&files[1], Default::default()).unwrap();
        let title_before = read_file(&files[1]).unwrap().title;
        let rep = save_album_tags(&conn, &aid, &shared, &touched, &ArtChange::Keep).unwrap();
        assert_eq!(rep.written, 1, "the drift is healed, the two agreeing files untouched");
        assert_eq!(genre_of(&files[1]), "X");
        assert_eq!(read_file(&files[1]).unwrap().title, title_before, "untouched fields stay");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn relabel_then_targeted_rescan_regroups_rows() {
        // The Xandria bug (owner report 2026-09-05): a staged album lives
        // where no library root points, an incremental scan can never
        // revisit it, so after the tag editor renamed it the DB carried
        // the OLD title forever. The save command's follow-up —
        // run_scan_files with the written files' parents as roots and
        // only=the written files — must re-group the rows like any scan.
        let root = temp_dir("retarget");
        let (mut conn, aid, files) = giants(&root);
        let touched = TouchedFields { album: true, ..Default::default() };
        let shared = TrackTags {
            album: "Keeper of The Seven Keys".into(),
            ..Default::default()
        };
        let rep = save_album_tags(&conn, &aid, &shared, &touched, &ArtChange::Keep).unwrap();
        assert_eq!(rep.written, files.len(), "every file took the new album tag");

        let mut roots: Vec<PathBuf> =
            files.iter().filter_map(|f| f.parent().map(PathBuf::from)).collect();
        roots.sort();
        roots.dedup();
        let only: std::collections::HashSet<PathBuf> = files.iter().cloned().collect();
        crate::library::scan::run_scan_files(&mut conn, &roots, Some(&only), |_, _| {}, false)
            .expect("targeted rescan");

        assert_eq!(count_albums(&conn, "Keeper of The Seven Keys"), 1, "new album row");
        assert_eq!(count_albums(&conn, "Giants & Monsters"), 0, "old row died with its rows");
        let moved: i64 = conn
            .query_row(
                "SELECT count(*) FROM tracks t JOIN albums a ON t.album_id = a.id
                 WHERE a.title = 'Keeper of The Seven Keys'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(moved, files.len() as i64, "every row followed the tag");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn year_edit_updates_the_album_row() {
        // The grid/panel must stop printing a year the user cleared (the
        // scan never UPDATEs an existing album row — see save_album_tags).
        let root = temp_dir("year-row");
        let (conn, aid, _files) = giants(&root);
        let touched = TouchedFields { year: true, ..Default::default() };

        let shared = TrackTags { year: Some(1999), ..Default::default() };
        save_album_tags(&conn, &aid, &shared, &touched, &ArtChange::Keep).unwrap();
        let year: Option<i64> = conn
            .query_row("SELECT year FROM albums WHERE id = ?1", [&aid], |r| r.get(0))
            .unwrap();
        assert_eq!(year, Some(1999), "a year edit lands in the row immediately");

        let shared = TrackTags { year: None, ..Default::default() };
        save_album_tags(&conn, &aid, &shared, &touched, &ArtChange::Keep).unwrap();
        let year: Option<i64> = conn
            .query_row("SELECT year FROM albums WHERE id = ?1", [&aid], |r| r.get(0))
            .unwrap();
        assert_eq!(year, None, "clearing the field clears the row, not just the files");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn art_change_wire_shape_is_the_webview_contract() {
        // src/lib/artChange.ts IS the contract: exactly "keep" | "clear" |
        // {hash} | {upload:{image,mime}}. Any drift dies at Tauri's arg
        // parse as `invalid args \`art\`: invalid type: string` — which is
        // exactly how picking a cover art candidate on the ASMR
        // compilation failed 2026-09-05 (Hash had been a STRUCT variant,
        // demanding {"hash":{"hash":…}}). Test the WIRE, not the
        // constructor — the constructor never lies about a serde shape.
        let keep: ArtChange = serde_json::from_str("\"keep\"").unwrap();
        assert_eq!(keep, ArtChange::Keep);
        let clear: ArtChange = serde_json::from_str("\"clear\"").unwrap();
        assert_eq!(clear, ArtChange::Clear);
        let hash: ArtChange = serde_json::from_str("{\"hash\":\"2d4afec9\"}").unwrap();
        assert_eq!(hash, ArtChange::Hash("2d4afec9".into()));
        let up: ArtChange =
            serde_json::from_str("{\"upload\":{\"image\":\"aGk=\",\"mime\":\"image/jpeg\"}}")
                .unwrap();
        assert!(matches!(up, ArtChange::Upload { image, mime }
            if image == "aGk=" && mime == "image/jpeg"));
        assert_eq!(
            serde_json::to_string(&ArtChange::Hash("2d4afec9".into())).unwrap(),
            "{\"hash\":\"2d4afec9\"}",
            "round-trip must emit the exact tagged shape the webview sends"
        );
    }

    #[test]
    fn artwork_lifecycle_upload_hash_clear_and_folder_art() {
        let root = temp_dir("artwork");
        let (conn, aid, files) = giants(&root);
        let cache = root.join("cache");
        std::fs::create_dir_all(&cache).unwrap();
        let flac = files.iter().find(|f| f.extension().is_some_and(|e| e == "flac")).unwrap();
        let other = files.iter().find(|f| *f != flac).unwrap();
        let red = png_solid([255, 0, 0]);
        let blue = png_solid([0, 0, 255]);
        let red_hash = blake3::hash(&red).to_hex().to_string();
        let blue_hash = blake3::hash(&blue).to_hex().to_string();
        let untouched = TouchedFields::default();
        let aid_s = aid.clone();

        // Upload onto one track: the file ends with EXACTLY that picture.
        let tid: String = conn
            .query_row("SELECT id FROM tracks WHERE path = ?1", [flac.to_str().unwrap()], |r| r.get(0))
            .unwrap();
        save_track_tags(
            &conn, &tid,
            &get_track_tags(&conn, &tid).unwrap(),
            &ArtChange::Upload { image: b64(&red), mime: String::new() },
        ).unwrap();
        let pics = pics_of(flac);
        assert_eq!(pics, vec![red_hash.clone()], "one picture, uploaded bytes");

        // The blue one goes onto a sibling by upload too…
        let tid2: String = conn
            .query_row("SELECT id FROM tracks WHERE path = ?1", [other.to_str().unwrap()], |r| r.get(0))
            .unwrap();
        save_track_tags(
            &conn, &tid2,
            &get_track_tags(&conn, &tid2).unwrap(),
            &ArtChange::Upload { image: b64(&blue), mime: String::new() },
        ).unwrap();

        // …then the ALBUM adjudicates on the blue via its hash: the red file
        // is written, the already-blue one is not, and the fixture's
        // folder.jpg is replaced in place (JPEG bytes) with a receipt naming
        // it — folder art outranks embedded, so the editor must not lie.
        let rep = save_album_tags(
            &conn, &aid_s,
            &TrackTags::default(),
            &untouched,
            &ArtChange::Hash(blue_hash.clone()),
        ).unwrap();
        assert_eq!(rep.written, 2, "the red file and the picture-less one differed; the blue one did not");
        assert_eq!(pics_of(flac), vec![blue_hash.clone()]);
        assert_eq!(pics_of(other), vec![blue_hash.clone()]);
        assert!(rep.receipt.iter().any(|r| r.starts_with("replaced ") && r.contains("folder.jpg")), "{rep:?}");
        let folder_jpg = root.join("Helloween/Giants & Monsters (2021)/folder.jpg");
        let folder_bytes = std::fs::read(&folder_jpg).unwrap();
        let fmt = image::ImageReader::new(std::io::Cursor::new(&folder_bytes))
            .with_guessed_format()
            .unwrap()
            .format();
        assert!(matches!(fmt, Some(image::ImageFormat::Jpeg)), "replaced with real JPEG");
        let img = image::load_from_memory(&folder_bytes).unwrap();
        let px = *img.to_rgb8().get_pixel(2, 2);
        assert!(px.0[2] > 128 && px.0[0] < 64, "it is the blue one: {:?}", px.0);

        // Inventory: candidates with census, folder winner named, the folder
        // art CURRENT (pipeline precedence), previews written. The folder
        // tile is its OWN candidate — it holds re-encoded JPEG bytes, not
        // the PNG that was adjudicated (dedupe is by bytes, as specced).
        let inv = list_art_candidates(&conn, &aid_s, Some(&tid), &cache).unwrap();
        assert_eq!(inv.folder_art.as_deref(), Some("folder.jpg"));
        let folder_hash = blake3::hash(&folder_bytes).to_hex().to_string();
        assert_eq!(inv.current.as_deref(), Some(folder_hash.as_str()), "winner = replaced folder art");
        assert_eq!(inv.track_pics, vec![blue_hash.clone()], "the asked track's own picture");
        let blue_c = inv.candidates.iter().find(|c| c.hash == blue_hash).expect("blue candidate");
        assert_eq!(blue_c.count, 3, "all three files carry the PNG bytes");
        assert!(blue_c.label.starts_with("in 3 files"), "{}", blue_c.label);
        let folder_c = inv
            .candidates
            .iter()
            .find(|c| c.hash == folder_hash)
            .expect("the folder JPEG is its own tile");
        assert_eq!(folder_c.folder.as_deref(), Some("folder.jpg"));
        assert_eq!(folder_c.count, 0, "it lives in no file (yet)");
        assert!(
            !inv.candidates.iter().any(|c| c.hash == red_hash),
            "the adjudication REPLACED the red — the inventory is the album's current truth"
        );
        assert_eq!(inv.files_with_art, 3, "every file carries the blue now");
        assert_eq!(inv.candidates.len(), 2, "blue + the folder JPEG, nothing else");
        let h8 = &blue_hash[..8];
        assert!(crate::library::artwork::thumbs_dir(&cache).join(format!("art-{h8}")).join("256.webp").is_file(), "preview thumb");

        // Clear: embedded gone from every file that had one, folder art
        // deleted, receipt says so out loud…
        let rep = save_album_tags(&conn, &aid_s, &TrackTags::default(), &untouched, &ArtChange::Clear).unwrap();
        for f in &files {
            if f.extension().is_some_and(|e| e == "wav") {
                continue;
            }
            assert!(pics_of(f).is_empty(), "{} cleared", f.display());
        }
        assert!(rep.receipt.iter().any(|r| r.starts_with("removed ") && r.contains("folder.jpg")), "{rep:?}");

        // …and the targeted refresh turns that truth into a NULLed cover —
        // the placeholder returns honestly.
        crate::library::artwork::refresh_one(&conn, &cache, &aid_s).unwrap();
        let cover: Option<String> = conn
            .query_row("SELECT cover FROM albums WHERE id = ?1", [&aid_s], |r| r.get(0))
            .unwrap();
        let cover_gone = cover.is_none() || cover.as_deref() == Some("");
        assert!(cover_gone, "no source left: cover = {cover:?}");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn cut_leading_tags_cuts_headers_and_keeps_audio() {
        // The arithmetic of the repair, independent of any lofty view:
        // two adjacent ID3v2 blocks of DIFFERENT versions, then a byte
        // blob that must survive untouched. (v2.2 counts too: 3-byte
        // big-endian size, no flags byte.)
        let dir = temp_dir("cut");
        let f = dir.join("cutme.mp3");
        let ss = |n: usize| -> [u8; 4] {
            [
                ((n >> 21) & 0x7f) as u8,
                ((n >> 14) & 0x7f) as u8,
                ((n >> 7) & 0x7f) as u8,
                (n & 0x7f) as u8,
            ]
        };
        let mut bytes: Vec<u8> = b"ID3".to_vec();
        bytes.extend([3, 0, 0]);
        bytes.extend(ss(6));
        bytes.extend(b"ABCDEF");
        bytes.extend(b"ID3");
        bytes.extend([4, 0, 0]);
        bytes.extend(ss(4));
        bytes.extend(b"GGGG");
        let audio: Vec<u8> = vec![0xff, 0xfb, b'1', b'2', b'3'];
        bytes.extend(&audio);
        std::fs::write(&f, &bytes).unwrap();

        assert_eq!(cut_leading_tags(&f).expect("cut"), 10 + 6 + 10 + 4);
        assert_eq!(std::fs::read(&f).unwrap(), audio);

        // A file that does not start with ID3 is left byte-identical.
        let g = dir.join("plain.mp3");
        std::fs::write(&g, &audio).unwrap();
        assert_eq!(cut_leading_tags(&g).expect("no-op cut"), 0);
        assert_eq!(std::fs::read(&g).unwrap(), audio);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn repair_stacked_tags_leaves_one_tag_with_expected() {
        // The repair's contract, run directly on a stacked file: every
        // leading block gone, `expected` the single winner. write_file's
        // probe-blindness branch ("No format could be determined" — the
        // owner's "19 Flash.mp3", 2026-09-05: a fat ID3v2.3 whose
        // DECLARED end is a second ID3v2.4, not a frame) reaches exactly
        // this function. The exact probe refusal itself is a lofty
        // property measured on a /tmp copy of the incident file; several
        // synthetic layouts were probed and none re-summoned it, so the
        // mechanism is pinned here instead.
        let dir = temp_dir("repair-direct");
        let f = dir.join("stack.mp3");
        let mut bytes: Vec<u8> = b"ID3".to_vec();
        bytes.extend([3, 0, 0, 0, 0, 0, 6]);
        bytes.extend(b"ABCDEF");
        bytes.extend(b"ID3");
        bytes.extend([4, 0, 0, 0, 0, 0, 4]);
        bytes.extend(b"GGGG");
        let fx = walkdir::WalkDir::new(fixtures_src())
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("mp3"))
            .expect("a fixture mp3")
            .path()
            .to_path_buf();
        bytes.extend(std::fs::read(&fx).unwrap());
        std::fs::write(&f, &bytes).unwrap();

        let mut want = Tag::new(lofty::tag::TagType::Id3v2);
        want.set_title("Repaired".to_string());
        repair_stacked_tags(&f, &want).expect("repair");
        let re = lofty::read_from_path(&f).expect("readable after repair");
        assert_eq!(
            re.primary_tag()
                .or_else(|| re.first_tag())
                .and_then(|t| t.get_string(&ItemKey::TrackTitle))
                .map(str::to_string)
                .as_deref(),
            Some("Repaired")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
