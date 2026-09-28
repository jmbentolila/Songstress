//! Per-track waveform peaks for the playbar's waveform progress bar.
//!
//! Decoded with pure-Rust `symphonia`, so the RPM keeps its
//! `depends = ["mpv"]` promise — no new SYSTEM package. Decoding is CPU work
//! and happens OUTSIDE the DB lock (see the `track_peaks` command in lib.rs).

use std::path::{Path, PathBuf};

use rayon::prelude::*;
use rusqlite::Connection;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Peaks per track. 256 bytes per track in the DB — enough resolution for the
/// ~430px lane; the UI resamples to its own bar count (`resample` in the
/// frontend's waveform.ts) so the stored count never has to match the design.
pub const BUCKETS: usize = 256;

/// Meaning of the stored `track_peaks.data` values. Revision 1 was peak-only;
/// revision 2 stores the geometric blend of a bucket's peak with its RMS. Bump
/// this constant whenever decoding changes what the bytes mean —
/// `ensure_algo_version` then deletes rows written by the older shape, so an
/// unchanged `(track_id, mtime_ns)` never falsely implies an unchanged metric.
pub const PEAKS_ALGO_VERSION: u32 = 2;

/// Blend one bucket's peak and RMS into an amplitude that preserves a transient
/// while following the bucket's energy. A sinusoid and a heavily compressed
/// master both have saturated peaks; RMS still varies across sections, while
/// pure RMS flattens deliberate attacks. Invalid inputs can come from a corrupt
/// packet, so this never returns NaN or infinity — debug builds panic inside
/// Tauri commands, and user-owned files always deserve arithmetic that degrades
/// instead of aborting.
fn blend_bucket_level(peak: f32, square_sum: f64, frames: u64) -> f32 {
    if frames == 0 {
        return 0.0;
    }
    let peak = if peak.is_finite() { peak.max(0.0) } else { 0.0 };
    let mean_square = square_sum / frames.max(1) as f64;
    if !mean_square.is_finite() || mean_square <= 0.0 {
        return 0.0;
    }
    let rms = mean_square.sqrt();
    if !rms.is_finite() || rms <= 0.0 {
        return 0.0;
    }
    let level = (f64::from(peak) * rms).sqrt();
    if !level.is_finite() || level <= 0.0 {
        return 0.0;
    }
    level.min(f32::MAX as f64) as f32
}

