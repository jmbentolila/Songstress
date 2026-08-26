//! Tag editor backend (PLAN.md Step 1): read/write file tags via lofty.
//!
//! Reads always hit the FILES (not the DB) — the editor shows what a rescan
//! would see. Saves rewrite tags and let the edited mtimes drive the existing
//! incremental scan through the real grouping path; there is deliberately NO
//! parallel DB-update code here.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use lofty::file::{AudioFile, TaggedFileExt};
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

/// Album-level view: consensus values ("first non-empty wins") with
/// disagreement flags. Per-track fields are edited track-by-track only.
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
    let tagged = lofty::read_from_path(path).map_err(|e| format!("{path:?}: {e}"))?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    Ok(TrackTags {
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
    })
}

pub fn get_track_tags(conn: &Connection, track_id: &str) -> Result<TrackTags, String> {
    let path: String = conn
        .query_row("SELECT path FROM tracks WHERE id = ?1", [track_id], |r| r.get(0))
        .map_err(|_| "unknown track".to_string())?;
    read_file(Path::new(&path))
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

struct RowPaths(Vec<(String, PathBuf)>); // (track_id, path)

fn album_rows(conn: &Connection, album_id: &str) -> Result<RowPaths, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, path FROM tracks WHERE album_id = ?1
             ORDER BY disc, (track IS NOT NULL), track, title",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([album_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for row in rows {
        let (id, p) = row.map_err(|e| e.to_string())?;
        out.push((id, PathBuf::from(p)));
    }
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
fn apply_shared(tag: &mut Tag, t: &TrackTags) {
    set_text(tag, ItemKey::AlbumArtist, &t.album_artist);
    set_text(tag, ItemKey::AlbumTitle, &t.album);
    set_text(tag, ItemKey::Genre, &t.genre);
    set_text(tag, ItemKey::Composer, &t.composer);
    set_text(tag, ItemKey::Label, &t.label);
    set_text(tag, ItemKey::Comment, &t.comment);
    set_text(tag, ItemKey::ContentGroup, &t.grouping);
    // NOTE: the year() accessor falls back to RecordingDate (and set_year
    // rewrites an existing RecordingDate), so clearing must purge BOTH or
    // the old value keeps reading back.
    if let Some(y) = t.year {
        tag.set_year(u32::try_from(y.max(0)).unwrap_or(u32::MAX));
    } else {
        tag.remove_key(&ItemKey::Year);
        tag.remove_key(&ItemKey::RecordingDate);
    }
    tag.remove_key(&ItemKey::TrackTotal);
    if let Some(n) = t.track_total {
        tag.set_track_total(n);
    }
    tag.remove_key(&ItemKey::DiscTotal);
    if let Some(n) = t.disc_total {
        tag.set_disk_total(n);
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
    tagged
        .save_to_path(path, lofty::config::WriteOptions::default())
        .map_err(|e| format!("{path:?}: {e}"))?;

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

/// Strip every on-disk tag of `expected`'s type, then write `expected` back
/// as the single tag. Lossless for stacked-ID3v2 files: the merged view the
/// reader produced already contains the union of all stacked blocks.
fn repair_stacked_tags(path: &Path, expected: &Tag) -> Result<(), String> {
    let tt = expected.tag_type();
    for attempt in 1..=16 {
        let present = lofty::read_from_path(path)
            .map_err(|e| format!("{path:?}: {e}"))?
            .contains_tag_type(tt);
        if !present {
            break;
        }
        tt.remove_from_path(path)
            .map_err(|e| format!("{path:?}: strip {attempt}: {e}"))?;
    }
    let mut tagged = lofty::read_from_path(path).map_err(|e| format!("{path:?}: {e}"))?;
    tagged.insert_tag(expected.clone());
    tagged
        .save_to_path(path, lofty::config::WriteOptions::default())
        .map_err(|e| format!("{path:?}: rewrite: {e}"))?;

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

pub fn save_track_tags(conn: &Connection, track_id: &str, tags: &TrackTags) -> Result<(), String> {
    let path: String = conn
        .query_row("SELECT path FROM tracks WHERE id = ?1", [track_id], |r| r.get(0))
        .map_err(|_| "unknown track".to_string())?;
    let tags = tags.clone();
    write_file(Path::new(&path), |tag| {
        apply_shared(tag, &tags);
        apply_per_track(
            tag,
            &tags.title,
            Some(&tags.artist),
            tags.track_no,
            tags.disc_no,
        );
    })
}

/// Apply the shared album-level fields to every file in the album.
/// Per-track fields (title/artist/number/disc) are NEVER touched here —
/// they are edited track-by-track. Every file is attempted even if some
/// fail; returns all errors joined.
pub fn save_album_tags(
    conn: &Connection,
    album_id: &str,
    shared: &TrackTags,
) -> Result<usize, String> {
    let RowPaths(rows) = album_rows(conn, album_id)?;
    let shared = shared.clone();

    let mut saved = 0usize;
    let mut errors: Vec<String> = Vec::new();
    for (_, path) in &rows {
        let result = write_file(path, |tag| apply_shared(tag, &shared));
        match result {
            Ok(()) => saved += 1,
            Err(e) => errors.push(e),
        }
    }
    if errors.is_empty() {
        Ok(saved)
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
        save_track_tags(&conn, &tid, &full).expect("save");

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
        save_track_tags(&conn, &tid, &cleared).expect("clear-save");
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
        save_track_tags(&conn, &tid, &tags).expect("save onto tagless file");
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
            save_album_tags(&conn, &aid, &shared).expect("save album half");
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
}
