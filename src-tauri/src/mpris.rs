//! MPRIS D-Bus control (Phase 4).
//!
//! Serves `org.mpris.MediaPlayer2` + `.Player` on the session bus as
//! `org.mpris.MediaPlayer2.songstress`, backed by the same engine state the
//! frontend mirrors. Rust owns `{albumId, trackIndex}` (Phase 3), so external
//! controllers (playerctl, KDE media widget) and the UI stay in lockstep via
//! the shared `PlayState`.
//!
//! Property changes are announced by a lightweight watcher task that polls
//! the PlayState (~5 Hz) and emits PropertiesChanged only on diffs — avoids
//! threading notification hooks through every mpv.rs mutation. Position is
//! deliberately NOT pushed via PropertiesChanged (spec says clients poll it /
//! listen to Seeked); Seeked fires from explicit seeks.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use zbus::object_server::InterfaceRef;
use zbus::zvariant::{ObjectPath, OwnedValue, Value};

use crate::library;
use crate::mpv;

/// Handle for emitting Seeked from mpv.rs (registered once the server is up).
static PLAYER_REF: OnceLock<InterfaceRef<Player>> = OnceLock::new();

pub struct Root {
    app: tauri::AppHandle,
}

#[zbus::interface(name = "org.mpris.MediaPlayer2")]
impl Root {
    #[zbus(property)]
    fn can_quit(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_raise(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn has_track_list(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn identity(&self) -> String {
        "Songstress".into()
    }

    // No .desktop file yet (deferred); the name is still what clients expect.
    #[zbus(property)]
    fn desktop_entry(&self) -> String {
        "songstress".into()
    }

    #[zbus(property)]
    fn supported_uri_schemes(&self) -> Vec<String> {
        Vec::new()
    }

    #[zbus(property)]
    fn supported_mime_types(&self) -> Vec<String> {
        Vec::new()
    }

    fn raise(&self) {
        use tauri::Manager;
        if let Some(win) = self.app.get_webview_window("main") {
            let _ = win.set_focus();
        }
    }

    /// RunEvent::Exit hook reaps mpv, so a clean exit is enough.
    fn quit(&self) {
        self.app.exit(0);
    }
}

pub struct Player {
    engine: Arc<mpv::Mpv>,
    db_path: PathBuf,
    thumbs_dir: PathBuf,
}

impl Player {
    fn album_present(&self) -> bool {
        self.engine.state.lock().unwrap().current().is_some()
    }

    /// Value → OwnedValue (infallible in practice; only fd values can fail).
    fn owned(v: Value<'static>) -> OwnedValue {
        OwnedValue::try_from(v).unwrap_or(0u8.into())
    }

    /// The MPRIS trackid for a track id (object-path safe form).
    fn track_path(track_id: &str) -> String {
        let safe: String = track_id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
            .collect();
        format!("/org/songstress/track/{safe}")
    }

    /// Metadata map for the current track, straight from the library DB.
    /// Own connection: microseconds-scale lookup, never blocks the shared
    /// AppState mutex.
    fn metadata_map(&self) -> HashMap<String, OwnedValue> {
        let mut out: HashMap<String, OwnedValue> = HashMap::new();
        let (track_id, path) = {
            let st = self.engine.state.lock().unwrap();
            match st.current() {
                Some(item) => (Some(item.track_id.clone()), Some(item.path.clone())),
                None => (None, None),
            }
        };

        if let Some(id) = &track_id {
            // Object path elements only allow [A-Za-z0-9_]; blake3 ids carry
            // hyphens, so fold everything else to '_'.
            let track_path =
                ObjectPath::try_from(Self::track_path(id)).map(Value::from).ok();
            if let Some(v) = track_path {
                out.insert("mpris:trackid".into(), Self::owned(v));
            }
        }
        if let Some(p) = path {
            out.insert(
                "xesam:url".into(),
                Self::owned(Value::from(format!("file://{p}"))),
            );
        }

        let Some(tid) = track_id else { return out };
        let Ok(conn) = library::db::open(&self.db_path) else {
            return out;
        };
        let row = conn
            .query_row(
                "SELECT t.title, t.duration_sec, al.title, al.year, ar.name, al.cover
                 FROM tracks t
                 JOIN albums al ON al.id = t.album_id
                 JOIN artists ar ON ar.id = al.artist_id
                 WHERE t.id = ?1",
                [&tid],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, f64>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, Option<i64>>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, Option<String>>(5)?,
                    ))
                },
            )
            .ok();

        if let Some((title, dur, album, year, artist, cover)) = row {
            out.insert("xesam:title".into(), Self::owned(Value::from(title)));
            out.insert(
                "mpris:length".into(),
                Self::owned(Value::from((dur * 1_000_000.0) as i64)),
            );
            out.insert("xesam:album".into(), Self::owned(Value::from(album)));
            out.insert(
                "xesam:artist".into(),
                Self::owned(Value::from(vec![artist.clone()])),
            );
            out.insert("xesam:albumArtist".into(), Self::owned(Value::from(vec![artist])));
            if let Some(y) = year {
                out.insert(
                    "xesam:contentCreated".into(),
                    Self::owned(Value::from(y.to_string())),
                );
            }
            // albums.cover stores thumb://<id>/512.webp; resolve to the real
            // cache file so external clients can actually load it.
            if let Some(c) = cover {
                let file_id = c
                    .strip_prefix("thumb://")
                    .and_then(|rest| rest.split('/').next())
                    .map(str::to_string);
                if let Some(id) = file_id {
                    let p = library::artwork::thumb_path(&self.thumbs_dir, &id, 512);
                    if p.exists() {
                        if let Some(s) = p.to_str() {
                            out.insert(
                                "mpris:artUrl".into(),
                                Self::owned(Value::from(format!("file://{s}"))),
                            );
                        }
                    }
                }
            }
        }
        out
    }
}