/// Decode `path` and reduce it to `BUCKETS` mono amplitudes (0..=255): the
/// geometric blend of each bucket's peak with its RMS, normalized so the
/// loudest bucket is 255 — each track then fills the lane.
/// `duration_sec` sizes the buckets when the container reports no frame count.
pub fn compute(path: &Path, duration_sec: f64) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| e.to_string())?;
    let mut format = probed.format;

    let track = format
        .default_track()
        .ok_or_else(|| "no audio track".to_string())?;
    let track_id = track.id;
    let frames_hint = track.codec_params.n_frames.unwrap_or(0);
    let codec_params = track.codec_params.clone();

    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;

    let mut peaks = vec![0f32; BUCKETS];
    let mut energy = vec![0f64; BUCKETS];
    let mut bucket_frames = vec![0u64; BUCKETS];
    let mut total_frames: u64 = 0;
    // `frame_index` is reused as the CURRENT BUCKET while accumulating.
    let mut frame_index: u64 = 0;
    let mut bucket_left: u64 = 0;
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            // A clean end-of-stream is not an error; neither is a stream that
            // asked us to reset (we have no better track to fall back to).
            Err(SymError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(SymError::ResetRequired) => break,
            Err(e) => return Err(e.to_string()),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            // One bad frame must not fail the whole track.
            Err(SymError::DecodeError(_)) => continue,
            Err(e) => return Err(e.to_string()),
        };

        let spec = *decoded.spec();
        if total_frames == 0 {
            total_frames = if frames_hint > 0 {
                frames_hint
            } else {
                (duration_sec * spec.rate as f64).round() as u64
            };
            if total_frames == 0 {
                total_frames = 1;
            }
        }

        let channels = spec.channels.count().max(1);
        let buf = sample_buf
            .get_or_insert_with(|| SampleBuffer::<f32>::new(decoded.capacity() as u64, spec));
        buf.copy_interleaved_ref(decoded);

        // Bucket by a RUNNING counter instead of a per-frame multiply+divide:
        // 26M samples on a long track make that arithmetic the whole cost in an
        // opt-level-0 dev build (measured 1-3s/track vs ~150ms in release).
        let per_bucket = (total_frames / BUCKETS as u64).max(1);
        for frame in buf.samples().chunks(channels) {
            // Ordinary mono downmix: incoming frames remain signed so opposite
            // phases can cancel, the way playback does. Guard only finite
            // arithmetic rather than assuming decoder samples are clean.
            let mut mono = 0f32;
            for &s in frame {
                mono += if s.is_finite() { s } else { 0.0 };
            }
            mono /= frame.len().max(1) as f32;
            if !mono.is_finite() {
                mono = 0.0;
            }
            let bucket = frame_index as usize;
            let magnitude = mono.abs();
            if magnitude > peaks[bucket] {
                peaks[bucket] = magnitude;
            }
            energy[bucket] += f64::from(mono) * f64::from(mono);
            bucket_frames[bucket] += 1;
            bucket_left += 1;
            if bucket_left >= per_bucket && frame_index + 1 < BUCKETS as u64 {
                frame_index += 1;
                bucket_left = 0;
            }
        }
    }

    let mut levels = Vec::with_capacity(BUCKETS);
    for i in 0..BUCKETS {
        levels.push(blend_bucket_level(peaks[i], energy[i], bucket_frames[i]));
    }
    let max = levels.iter().copied().fold(0f32, f32::max);
    if !(max > 0.0) {
        return Ok(vec![0u8; BUCKETS]);
    }
    Ok(levels
        .iter()
        .map(|level| ((level / max) * 255.0).round().clamp(0.0, 255.0) as u8)
        .collect())
}

/// The metric revision recorded in the `peaksAlgo` setting, so stored rows
/// from an older metric revision do not masquerade as current values.
pub const PEAKS_ALGO_SETTING: &str = "peaksAlgo";

/// Delete all cached waveform values when `PEAKS_ALGO_VERSION` has moved on,
/// then record the current revision. Returns how many stale rows were deleted.
///
/// This runs at app setup, while the DB connection is still owned by setup
/// and before any command can read the cache: user files and real peak rows
/// are never touched, only their derived display cache. If the setting row is
/// missing or unreadable but there are no peak rows anyway, it still records
/// the current revision so the check is one row read on later startups.
pub fn ensure_algo_version(conn: &Connection) -> rusqlite::Result<usize> {
    let recorded: Option<u32> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [PEAKS_ALGO_SETTING],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .and_then(|value| serde_json::from_str(&value).ok());
    if recorded == Some(PEAKS_ALGO_VERSION) {
        return Ok(0);
    }
    let cleared = conn.execute("DELETE FROM track_peaks", [])? as usize;
    crate::library::settings::set(conn, PEAKS_ALGO_SETTING, &PEAKS_ALGO_VERSION.to_string())?;
    Ok(cleared)
}

/// The cached peaks for a track, if the file is unchanged since they were
/// computed (keyed on mtime — a re-encode invalidates them). Startup also
/// rotates out rows whose bytes predate `PEAKS_ALGO_VERSION`.
pub fn cached(conn: &Connection, track_id: &str, mtime_ns: i64) -> Option<Vec<u8>> {
    conn.query_row(
        "SELECT data FROM track_peaks WHERE track_id = ?1 AND mtime_ns = ?2",
        rusqlite::params![track_id, mtime_ns],
        |row| row.get::<_, Vec<u8>>(0),
    )
    .ok()
}

/// Persist peaks for a track (upsert: a re-decode replaces the row).
pub fn store(conn: &Connection, track_id: &str, mtime_ns: i64, data: &[u8]) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO track_peaks(track_id, mtime_ns, buckets, data) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(track_id) DO UPDATE SET
             mtime_ns = excluded.mtime_ns,
             buckets  = excluded.buckets,
             data     = excluded.data",
        rusqlite::params![track_id, mtime_ns, BUCKETS as i64, data],
    )?;
    Ok(())
}

