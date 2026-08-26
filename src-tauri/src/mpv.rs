//! MPV JSON IPC playback engine (Phase 3).
//!
//! One mpv process is spawned at app start with `--no-config` (the user's
//! ~/.config/mpv/mpv.conf is broken Windows stuff) and talked to over its
//! JSON IPC unix socket: newline-delimited JSON, requests carry a numeric
//! `request_id`, responses echo it, everything else is an event.
//!
//! Rust owns the album context `{albumId, trackIndex}` (no queue — an album
//! click replaces it entirely). The remaining tracks are appended to mpv's
//! internal playlist for native gapless advance. State changes are forwarded
//! to the frontend as `playback-changed` / `playback-paused` /
//! throttled `playback-position` / `playback-stopped`.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, oneshot};

/// Shuffle mode (Step 5a): Off = album order; Album/Artist/All = the play
/// order is a shuffled pool (clicked track first, remainder shuffled).
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ShuffleStage {
    #[default]
    Off,
    Album,
    Artist,
    All,
}

impl ShuffleStage {
    pub fn parse(s: &str) -> Self {
        match s {
            "album" => Self::Album,
            "artist" => Self::Artist,
            "all" => Self::All,
            _ => Self::Off,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Album => "album",
            Self::Artist => "artist",
            Self::All => "all",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Off => Self::Album,
            Self::Album => Self::Artist,
            Self::Artist => Self::All,
            Self::All => Self::Off,
        }
    }
}

/// Repeat mode (Step 5a): Track = mpv loop-file=inf (native); Album = the
/// play order restarts at 0 on last-track eof (reshuffled when shuffle is
/// active); Off = stop.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum RepeatStage {
    #[default]
    Off,
    Album,
    Track,
}

impl RepeatStage {
    pub fn parse(s: &str) -> Self {
        match s {
            "album" => Self::Album,
            "track" => Self::Track,
            _ => Self::Off,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Album => "album",
            Self::Track => "track",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Off => Self::Album,
            Self::Album => Self::Track,
            Self::Track => Self::Off,
        }
    }
}

/// One entry of the play order. The order usually IS one album, but
/// artist/all shuffle pools span albums — every item carries its own album
/// context so the `{albumId, trackIndex}` event contract stays intact.
#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct OrderItem {
    #[serde(rename = "trackId")]
    pub track_id: String,
    pub path: String,
    #[serde(rename = "albumId")]
    pub album_id: String,
    /// Position of this track within ITS album (event contract).
    #[serde(rename = "albumIndex")]
    pub album_index: usize,
}

/// The play order + current index. Rust owns this; the frontend mirrors it
/// from events.
#[derive(Default)]
pub struct PlayState {
    pub order: Vec<OrderItem>,
    pub index: usize,
    pub paused: bool,
    pub shuffle: ShuffleStage,
    pub repeat: RepeatStage,
    /// Pre-armed continuation for repeat=album: the (reshuffled) next pass,
    /// appended to mpv's playlist WHILE the last track plays so the wrap is
    /// gapless — mpv never goes idle, so the audio device never reopens
    /// (the reopen was the multi-second wrap delay). Promoted to `order` on
    /// the last track's eof.
    pub wrapped: Option<Vec<OrderItem>>,
    /// User queue (Step 7a): entries play BEFORE the order resumes, in
    /// enqueue order, never shuffled; session-only. While an entry plays it
    /// is PROMOTED into `order` at the current index (so events/MPRIS
    /// metadata stay correct) and consumed from `order` at its eof —
    /// `queue` itself only holds entries that have not played yet.
    pub queue: VecDeque<OrderItem>,
    /// True while the entry at `order[index]` is a promoted queue entry
    /// (consumed at its eof instead of advancing the index).
    pub from_queue: bool,
}

impl PlayState {
    pub fn current(&self) -> Option<&OrderItem> {
        self.order.get(self.index)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn current_track_id(&self) -> Option<&str> {
        self.current().map(|i| i.track_id.as_str())
    }
}

/// Tiny xorshift RNG — no `rand` dependency; seeded from the clock, only
/// used for Fisher-Yates on play orders.
pub struct Rng(u64);

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// In-place Fisher-Yates. Pure fn of the slice + seed → unit-testable.
    pub fn shuffle<T>(items: &mut [T], seed: u64) {
        let mut rng = Rng(seed | 1);
        for i in (1..items.len()).rev() {
            let j = (rng.next_u64() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }
}

#[derive(Serialize)]
struct PlayChangedEvent<'a> {
    #[serde(rename = "albumId")]
    album_id: &'a str,
    #[serde(rename = "trackIndex")]
    track_index: usize,
    /// Anchors the UI to the actual playing track even when a rescan
    /// re-orders/re-groups the album between events (index alone drifts).
    #[serde(rename = "trackId")]
    track_id: &'a str,
}

#[derive(Serialize)]
struct PositionEvent {
    pos: f64,
    dur: f64,
}

#[derive(Serialize)]
struct QueueEntry<'a> {
    #[serde(rename = "trackId")]
    track_id: &'a str,
    #[serde(rename = "albumId")]
    album_id: &'a str,
    #[serde(rename = "albumIndex")]
    album_index: usize,
}

#[derive(Serialize)]
struct QueueEvent<'a> {
    /// Entries waiting to play (never includes the playing one).
    queue: Vec<QueueEntry<'a>>,
    /// First few ORDER entries after the current one, for the popover's
    /// "up next from <album>" preview.
    #[serde(rename = "upNext")]
    up_next: Vec<QueueEntry<'a>>,
}

fn socket_path() -> PathBuf {
    runtime_dir().join("mpv.sock")
}