#[zbus::interface(name = "org.mpris.MediaPlayer2.Player")]
impl Player {
    #[zbus(property)]
    fn playback_status(&self) -> String {
        let st = self.engine.state.lock().unwrap();
        if st.current().is_none() {
            "Stopped"
        } else if st.paused {
            "Paused"
        } else {
            "Playing"
        }
        .to_string()
    }

    #[zbus(property)]
    fn loop_status(&self) -> String {
        match self.engine.state.lock().unwrap().repeat {
            mpv::RepeatStage::Off => "None".to_string(),
            mpv::RepeatStage::Album => "Playlist".to_string(),
            mpv::RepeatStage::Track => "Track".to_string(),
        }
    }

    /// MPRIS clients setting LoopStatus map onto our repeat stages.
    #[zbus(property)]
    async fn set_loop_status(&self, status: String) {
        let stage = match status.as_str() {
            "Track" => mpv::RepeatStage::Track,
            "Playlist" => mpv::RepeatStage::Album,
            _ => mpv::RepeatStage::Off,
        };
        let engine = self.engine.clone();
        let db_path = self.db_path.clone();
        tauri::async_runtime::spawn(async move {
            let _ = engine.set_repeat(stage).await;
            persist_stage(&db_path, "repeatStage", stage.as_str());
        });
    }

    #[zbus(property)]
    fn rate(&self) -> f64 {
        1.0
    }

    #[zbus(property)]
    fn shuffle(&self) -> bool {
        self.engine.state.lock().unwrap().shuffle != mpv::ShuffleStage::Off
    }