/// One track whose peaks still need computing.
pub struct Pending {
    pub id: String,
    pub path: String,
    pub mtime_ns: i64,
    pub duration_sec: f64,
}

/// Every track with no cached peaks row for its current mtime.
pub fn pending_all(conn: &Connection) -> rusqlite::Result<Vec<Pending>> {
    let mut st = conn.prepare(
        "SELECT t.id, t.path, t.mtime_ns, t.duration_sec
           FROM tracks t
           LEFT JOIN track_peaks p ON p.track_id = t.id AND p.mtime_ns = t.mtime_ns
          WHERE p.track_id IS NULL",
    )?;
    let rows = st
        .query_map([], |r| {
            Ok(Pending {
                id: r.get(0)?,
                path: r.get(1)?,
                mtime_ns: r.get(2)?,
                duration_sec: r.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

/// The rows for a specific set of files (an import), cached or not — the import
/// path has just indexed these and wants their peaks immediately.
pub fn pending_paths(
    conn: &Connection,
    paths: &[PathBuf],
) -> rusqlite::Result<Vec<Pending>> {
    let mut st =
        conn.prepare("SELECT id, path, mtime_ns, duration_sec FROM tracks WHERE path = ?1")?;
    let mut out = Vec::new();
    for p in paths {
        let s = p.to_string_lossy();
        if let Ok(row) = st.query_row([s.as_ref()], |r| {
            Ok(Pending {
                id: r.get(0)?,
                path: r.get(1)?,
                mtime_ns: r.get(2)?,
                duration_sec: r.get(3)?,
            })
        }) {
            out.push(row);
        }
    }
    Ok(out)
}

/// Decode + store peaks for `rows` in PARALLEL, writing on THIS thread (the DB
/// connection is not shareable). One core is left free so playback and the UI
/// keep theirs. `progress(done, total)` fires once per chunk. Returns the number
/// stored; a track that fails to decode is skipped, not fatal.
pub fn backfill(
    conn: &Connection,
    rows: Vec<Pending>,
    progress: &mut dyn FnMut(usize, usize),
) -> usize {
    // Small chunks bound memory AND persist incrementally, so an interrupted
    // backfill resumes from where it stopped instead of starting over.
    const CHUNK: usize = 64;
    let total = rows.len();
    progress(0, total);
    if total == 0 {
        return 0;
    }
    // Leave room for playback and the UI: on a BIG box (more than 10 cores) use
    // all but one; on a smaller one only HALF, so a modest machine is not
    // saturated by the decode (16 → 15, 8 → 4, 4 → 2, 2 → 1).
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let threads = (if cores > 10 { cores - 1 } else { cores / 2 }).max(1);
    // Run the decode threads BELOW the UI and mpv (nice +10). A full-tilt
    // backfill was starving the compositor — the playbar toggle dropped to
    // ~40fps with 60-84ms spikes while it ran. Low priority means the UI always
    // preempts the decode, so the app stays smooth even mid-backfill.
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .spawn_handler(|thread| {
            let mut b = std::thread::Builder::new();
            if let Some(name) = thread.name() {
                b = b.name(name.to_owned());
            }
            if let Some(size) = thread.stack_size() {
                b = b.stack_size(size);
            }
            b.spawn(move || {
                unsafe {
                    libc::setpriority(libc::PRIO_PROCESS, 0, 10);
                }
                thread.run();
            })?;
            Ok(())
        })
        .build();
    let mut done = 0usize;
    for chunk in rows.chunks(CHUNK) {
        let decode = |p: &Pending| {
            compute(Path::new(&p.path), p.duration_sec)
                .ok()
                .map(|d| (p.id.clone(), p.mtime_ns, d))
        };
        let decoded: Vec<(String, i64, Vec<u8>)> = match &pool {
            Ok(pool) => pool.install(|| chunk.par_iter().filter_map(decode).collect()),
            Err(_) => chunk.iter().filter_map(decode).collect(),
        };
        for (id, mtime, data) in decoded {
            if store(conn, &id, mtime, &data).is_ok() {
                done += 1;
            }
        }
        progress(done, total);
    }
    done
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blend_bucket_level_weights_peak_and_energy() {
        // RMS 1: a full transient stays full.
        assert!((blend_bucket_level(1.0, 4.0, 4) - 1.0).abs() < 1e-6);
        // Same peak, sparse energy: the blend stays well below full.
        assert!((blend_bucket_level(1.0, 1.0, 16) - 0.5).abs() < 1e-6);
        // Same visible level from a lower peak with denser energy.
        assert!((blend_bucket_level(0.5, 4.0, 16) - 0.5).abs() < 1e-6);
        // Silent or corrupt buckets degrade to zero rather than NaN.
        assert_eq!(blend_bucket_level(0.0, 16.0, 16), 0.0);
        assert_eq!(blend_bucket_level(1.0, f64::NAN, 16), 0.0);
        assert_eq!(blend_bucket_level(f32::NAN, 16.0, 16), 0.0);
        assert_eq!(blend_bucket_level(1.0, 1.0, 0), 0.0);
    }

    #[test]
    fn stale_metric_rows_are_expired_before_commands_read_them() -> rusqlite::Result<()> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("ss-peaks-algo-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("lib.db");
        let _ = std::fs::remove_file(&db);
        let conn = crate::library::db::open(&db).unwrap();
        conn.execute(
            "INSERT INTO artists(id,name,sort_name) VALUES('ar2','A2','a2')",
            [],
        )?;
        conn.execute(
            "INSERT INTO albums(id,artist_id,title,year) VALUES('al2','ar2','Al2',2020)",
            [],
        )?;
        conn.execute(
            "INSERT INTO tracks(id,album_id,disc,track,title,duration_sec,path,mtime_ns,size)
             VALUES('tr2','al2',1,1,'T2',1.0,'fixtures/library/sample/untitled-song.wav',124,456)",
            [],
        )?;
        conn.execute(
            "INSERT INTO track_peaks(track_id,mtime_ns,buckets,data)
             VALUES('tr2',124,256,X'ff')",
            [],
        )?;
        crate::library::settings::set(&conn, PEAKS_ALGO_SETTING, "1")?;
        assert_eq!(ensure_algo_version(&conn)?, 1);
        let recorded: String = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [PEAKS_ALGO_SETTING],
            |row| row.get(0),
        )?;
        assert_eq!(
            serde_json::from_str::<u32>(&recorded).unwrap(),
            PEAKS_ALGO_VERSION
        );
        let remaining: i64 =
            conn.query_row("SELECT COUNT(*) FROM track_peaks", [], |row| row.get(0))?;
        assert_eq!(remaining, 0);
        // Deleting them puts the track back in the pending set even though its
        // mtime did not change.
        assert_eq!(pending_all(&conn)?.len(), 1);
        assert_eq!(ensure_algo_version(&conn)?, 0);
        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }

    fn fixture(rel: &str) -> std::path::PathBuf {
        std::path::Path::new("fixtures/library").join(rel)
    }

    /// Every format the library actually holds decodes end-to-end without a
    /// panic and yields exactly one bucket per bucket. The fixtures are SILENT
    /// (generated for tag tests), so their peaks are legitimately all zero —
    /// the normalization path is covered by the synthetic tone below. (Opus is
    /// in the fixtures too but symphonia 0.5 has no Opus decoder; that track
    /// simply keeps its quiet floor line, which is the intended degrade.)
    #[test]
    fn decodes_the_common_formats() {
        let cases = [
            "Helloween/Giants & Monsters (2021)/02 - Throne of the Iron Vigil.mp3",
            "Helloween/Giants & Monsters (2021)/01 - Silent Echoes.flac",
            "The Birthday Massacre/Walking With Strangers (2007)/01 - Looking Glass.ogg",
            "Helloween/Giants & Monsters (2021)/Disc 2/01 - Echoes of the Hollow Prophecy.m4a",
            "sample/untitled-song.wav",
            "sample/untitled-song.aiff",
        ];
        for rel in cases {
            let p = fixture(rel);
            if !p.exists() {
                continue;
            }
            let peaks = compute(&p, 1.0).unwrap_or_else(|e| panic!("{rel}: {e}"));
            assert_eq!(peaks.len(), BUCKETS, "{rel}");
        }
    }

    #[test]
    fn a_missing_file_is_an_error_not_a_panic() {
        assert!(compute(std::path::Path::new("fixtures/nope.mp3"), 1.0).is_err());
    }

    /// A mono 16-bit PCM WAV whose amplitude ramps 0.1 → 1.0 across the file,
    /// so quiet buckets must stay low and the blend's loudest bucket must
    /// normalize to 255.
    fn write_ramp_wav(path: &std::path::Path, sample_rate: u32, secs: f64) {
        let n = (sample_rate as f64 * secs) as usize;
        let mut pcm = Vec::with_capacity(n * 2);
        for i in 0..n {
            let t = i as f64 / sample_rate as f64;
            let amp = 0.1 + 0.9 * (i as f64 / n as f64);
            let s = (amp * (2.0 * std::f64::consts::PI * 440.0 * t).sin() * i16::MAX as f64) as i16;
            pcm.extend_from_slice(&s.to_le_bytes());
        }
        let mut w = Vec::new();
        w.extend_from_slice(b"RIFF");
        w.extend_from_slice(&((36 + pcm.len()) as u32).to_le_bytes());
        w.extend_from_slice(b"WAVEfmt ");
        w.extend_from_slice(&16u32.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());
        w.extend_from_slice(&1u16.to_le_bytes());
        w.extend_from_slice(&sample_rate.to_le_bytes());
        w.extend_from_slice(&(sample_rate * 2).to_le_bytes());
        w.extend_from_slice(&2u16.to_le_bytes());
        w.extend_from_slice(&16u16.to_le_bytes());
        w.extend_from_slice(b"data");
        w.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
        w.extend_from_slice(&pcm);
        std::fs::write(path, w).unwrap();
    }

    #[test]
    fn backfill_stores_pending_then_none_remain() {
        let dir = std::env::temp_dir().join(format!("ss-peaks-db-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db = dir.join("lib.db");
        let _ = std::fs::remove_file(&db);
        let conn = crate::library::db::open(&db).unwrap();
        conn.execute(
            "INSERT INTO artists(id,name,sort_name) VALUES('ar1','A','a')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO albums(id,artist_id,title,year) VALUES('al1','ar1','Al',2020)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO tracks(id,album_id,disc,track,title,duration_sec,path,mtime_ns,size)
             VALUES('tr1','al1',1,1,'T',1.0,'fixtures/library/sample/untitled-song.wav',123,456)",
            [],
        )
        .unwrap();

        let rows = pending_all(&conn).unwrap();
        assert_eq!(rows.len(), 1, "the one track lacks peaks");
        let n = backfill(&conn, rows, &mut |_done, _total| {});
        assert_eq!(n, 1);
        assert!(pending_all(&conn).unwrap().is_empty(), "cached now");
        let bytes: Vec<u8> = conn
            .query_row("SELECT data FROM track_peaks WHERE track_id='tr1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(bytes.len(), BUCKETS);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_ramped_tone_normalizes_and_keeps_its_shape() {
        // A scratch file in /tmp — never a path under the owner's library.
        let p = std::env::temp_dir().join("songstress-peaks-ramp.wav");
        write_ramp_wav(&p, 8000, 0.5);
        let peaks = compute(&p, 0.5).unwrap();
        let _ = std::fs::remove_file(&p);
        assert_eq!(peaks.len(), BUCKETS);
        assert_eq!(
            peaks.iter().copied().max().unwrap(),
            255,
            "loudest bucket fills the lane"
        );
        assert!(peaks[0] < 128, "quiet start stays low: {}", peaks[0]);
        assert!(
            peaks[BUCKETS - 1] >= 200,
            "loud end reads high: {}",
            peaks[BUCKETS - 1]
        );
    }
}