fn runtime_dir() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("songstress");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Reap an orphaned engine from a previous app session (idle=yes keeps mpv
/// alive after its parent dies, and each launch would otherwise stack
/// another silent instance).
fn reap_previous() {
    let pidfile = runtime_dir().join("mpv.pid");
    if let Ok(txt) = std::fs::read_to_string(&pidfile) {
        if let Ok(pid) = txt.trim().parse::<u32>() {
            kill_pid_guarded(pid);
        }
    }
}

/// Kill the running engine — called on app exit, since mpv is a detached
/// child that would otherwise keep playing after the window closes.
pub fn kill_engine() {
    if let Ok(txt) = std::fs::read_to_string(runtime_dir().join("mpv.pid")) {
        if let Ok(pid) = txt.trim().parse::<u32>() {
            kill_pid_guarded(pid);
        }
    }
}

fn kill_pid_guarded(pid: u32) {
    // Only kill if it's really our mpv (pid reuse guard).
    let cmdline =
        std::fs::read_to_string(format!("/proc/{pid}/cmdline")).unwrap_or_default();
    if cmdline.contains("input-ipc-server=") && cmdline.contains("songstress") {
        let _ = std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .status();
    }
}

static APP: std::sync::OnceLock<tauri::AppHandle> = std::sync::OnceLock::new();

pub fn set_app_handle(app: tauri::AppHandle) {
    let _ = APP.set(app);
}

fn emit<T: Serialize>(event: &str, payload: &T) {
    use tauri::Emitter;
    if let Some(app) = APP.get() {
        let _ = app.emit(event, serde_json::to_value(payload).ok());
    }
}

/// Current-track duration, shared between the reader task and commands.
static DURATION_BITS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn store_duration(d: f64) {
    DURATION_BITS.store(d.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

pub fn current_duration() -> f64 {
    f64::from_bits(DURATION_BITS.load(std::sync::atomic::Ordering::Relaxed))
}

/// Current position — stored on EVERY mpv time-pos tick (the ~10 Hz throttle
/// is only for frontend events; MPRIS Position polls this and must be fresh).
static POS_BITS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub fn store_position(p: f64) {
    POS_BITS.store(p.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

pub fn current_position() -> f64 {
    f64::from_bits(POS_BITS.load(std::sync::atomic::Ordering::Relaxed))
}

/// Engine volume, observed from mpv so external + UI changes stay in sync
/// for the MPRIS Volume property.
static VOL_BITS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn store_volume(v: f64) {
    VOL_BITS.store(v.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

pub fn current_volume() -> f64 {
    f64::from_bits(VOL_BITS.load(std::sync::atomic::Ordering::Relaxed))
}

// --- pending request registry -------------------------------------------------
// Responses arrive on the reader task while commands await on their own tasks,
// so the map must be process-global, not thread-local.

static PENDING: LazyLock<Mutex<HashMap<u64, oneshot::Sender<String>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// How many upcoming tracks to keep in mpv's playlist for gapless advance.
/// The play ORDER can be thousands long (all-artists shuffle); the playlist
/// is just a rolling buffer — the eof advance tops it up by one.
const PLAYLIST_WINDOW: usize = 32;

fn register(id: u64, tx: oneshot::Sender<String>) {
    PENDING.lock().unwrap().insert(id, tx);
}

fn resolve(id: u64, line: &str) -> bool {
    if let Some(tx) = PENDING.lock().unwrap().remove(&id) {
        let _ = tx.send(line.to_string());
        true
    } else {
        false
    }
}

pub struct Mpv {
    pub state: Arc<Mutex<PlayState>>,
    /// Requests funnel through one channel into a single writer task so
    /// lines never interleave.
    tx: mpsc::UnboundedSender<String>,
    next_id: std::sync::atomic::AtomicU64,
}

impl Mpv {
    /// Spawn mpv and connect. Returns after the socket answers (retrying
    /// while mpv creates it). A dead engine just makes commands fail later.
    /// Spawn mpv and connect. Returns after the socket answers (retrying
    /// while mpv creates it). A dead engine just makes commands fail later.
    /// `initial_volume` comes from the persisted settings so the FIRST track
    /// of a session plays at the right level (the old hardcoded --volume=80
    /// blared until the frontend pushed the saved value ~1.5s later).
    pub async fn spawn(
        state: Arc<Mutex<PlayState>>,
        initial_volume: f64,
    ) -> Result<Arc<Self>, String> {
        reap_previous();
        let sock = socket_path();
        let _ = std::fs::remove_file(&sock);

        let child = tokio::process::Command::new("mpv")
            .args([
                "--no-config",
                "--idle=yes",
                "--no-video",
                "--gapless-audio=yes",
                "--replaygain=album",
                "--really-quiet",
                &format!("--volume={initial_volume}"),
                &format!("--input-ipc-server={}", sock.display()),
            ])
            .stdin(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn mpv: {e}"))?;
        if let Some(pid) = child.id() {
            let _ = std::fs::write(runtime_dir().join("mpv.pid"), pid.to_string());
        }

        // Retry while mpv gets around to creating the socket.
        let mut stream = None;
        for _ in 0..100 {
            match tokio::net::UnixStream::connect(&sock).await {
                Ok(s) => {
                    stream = Some(s);
                    break;
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(50)).await,
            }
        }
        let stream = stream.ok_or("mpv socket never appeared")?;
        let (reader, mut writer) = stream.into_split();

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                if writer.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        let this = Arc::new(Self {
            state,
            tx,
            next_id: std::sync::atomic::AtomicU64::new(1),
        });
        store_volume(initial_volume);
        Self::spawn_reader(&this, reader);
        // mpv pushes NOTHING for these properties until explicitly observed.
        for (id, prop) in
            [(1i64, "time-pos"), (2, "duration"), (3, "pause"), (4, "volume")]
        {
            let cmd = serde_json::json!(["observe_property", id, prop]);
            if let Err(e) = this.command(cmd).await {
                eprintln!("[mpv] observe_property({prop}) failed: {e}");
            }
        }
        Ok(this)
    }

    /// A dead handle for when mpv could not be spawned: every command fails
    /// with a clean error (the writer's receiver is dropped immediately).
    pub fn detached(state: Arc<Mutex<PlayState>>) -> Arc<Self> {
        let (tx, _rx) = mpsc::unbounded_channel::<String>();
        Arc::new(Self {
            state,
            tx,
            next_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    fn spawn_reader(this: &Arc<Self>, reader: tokio::net::unix::OwnedReadHalf) {
        let this = Arc::clone(this);
        tauri::async_runtime::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            let mut last_pos_emit = Instant::now();
            while let Ok(Some(line)) = lines.next_line().await {
                let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) else {
                    continue;
                };
                // Responses echo request_id; events never carry one.
                if let Some(id) = msg.get("request_id").and_then(|v| v.as_u64()) {
                    resolve(id, &line);
                    continue;
                }
                let event = msg.get("event").and_then(|v| v.as_str()).unwrap_or("");
                match event {
                    "property-change" => {
                        let name = msg.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        match name {
                            "time-pos" => {
                                // Throttle to ~10 Hz — mpv fires far more often.
                                if let Some(pos) = number(&msg, "data") {
                                    store_position(pos);
                                    if last_pos_emit.elapsed() >= Duration::from_millis(100) {
                                        last_pos_emit = Instant::now();
                                        emit(
                                            "playback-position",
                                            &PositionEvent { pos, dur: current_duration() },
                                        );
                                    }
                                }
                            }
                            "duration" => {
                                if let Some(d) = number(&msg, "data") {
                                    store_duration(d);
                                }
                            }
                            "pause" => {
                                let paused = msg
                                    .get("data")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                this.state.lock().unwrap().paused = paused;
                                emit("playback-paused", &paused);
                            }
                            "volume" => {
                                if let Some(v) = number(&msg, "data") {
                                    store_volume(v);
                                }
                            }
                            _ => {}
                        }
                    }
                    "end-file" => {
                        // eof → advance/repeat/stop; error → skip forward too.
                        // The DECISION is a pure fn of PlayState (unit-tested);
                        // the reader only executes what it prescribes — and
                        // everything here is fire-and-forget (NEVER await IPC
                        // in the reader task).
                        let reason = msg.get("reason").and_then(|v| v.as_str());
                        if matches!(reason, Some("eof") | Some("error")) {
                            let action = eof_advance(&mut this.state.lock().unwrap());
                            match action {
                                Some(EofAction::Advance { top_up, pre }) => {
                                    for p in pre {
                                        this.fire(serde_json::json!(["loadfile", p, "append-play"]));
                                    }
                                    if let Some(p) = top_up {
                                        this.fire(serde_json::json!(["loadfile", p, "append-play"]));
                                    }
                                    this.emit_changed();
                                    this.emit_queue_changed();
                                }
                                Some(EofAction::Reload(paths)) => {
                                    // mpv is idle after eof — the wrapped
                                    // order must be LOADED, not appended.
                                    this.fire(serde_json::json!(["playlist-clear"]));
                                    this.fire(serde_json::json!([
                                        "loadfile", &paths[0], "replace"
                                    ]));
                                    let end = PLAYLIST_WINDOW.min(paths.len());
                                    for p in &paths[1..end] {
                                        this.fire(serde_json::json!([
                                            "loadfile", p, "append-play"
                                        ]));
                                    }
                                    this.emit_changed();
                                    this.emit_queue_changed();
                                }
                                Some(EofAction::Stop) => {
                                    emit("playback-stopped", &true);
                                    this.emit_queue_changed();
                                }
                                None => {} // idle transition (we called stop())
                            }
                        }
                    }
                    _ => {}
                }
            }
            eprintln!("[mpv] reader exited (socket closed)");
        });
    }

    /// `args` must be a JSON array — mpv IPC property VALUES are typed
    /// (booleans as real booleans, observe ids as ints); strings fail with
    /// "unsupported format"/"invalid parameter".
    async fn command(&self, args: serde_json::Value) -> Result<Option<serde_json::Value>, String> {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let payload = serde_json::json!({ "command": args, "request_id": id });
        let (tx, rx) = oneshot::channel();
        register(id, tx);
        self.tx
            .send(format!("{payload}\n"))
            .map_err(|_| "mpv writer gone")?;

        let line = tokio::time::timeout(Duration::from_secs(10), rx)
            .await
            .map_err(|_| format!("mpv response timeout for {args}"))?
            .map_err(|_| "mpv dropped the response")?;
        let v: serde_json::Value =
            serde_json::from_str(&line).map_err(|e| format!("bad ipc line: {e}"))?;
        match v.get("error").and_then(|e| e.as_str()) {
            Some("success") => Ok(v.get("data").cloned()),
            other => Err(other.unwrap_or("unknown mpv error").to_string()),
        }
    }

    fn emit_changed(&self) {
        let st = self.state.lock().unwrap();
        if let Some(item) = st.current() {
            emit(
                "playback-changed",
                &PlayChangedEvent {
                    album_id: &item.album_id,
                    track_index: item.album_index,
                    track_id: &item.track_id,
                },
            );
        }
    }

    /// Mirror the user queue + order preview to the frontend (Step 7a).
    fn emit_queue_changed(&self) {
        let st = self.state.lock().unwrap();
        let queue: Vec<QueueEntry> = st
            .queue
            .iter()
            .map(|i| QueueEntry {
                track_id: &i.track_id,
                album_id: &i.album_id,
                album_index: i.album_index,
            })
            .collect();
        let up_next: Vec<QueueEntry> = st.order[(st.index + 1).min(st.order.len())..]
            .iter()
            .take(3)
            .map(|i| QueueEntry {
                track_id: &i.track_id,
                album_id: &i.album_id,
                album_index: i.album_index,
            })
            .collect();
        emit("queue-changed", &QueueEvent { queue, up_next });
    }

    /// Send a command WITHOUT waiting for its response. Used for playlist
    /// appends: awaiting each one made queue rebuilds take seconds (every
    /// later command queues behind the drain). Responses for unregistered ids
    /// are dropped by resolve().
    fn fire(&self, args: serde_json::Value) {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let payload = serde_json::json!({ "command": args, "request_id": id });
        let _ = self.tx.send(format!("{payload}\n"));
    }

    /// Replace the mpv playlist with `order[start..]` after the currently
    /// playing file. `replace_current` loads order[start] right away
    /// (play/seek/eof-wrap semantics); otherwise the playing file keeps
    /// playing and only the upcoming queue is swapped (mid-playback shuffle
    /// rebuild).
    ///
    /// Only a WINDOW of upcoming tracks is appended: artist/all pools can be
    /// thousands of entries, and one IPC command per entry both floods mpv
    /// (every later command queues behind the drain → 10s timeouts) and
    /// wastes memory. PlayState.order stays the full source of truth; the
    /// eof advance tops the window up by one. Appends are fire-and-forget so
    /// rebuilds complete instantly.
    async fn load_queue(&self, start: usize, order_paths: &[String], replace_current: bool) {
        // NOTE: playlist-clear takes no argument (`playlist-remove` is the
        // one that wants "current") — passing one errors out the whole play.
        self.command(serde_json::json!(["playlist-clear"])).await.ok();
        if replace_current {
            self.command(serde_json::json!(["loadfile", &order_paths[start], "replace"])).await.ok();
        }
        // User queue entries sit BETWEEN the current track and the order
        // window — appending them first yields exactly that playlist order.
        let queued: Vec<String> = {
            let st = self.state.lock().unwrap();
            st.queue.iter().map(|i| i.path.clone()).collect()
        };
        for path in &queued {
            self.fire(serde_json::json!(["loadfile", path, "append-play"]));
        }
        let end = (start + PLAYLIST_WINDOW).min(order_paths.len());
        for path in &order_paths[start + 1..end] {
            self.fire(serde_json::json!(["loadfile", path, "append-play"]));
        }
        // A pre-armed repeat=album pass plays after everything above; the
        // rebuild wiped it from mpv's playlist, so re-append it.
        let wrapped: Vec<String> = {
            let st = self.state.lock().unwrap();
            st.wrapped.as_ref().map(|w| w.iter().map(|i| i.path.clone()).collect()).unwrap_or_default()
        };
        let end = PLAYLIST_WINDOW.min(wrapped.len());
        for path in &wrapped[..end] {
            self.fire(serde_json::json!(["loadfile", path, "append-play"]));
        }
    }

    /// Load a play order: replace the playlist with the target track and
    /// append the rest for native gapless advance.
    pub async fn play_order(&self, order: Vec<OrderItem>, start: usize) -> Result<(), String> {
        if start >= order.len() {
            return Err("track index out of range".into());
        }
        let (shuffle, repeat) = {
            let st = self.state.lock().unwrap();
            (st.shuffle, st.repeat)
        };
        *self.state.lock().unwrap() = PlayState {
            order,
            index: start,
            paused: false,
            shuffle,
            repeat,
            wrapped: None,
            queue: Default::default(),
            from_queue: false,
        };
        store_duration(0.0);
        store_position(0.0);
        let paths: Vec<String> = {
            let st = self.state.lock().unwrap();
            st.order.iter().map(|i| i.path.clone()).collect()
        };
        // Clicking straight INTO the last track with repeat=album: pre-arm
        // the wrap BEFORE the playlist rebuild — load_queue re-appends the
        // armed pass itself (single append path, no duplicates).
        {
            let mut st = self.state.lock().unwrap();
            if start + 1 == st.order.len() && st.repeat == RepeatStage::Album {
                arm_wrap(&mut st);
            }
        }
        self.load_queue(start, &paths, true).await;
        self.set_paused(false).await?;
        self.emit_changed();
        self.emit_queue_changed();
        Ok(())
    }

    /// Next/Prev wrap around within the play order (user request: prev on
    /// the first track plays the last, next past the end plays the first).
    /// Rebuilds the queue rather than trusting mpv playlist alignment:
    /// played entries are consumed, so a plain `loadfile replace` would leave
    /// the tail pointing past the wrong tracks.
    ///
    /// Step 7a: Next RESPECTS the user queue — the queue front plays before
    /// the order resumes (MPRIS Next goes through here too). Previous from a
    /// promoted queue entry returns to the order entry before it.
    pub async fn jump(&self, delta: i64) -> Result<(), String> {
        let (index, paths) = {
            let mut st = self.state.lock().unwrap();
            if st.current().is_none() {
                return Ok(());
            }
            if delta > 0 && !st.queue.is_empty() {
                // Consume the current queue entry (if any), promote the front.
                if st.from_queue {
                    let i = st.index;
                st.order.remove(i);
                } else {
                    st.index += 1;
                }
                let next = st.queue.pop_front().unwrap();
                let i = st.index;
                st.order.insert(i, next);
                st.from_queue = true;
            } else if delta > 0 && st.from_queue {
                // Queue drained: fall back into the order at the entry after
                // the consumed one (it slid into `index` on removal).
                let i = st.index;
                st.order.remove(i);
                st.from_queue = false;
            } else if delta < 0 && st.from_queue {
                // Previous from a queue entry → the order entry before it.
                let i = st.index;
                st.order.remove(i);
                st.from_queue = false;
                st.index = if st.index == 0 { st.order.len() - 1 } else { st.index - 1 };
            } else {
                let mut t = st.index as i64 + delta;
                if delta > 0 && t >= st.order.len() as i64 {
                    // Forward wrap past the end: a pre-armed next pass IS what
                    // mpv plays next — promote it so the jump matches.
                    if let Some(w) = st.wrapped.take() {
                        st.order = w;
                    }
                    t = 0;
                } else if t < 0 {
                    t = st.order.len() as i64 - 1;
                } else if t >= st.order.len() as i64 {
                    t = 0;
                }
                st.index = t as usize;
                st.from_queue = false;
            }
            st.paused = false;
            // The queue rebuild discards any pre-appended next pass.
            st.wrapped = None;
            (st.index, st.order.iter().map(|i| i.path.clone()).collect::<Vec<_>>())
        };
        store_duration(0.0);
        store_position(0.0);
        self.load_queue(index, &paths, true).await;
        self.set_paused(false).await?;
        self.emit_changed();
        self.emit_queue_changed();
        Ok(())
    }

    // --- user queue (Step 7a) ------------------------------------------------

    /// Enqueue tracks. `front` = "Play next" (before other queued entries,
    /// after the current track); otherwise "Add to queue" (back). While
    /// playing, the playlist tail is rebuilt so queue entries sit between
    /// the current track and the order window. While STOPPED the entries
    /// just wait in `queue` (they never auto-play; any album click clears
    /// them — the popover shows and removes them).
    pub async fn queue_tracks(&self, items: Vec<OrderItem>, front: bool) -> Result<(), String> {
        let (index, playing, paths) = {
            let mut st = self.state.lock().unwrap();
            for (k, item) in items.into_iter().enumerate() {
                if front {
                    st.queue.insert(k, item);
                } else {
                    st.queue.push_back(item);
                }
            }
            let playing = st.current().is_some();
            let paths = st.order.iter().map(|i| i.path.clone()).collect::<Vec<_>>();
            (st.index, playing, paths)
        };
        if playing {
            // Current file keeps playing; only the tail is rebuilt.
            self.load_queue(index, &paths, false).await;
        }
        self.emit_queue_changed();
        Ok(())
    }

    /// Remove the queued entry at `pos` (popover ×).
    pub async fn queue_remove(&self, pos: usize) -> Result<(), String> {
        let (index, playing, paths) = {
            let mut st = self.state.lock().unwrap();
            if pos >= st.queue.len() {
                return Err("queue position out of range".into());
            }
            st.queue.remove(pos);
            let playing = st.current().is_some();
            let paths = st.order.iter().map(|i| i.path.clone()).collect::<Vec<_>>();
            (st.index, playing, paths)
        };
        if playing {
            self.load_queue(index, &paths, false).await;
        }
        self.emit_queue_changed();
        Ok(())
    }

    /// Play the queued entry at `pos` NOW (popover click). Entries before it
    /// are discarded; the rest stay queued. A currently-playing queue entry
    /// is consumed first.
    pub async fn queue_jump(&self, pos: usize) -> Result<(), String> {
        let (index, paths) = {
            let mut st = self.state.lock().unwrap();
            if st.current().is_none() {
                return Ok(()); // stopped: nothing to take over the click
            }
            if pos >= st.queue.len() {
                return Err("queue position out of range".into());
            }
            if st.from_queue {
                let i = st.index;
                st.order.remove(i);
            } else {
                st.index += 1;
            }
            for _ in 0..pos {
                st.queue.pop_front();
            }
            let next = st.queue.pop_front().unwrap();
            let i = st.index;
            st.order.insert(i, next);
            st.from_queue = true;
            st.paused = false;
            st.wrapped = None;
            (st.index, st.order.iter().map(|i| i.path.clone()).collect::<Vec<_>>())
        };
        store_duration(0.0);
        store_position(0.0);
        self.load_queue(index, &paths, true).await;
        self.set_paused(false).await?;
        self.emit_changed();
        self.emit_queue_changed();
        Ok(())
    }

    pub async fn set_paused(&self, paused: bool) -> Result<(), String> {
        self.state.lock().unwrap().paused = paused;
        self.command(serde_json::json!(["set_property", "pause", paused])).await?;
        emit("playback-paused", &paused);
        Ok(())
    }

    pub async fn seek_to(&self, sec: f64) -> Result<(), String> {
        self.command(serde_json::json!(["seek", sec, "absolute"])).await?;
        store_position(sec);
        crate::mpris::emit_seeked(sec);
        emit(
            "playback-position",
            &PositionEvent { pos: sec, dur: current_duration() },
        );
        Ok(())
    }

    pub async fn set_volume(&self, vol: f64) -> Result<(), String> {
        self.command(serde_json::json!(["set_property", "volume", vol])).await?;
        store_volume(vol);
        Ok(())
    }

    /// Replace the audio filter chain (Step 6 EQ). None clears `af` — an
    /// empty string resets it to the default (no filters), so a disabled or
    /// flat EQ costs nothing per sample. `af` is a plain option; setting it
    /// mid-playback applies to the current file immediately and survives
    /// gapless transitions.
    pub async fn set_af(&self, chain: Option<String>) -> Result<(), String> {
        self.command(serde_json::json!([
            "set_property", "af", chain.unwrap_or_default()
        ]))
        .await?;
        Ok(())
    }

    /// Store the shuffle stage. `anchored_order` (built by the DB-owning
    /// caller with the current track first) replaces the queue mid-playback
    /// without restarting the current file; None (stopped) just stores.
    pub async fn set_shuffle(
        &self,
        stage: ShuffleStage,
        anchored_order: Option<(Vec<OrderItem>, usize)>,
    ) -> Result<(), String> {
        let playing = { self.state.lock().unwrap().current().is_some() };
        if let Some((order, start)) = anchored_order {
            if playing && !order.is_empty() {
                {
                    let mut st = self.state.lock().unwrap();
                    st.order = order;
                    st.index = start;
                    // The queue rebuild discards any pre-appended next pass.
                    st.wrapped = None;
                }
                let paths: Vec<String> = {
                    let st = self.state.lock().unwrap();
                    st.order.iter().map(|i| i.path.clone()).collect()
                };
                // Current file keeps playing; only the upcoming queue swaps.
                self.load_queue(start, &paths, false).await;
            }
        }
        self.state.lock().unwrap().shuffle = stage;
        Ok(())
    }

    /// Store the repeat stage; Track = native mpv loop (never reaches
    /// end-file), anything else clears it. Arming repeat=album while the
    /// LAST track is already playing pre-wraps immediately (otherwise the
    /// eof fallback reload would idle mpv).
    pub async fn set_repeat(&self, stage: RepeatStage) -> Result<(), String> {
        let pre_wrap: Option<Vec<String>> = {
            let mut st = self.state.lock().unwrap();
            st.repeat = stage;
            if stage != RepeatStage::Album {
                st.wrapped = None;
                None
            } else if st.wrapped.is_none()
                && st.queue.is_empty()
                && st.current().is_some()
                && st.index + 1 == st.order.len()
            {
                let paths = arm_wrap(&mut st);
                let end = paths.len().min(PLAYLIST_WINDOW);
                Some(paths[..end].to_vec())
            } else {
                None
            }
        };
        if let Some(paths) = pre_wrap {
            for p in paths {
                self.fire(serde_json::json!(["loadfile", p, "append-play"]));
            }
        }
        let loop_val = if stage == RepeatStage::Track { "inf" } else { "no" };
        self.command(serde_json::json!(["set_property", "loop-file", loop_val])).await?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        // Stages survive stop — they are modes, not transient state.
        let (shuffle, repeat) = {
            let st = self.state.lock().unwrap();
            (st.shuffle, st.repeat)
        };
        *self.state.lock().unwrap() = PlayState {
            shuffle,
            repeat,
            ..PlayState::default()
        };
        store_position(0.0);
        // Already idle after natural end → error is fine to swallow.
        let _ = self.command(serde_json::json!(["stop"])).await;
        emit("playback-stopped", &true);
        Ok(())
    }
}

/// Build the next pass for repeat=album: same order, reshuffled when
/// shuffle is active. Stashes it in `st.wrapped` and returns the append
/// paths (caller fires them into mpv's playlist while the last track plays).
/// A currently-playing PROMOTED QUEUE ENTRY is excluded — queue entries play
/// once and must not leak into the repeat pass.
fn arm_wrap(st: &mut PlayState) -> Vec<String> {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mut next_pass: Vec<OrderItem> = st
        .order
        .iter()
        .enumerate()
        .filter(|(i, _)| !(st.from_queue && *i == st.index))
        .map(|(_, it)| it.clone())
        .collect();
    if st.shuffle != ShuffleStage::Off {
        Rng::shuffle(&mut next_pass, seed);
    }
    let paths: Vec<String> = next_pass.iter().map(|i| i.path.clone()).collect();
    st.wrapped = Some(next_pass);
    paths
}

/// What the eof handler must DO after an end-file (eof or error): pure fn of
/// PlayState so the queue/order/window choreography is unit-testable without
/// a live mpv. `None` = idle transition (stop() was called) — do nothing.
///
/// mpv advances NATIVELY into playlist position 1; the invariant this fn
/// maintains is: playlist = [current] ++ queue ++ order[index+1..index+31]
/// (++ pre-armed wrap pass, which only exists when the window is empty).
/// A promoted queue entry lives at `order[index]` until its own eof consumes
/// it, keeping events/MPRIS metadata correct in the meantime.
pub enum EofAction {
    /// Keep playing (mpv already advanced); fire the pre-wrap appends, then
    /// at most one window top-up — IN THIS ORDER.
    Advance { top_up: Option<String>, pre: Vec<String> },
    /// mpv is idle (wrap without pre-arm): playlist-clear + reload from 0.
    Reload(Vec<String>),
    Stop,
}

impl std::fmt::Debug for EofAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EofAction::Advance { top_up, pre } => f
                .debug_struct("Advance")
                .field("top_up", &top_up.as_deref())
                .field("pre", &pre.len())
                .finish(),
            EofAction::Reload(paths) => write!(f, "Reload({} paths)", paths.len()),
            EofAction::Stop => write!(f, "Stop"),
        }
    }
}

/// Pre-arm the repeat=album wrap when the LAST entry just started playing
/// and nothing is queued (queue entries must play before any next pass).
fn pre_arm_if_last(st: &mut PlayState) -> Vec<String> {
    if st.index + 1 == st.order.len()
        && st.repeat == RepeatStage::Album
        && st.wrapped.is_none()
        && st.queue.is_empty()
    {
        let paths = arm_wrap(st);
        let end = paths.len().min(PLAYLIST_WINDOW);
        paths[..end].to_vec()
    } else {
        Vec::new()
    }
}

pub fn eof_advance(st: &mut PlayState) -> Option<EofAction> {
    if st.current().is_none() {
        return None; // idle transition (we called stop())
    }
    let was_queued = st.from_queue;
    if was_queued {
        // The ended entry was a promoted queue entry: consume it. What mpv
        // advanced into (playlist position 1) is the queue front, or the
        // order entry that slides into `index` when the queue is empty.
        let i = st.index;
                st.order.remove(i);
        st.from_queue = false;
    }
    if let Some(next) = st.queue.pop_front() {
        if was_queued {
            let i = st.index;
            st.order.insert(i, next); // replaces the consumed entry
        } else {
            st.index += 1;
            let i = st.index;
            st.order.insert(i, next);
        }
        st.from_queue = true;
        // No window top-up: mpv consumed a QUEUE entry, the order window in
        // the playlist did not slide — it already satisfies the invariant.
        let pre = pre_arm_if_last(st);
        return Some(EofAction::Advance { top_up: None, pre });
    }
    if was_queued {
        // Queue drained with this eof. If the consumed entry was the LAST
        // order entry, the index is now out of bounds: mpv either advanced
        // into the pre-armed pass (promote it) or went idle (stop).
        if st.index >= st.order.len() {
            if st.repeat == RepeatStage::Album {
                if let Some(wrapped) = st.wrapped.take() {
                    st.order = wrapped;
                    st.index = 0;
                    let top_up = st.order.get(PLAYLIST_WINDOW - 1).map(|i| i.path.clone());
                    return Some(EofAction::Advance { top_up, pre: Vec::new() });
                }
            }
            *st = PlayState {
                shuffle: st.shuffle,
                repeat: st.repeat,
                ..PlayState::default()
            };
            return Some(EofAction::Stop);
        }
        // current = order[index]; the order window slid by one.
        if st.index + 1 == st.order.len() && st.repeat == RepeatStage::Album {
            if let Some(wrapped) = st.wrapped.take() {
                // mpv advanced into the pre-armed pass: promote it wholesale.
                st.order = wrapped;
                st.index = 0;
                let top_up = st.order.get(PLAYLIST_WINDOW - 1).map(|i| i.path.clone());
                return Some(EofAction::Advance { top_up, pre: Vec::new() });
            }
        }
        let top_up = st.order.get(st.index + PLAYLIST_WINDOW - 1).map(|i| i.path.clone());
        let pre = pre_arm_if_last(st);
        return Some(EofAction::Advance { top_up, pre });
    }
    if st.index + 1 < st.order.len() {
        st.index += 1;
        let top_up = st.order.get(st.index + PLAYLIST_WINDOW - 1).map(|i| i.path.clone());
        let pre = pre_arm_if_last(st);
        Some(EofAction::Advance { top_up, pre })
    } else if let Some(wrapped) = st.wrapped.take() {
        // Gapless wrap: mpv already advanced into the pre-appended next pass.
        st.order = wrapped;
        st.index = 0;
        let top_up = st.order.get(PLAYLIST_WINDOW - 1).map(|i| i.path.clone());
        Some(EofAction::Advance { top_up, pre: Vec::new() })
    } else if st.repeat == RepeatStage::Album {
        // No pre-arm (repeat set during the last track): idle mpv — reload.
        if st.shuffle != ShuffleStage::Off {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            Rng::shuffle(&mut st.order, seed);
        }
        st.index = 0;
        Some(EofAction::Reload(st.order.iter().map(|i| i.path.clone()).collect()))
    } else {
        *st = PlayState {
            shuffle: st.shuffle,
            repeat: st.repeat,
            ..PlayState::default()
        };
        Some(EofAction::Stop)
    }
}

fn number<'a>(msg: &'a serde_json::Value, key: &str) -> Option<f64> {
    msg.get(key).and_then(|v| v.as_f64())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn play_state_current_track_id() {
        let mut st = PlayState::default();
        assert!(st.current_track_id().is_none());
        st.order = vec![
            OrderItem {
                track_id: "tr-1".into(),
                path: "/a.mp3".into(),
                album_id: "al-1".into(),
                album_index: 0,
            },
            OrderItem {
                track_id: "tr-2".into(),
                path: "/b.mp3".into(),
                album_id: "al-1".into(),
                album_index: 1,
            },
        ];
        assert_eq!(st.current_track_id(), Some("tr-1"), "index 0 of a set order");
        st.index = 5;
        assert_eq!(st.current_track_id(), None, "out of range → none");
    }

    #[test]
    fn stage_parsing_and_cycles() {
        assert_eq!(ShuffleStage::parse("album"), ShuffleStage::Album);
        assert_eq!(ShuffleStage::parse("bogus"), ShuffleStage::Off);
        let mut s = ShuffleStage::Off;
        for _ in 0..4 {
            s = s.next();
        }
        assert_eq!(s, ShuffleStage::Off, "shuffle cycle closes the ring");
        assert_eq!(RepeatStage::parse("track"), RepeatStage::Track);
        let mut r = RepeatStage::Off;
        for _ in 0..3 {
            r = r.next();
        }
        assert_eq!(r, RepeatStage::Off, "repeat cycle closes the ring");
    }

    #[test]
    fn fisher_yates_is_a_permutation_and_deterministic() {
        let a: Vec<u32> = (0..64).collect();
        let mut b = a.clone();
        Rng::shuffle(&mut b, 42);
        let mut sorted = b.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, a, "same multiset after shuffle");
        let mut c = a.clone();
        Rng::shuffle(&mut c, 42);
        assert_eq!(b, c, "same seed → same order");
        assert_ne!(b, a, "64 items shuffling to identity is astronomically unlikely");
    }

    #[test]
    fn pending_registry_roundtrip_and_miss() {
        let (tx, rx) = oneshot::channel();
        register(42, tx);
        assert!(resolve(42, r#"{"error":"success"}"#));
        assert!(!resolve(42, "again"), "already consumed");
        assert!(!resolve(9999, "nobody waiting"));
        assert_eq!(
            futures_block(rx).as_deref(),
            Some(r#"{"error":"success"}"#)
        );
    }

    /// oneshot receiver is not sync-pollable without a runtime; a tiny single-
    /// step runtime keeps the test dependency-free.
    fn futures_block(mut rx: oneshot::Receiver<String>) -> Option<String> {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .expect("rt")
            .block_on(async move {
                tokio::select! {
                    msg = &mut rx => msg.ok(),
                    _ = tokio::time::sleep(Duration::from_secs(1)) => None,
                }
            })
    }

    #[test]
    fn socket_path_is_under_runtime_dir() {
        // SAFETY-ish: tests run multi-threaded, but no other test reads this
        // var concurrently; worst case the fallback branch is exercised.
        std::env::set_var("XDG_RUNTIME_DIR", "/tmp/opencode");
        let p = socket_path();
        assert!(p.starts_with("/tmp/opencode"));
        assert!(p.ends_with("songstress/mpv.sock"));
    }

    // --- Step 7a: eof_advance queue choreography (pure fn) -------------------

    fn item(id: &str, idx: usize) -> OrderItem {
        OrderItem {
            track_id: id.into(),
            path: format!("/{id}.mp3"),
            album_id: "al-1".into(),
            album_index: idx,
        }
    }

    fn order_of(ids: &[&str]) -> Vec<OrderItem> {
        ids.iter().enumerate().map(|(i, id)| item(id, i)).collect()
    }

    fn queued(ids: &[&str]) -> VecDeque<OrderItem> {
        ids.iter().enumerate().map(|(i, id)| item(id, i)).collect()
    }

    #[test]
    fn eof_promotes_queue_front_before_the_order() {
        let mut st = PlayState {
            order: order_of(&["a", "b", "c"]),
            queue: queued(&["q1"]),
            ..PlayState::default()
        };
        match eof_advance(&mut st) {
            Some(EofAction::Advance { top_up, pre }) => {
                assert_eq!(top_up, None, "mpv consumed a queue entry — no top-up");
                assert!(pre.is_empty());
            }
            other => panic!("expected Advance, got {other:?}"),
        }
        assert_eq!(st.index, 1);
        assert!(st.from_queue);
        assert!(st.queue.is_empty());
        assert_eq!(st.order[1].track_id, "q1", "promoted INTO the order");
        assert_eq!(st.order[2].track_id, "b", "order entries shifted, not replaced");
    }

    #[test]
    fn eof_consumes_promoted_entry_and_slides_the_order() {
        let mut st = PlayState {
            order: order_of(&["a", "q1", "b", "c"]),
            index: 1,
            from_queue: true,
            ..PlayState::default()
        };
        match eof_advance(&mut st) {
            Some(EofAction::Advance { .. }) => {}
            other => panic!("expected Advance, got {other:?}"),
        }
        assert_eq!(st.order.iter().map(|i| i.track_id.as_str()).collect::<Vec<_>>(), vec!["a", "b", "c"]);
        assert_eq!(st.index, 1, "index stays — the next order entry slid in");
        assert!(!st.from_queue);
        assert_eq!(st.current().unwrap().track_id, "b");
    }

    #[test]
    fn eof_queue_chain_plays_every_entry_then_resumes_the_order() {
        let mut st = PlayState {
            order: order_of(&["a", "b", "c"]),
            queue: queued(&["q1", "q2"]),
            ..PlayState::default()
        };
        // 1st eof: a → q1 (promote)
        eof_advance(&mut st);
        assert_eq!(st.current().unwrap().track_id, "q1");
        // 2nd eof: q1 → q2 (consume + promote at the same slot)
        eof_advance(&mut st);
        assert_eq!(st.current().unwrap().track_id, "q2");
        assert_eq!(st.order.iter().map(|i| i.track_id.as_str()).collect::<Vec<_>>(), vec!["a", "q2", "b", "c"]);
        assert!(st.from_queue);
        // 3rd eof: q2 → b (consume, queue empty, order resumes)
        eof_advance(&mut st);
        assert_eq!(st.current().unwrap().track_id, "b");
        assert!(!st.from_queue);
        assert_eq!(st.index, 1);
    }

    #[test]
    fn eof_topup_fires_when_the_order_window_slides() {
        // 40 tracks: after a normal advance the window needs order[index+31].
        let ids: Vec<String> = (0..40).map(|i| format!("t{i}")).collect();
        let refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        let mut st = PlayState { order: order_of(&refs), ..PlayState::default() };
        match eof_advance(&mut st) {
            Some(EofAction::Advance { top_up, pre }) => {
                assert_eq!(top_up.as_deref(), Some("/t32.mp3"), "index 1 + 31");
                assert!(pre.is_empty());
            }
            other => panic!("expected Advance, got {other:?}"),
        }
    }

    #[test]
    fn eof_promoting_the_last_queue_entry_prearms_repeat_album() {
        let mut st = PlayState {
            order: order_of(&["a"]),
            index: 0,
            repeat: RepeatStage::Album,
            queue: queued(&["q1"]),
            ..PlayState::default()
        };
        match eof_advance(&mut st) {
            Some(EofAction::Advance { pre, .. }) => {
                assert_eq!(pre, vec!["/a.mp3"], "wrap pass = the album, NOT the queue entry");
            }
            other => panic!("expected Advance, got {other:?}"),
        }
        assert!(st.wrapped.is_some());
        assert_eq!(
            st.wrapped.as_ref().unwrap().iter().map(|i| i.track_id.as_str()).collect::<Vec<_>>(),
            vec!["a"],
            "the promoted queue entry must not leak into the repeat pass"
        );
    }

    #[test]
    fn eof_consuming_the_last_queue_entry_promotes_the_wrapped_pass() {
        // 40-entry pass so the fresh-pass top-up (order[31]) exists.
        let ids: Vec<String> = (0..40).map(|i| format!("t{i}")).collect();
        let refs: Vec<&str> = ids.iter().map(|s| s.as_str()).collect();
        let mut st = PlayState {
            order: order_of(&["a", "q1"]),
            index: 1,
            from_queue: true,
            repeat: RepeatStage::Album,
            wrapped: Some(order_of(&refs)),
            ..PlayState::default()
        };
        match eof_advance(&mut st) {
            Some(EofAction::Advance { top_up, .. }) => {
                assert_eq!(top_up.as_deref(), Some("/t31.mp3"), "fresh pass top-up");
            }
            other => panic!("expected Advance, got {other:?}"),
        }
        assert_eq!(st.order.first().unwrap().track_id, "t0");
        assert_eq!(st.index, 0);
        assert!(!st.from_queue);
    }

    #[test]
    fn eof_without_queue_behaves_as_before_and_stop_clears_the_queue() {
        let mut st = PlayState { order: order_of(&["a", "b"]), ..PlayState::default() };
        match eof_advance(&mut st) {
            Some(EofAction::Advance { top_up, pre }) => {
                assert_eq!(top_up, None);
                assert!(pre.is_empty());
            }
            other => panic!("expected Advance, got {other:?}"),
        }
        assert_eq!(st.index, 1);
        assert!(!st.from_queue);
        // Last track, no repeat → the queued entry still plays first, then
        // Stop resets everything incl. the queue.
        let mut st = PlayState {
            order: order_of(&["a"]),
            queue: queued(&["q9"]),
            ..PlayState::default()
        };
        assert!(matches!(eof_advance(&mut st), Some(EofAction::Advance { .. })));
        assert_eq!(st.current().unwrap().track_id, "q9");
        assert!(matches!(eof_advance(&mut st), Some(EofAction::Stop)));
        assert!(st.queue.is_empty(), "stop clears the queue");
        assert!(!st.from_queue);
    }

    #[test]
    fn eof_with_queued_entries_never_stops_or_reloads() {
        // Even at the last order entry, queued entries come first.
        let mut st = PlayState {
            order: order_of(&["a"]),
            repeat: RepeatStage::Off,
            queue: queued(&["q1"]),
            ..PlayState::default()
        };
        assert!(matches!(eof_advance(&mut st), Some(EofAction::Advance { .. })));
        assert_eq!(st.current().unwrap().track_id, "q1");
    }
}