    /// MPRIS Shuffle is a bool: true enables (keeps the current stage if one
    /// is active, else album shuffle), false turns shuffle off.
    #[zbus(property)]
    async fn set_shuffle(&self, on: bool) {
        let engine = self.engine.clone();
        let db_path = self.db_path.clone();
        tauri::async_runtime::spawn(async move {
            let cur = engine.state.lock().unwrap().shuffle;
            let stage = if on {
                if cur == mpv::ShuffleStage::Off {
                    mpv::ShuffleStage::Album
                } else {
                    cur
                }
            } else {
                mpv::ShuffleStage::Off
            };
            // No anchored rebuild from here (no DB access) — the stage lands;
            // the next play_album applies it. Mid-playback MPRIS toggles are
            // rare and the queue catches up on the next track click.
            let _ = engine.set_shuffle(stage, None).await;
            persist_stage(&db_path, "shuffleStage", stage.as_str());
        });
    }

    #[zbus(property)]
    fn metadata(&self) -> HashMap<String, OwnedValue> {
        self.metadata_map()
    }

    #[zbus(property)]
    fn volume(&self) -> f64 {
        (mpv::current_volume() / 100.0).clamp(0.0, 1.25)
    }

    #[zbus(property)]
    async fn set_volume(&self, volume: f64) {
        let engine = self.engine.clone();
        let vol = (volume * 100.0).clamp(0.0, 100.0);
        tauri::async_runtime::spawn(async move {
            let _ = engine.set_volume(vol).await;
        });
    }

    // Spec: Position must NOT be announced through PropertiesChanged; clients
    // poll it and resync on Seeked. We simply never call its *_changed hook.
    #[zbus(property)]
    fn position(&self) -> i64 {
        (mpv::current_position() * 1_000_000.0) as i64
    }

    #[zbus(property)]
    fn can_go_next(&self) -> bool {
        self.album_present()
    }

    #[zbus(property)]
    fn can_go_previous(&self) -> bool {
        self.album_present()
    }

    #[zbus(property)]
    fn can_play(&self) -> bool {
        self.album_present()
    }

    #[zbus(property)]
    fn can_pause(&self) -> bool {
        self.album_present()
    }

    #[zbus(property)]
    fn can_seek(&self) -> bool {
        self.album_present()
    }

    #[zbus(property)]
    fn can_control(&self) -> bool {
        true
    }

    async fn next(&self) {
        let _ = self.engine.jump(1).await;
    }

    async fn previous(&self) {
        let _ = self.engine.jump(-1).await;
    }

    async fn pause(&self) {
        let _ = self.engine.set_paused(true).await;
    }

    async fn play(&self) {
        if self.album_present() {
            let _ = self.engine.set_paused(false).await;
        }
    }

    async fn play_pause(&self) {
        if !self.album_present() {
            return;
        }
        let paused = !self.engine.state.lock().unwrap().paused;
        let _ = self.engine.set_paused(paused).await;
    }

    async fn stop(&self) {
        let _ = self.engine.stop().await;
    }

    /// Relative seek in microseconds, clamped to [0, duration].
    async fn seek(&self, offset: i64) {
        if !self.album_present() {
            return;
        }
        let target = (mpv::current_position() + offset as f64 / 1_000_000.0)
            .clamp(0.0, mpv::current_duration().max(0.0));
        let _ = self.engine.seek_to(target).await;
    }

    async fn set_position(&self, track_id: ObjectPath<'_>, position: i64) {
        let expected = {
            let st = self.engine.state.lock().unwrap();
            st.current().map(|item| Self::track_path(&item.track_id))
        };
        if expected.as_deref() != Some(track_id.as_str()) {
            return; // stale SetPosition from a previous track
        }
        let target = (position as f64 / 1_000_000.0)
            .clamp(0.0, mpv::current_duration().max(0.0));
        let _ = self.engine.seek_to(target).await;
    }

    #[zbus(signal)]
    async fn seeked(signal_emitter: &zbus::object_server::SignalEmitter<'_>, position: i64) -> zbus::Result<()>;
}

pub fn emit_seeked(pos_sec: f64) {
    if let Some(r) = PLAYER_REF.get() {
        let emitter = r.signal_emitter().clone();
        tauri::async_runtime::spawn(async move {
            let _ = Player::seeked(&emitter, (pos_sec * 1_000_000.0) as i64).await;
        });
    }
}

/// What the watcher compares between ticks to decide which PropertiesChanged
/// batches to emit.
#[derive(Clone, PartialEq)]
struct Snap {
    album: Option<String>,
    index: usize,
    paused: bool,
    volume: f64,
    shuffle: mpv::ShuffleStage,
    repeat: mpv::RepeatStage,
}

fn snap(engine: &mpv::Mpv) -> Snap {
    let st = engine.state.lock().unwrap();
    Snap {
        album: st.current().map(|i| i.album_id.clone()),
        index: st.index,
        paused: st.paused,
        volume: mpv::current_volume(),
        shuffle: st.shuffle,
        repeat: st.repeat,
    }
}

/// Persist a stage change that arrived over MPRIS (the app's own UI path
/// persists in the lib.rs commands). Best-effort: a failed write just means
/// the stage isn't remembered across launches.
fn persist_stage(db_path: &PathBuf, key: &str, value: &str) {
    if let Ok(conn) = rusqlite::Connection::open(db_path) {
        let _ = conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![key, value],
        );
    }
}

/// Spawn the MPRIS server + its property-change watcher on the tokio runtime.
/// Failure to claim the bus name (or any zbus error) is logged and swallowed:
/// a broken MPRIS must never take playback down with it.
pub fn serve(
    engine: Arc<mpv::Mpv>,
    db_path: PathBuf,
    thumbs_dir: PathBuf,
    app: tauri::AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        match serve_inner(engine, db_path, thumbs_dir, app).await {
            Ok(()) => eprintln!("[mpris] serving org.mpris.MediaPlayer2.songstress"),
            Err(e) => eprintln!("[mpris] unavailable: {e}"),
        }
    });
}

async fn serve_inner(
    engine: Arc<mpv::Mpv>,
    db_path: PathBuf,
    thumbs_dir: PathBuf,
    app: tauri::AppHandle,
) -> zbus::Result<()> {
    let player = Player {
        engine: engine.clone(),
        db_path,
        thumbs_dir,
    };
    let conn = zbus::connection::Builder::session()?
        .name("org.mpris.MediaPlayer2.songstress")?
        .serve_at("/org/mpris/MediaPlayer2", Root { app })?
        .serve_at("/org/mpris/MediaPlayer2", player)?
        .build()
        .await?;

    let iref = conn
        .object_server()
        .interface::<_, Player>("/org/mpris/MediaPlayer2")
        .await?;
    let _ = PLAYER_REF.set(iref.clone());

    let mut last = snap(&engine);
    loop {
        tokio::time::sleep(Duration::from_millis(200)).await;
        let cur = snap(&engine);
        if cur == last {
            continue;
        }
        let emitter = iref.signal_emitter().clone();
        let iface = iref.get_mut().await;
        let ctx_changed = cur.album != last.album || cur.index != last.index;
        let status_changed = cur.paused != last.paused
            || cur.album.is_some() != last.album.is_some();
        let volume_changed = (cur.volume - last.volume).abs() > f64::EPSILON;

        if status_changed {
            iface.playback_status_changed(&emitter).await?;
            iface.can_play_changed(&emitter).await?;
            iface.can_pause_changed(&emitter).await?;
        }
        if ctx_changed {
            iface.metadata_changed(&emitter).await?;
            iface.can_go_next_changed(&emitter).await?;
            iface.can_go_previous_changed(&emitter).await?;
            iface.can_seek_changed(&emitter).await?;
        }
        if volume_changed {
            iface.volume_changed(&emitter).await?;
        }
        if cur.shuffle != last.shuffle {
            iface.shuffle_changed(&emitter).await?;
        }
        if cur.repeat != last.repeat {
            iface.loop_status_changed(&emitter).await?;
        }
        drop(iface);
        last = cur;
    }
}
