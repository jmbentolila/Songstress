//! Live library watching (Step 7d): inotify via the `notify` crate.
//!
//! One recursive watcher per root on a dedicated thread; raw events are
//! debounced (fires only after a 2s quiet period), then a callback runs.
//! Only MUTATING event kinds count — see [`marks_library_dirty`], without that
//! filter the scanner's own directory reads feed the watcher forever.
//! The callback merely raises a DIRTY flag — the actual rescan happens on a
//! polling async loop in lib.rs that waits for SCAN_RUNNING to clear and
//! enforces a minimum interval, so bulk copies don't thrash.
//!
//! Step 7c: roots can change at runtime (music folders modal) — `spawn`
//! returns a handle; dropping/replacing it stops the old thread (its notify
//! watcher drops → watches release), so callers just re-spawn with the new
//! root list.
//!
//! inotify is local-FS only: network mounts (NFS/SMB) either fail at watch
//! time (logged, watching disabled) or never deliver events — the manual
//! Rescan remains the path there.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use notify::event::{EventKind, ModifyKind};
use notify::{Event, RecursiveMode, Watcher};

/// Quiet period: events must stop arriving this long before a rescan fires.
pub const DEBOUNCE: Duration = Duration::from_secs(2);

/// Poll tick for the stop channel + event drain.
const TICK: Duration = Duration::from_millis(50);

/// Paths our own save just moved, with the moment they landed. `save_imports`
/// registers its moved sources + destinations (+ the folders created for
/// them) here so the watcher does not echo them back ~2s later into a second
/// full scan + dump. Path-scoped AND time-scoped: an event counts unless ALL
/// of its paths are recently-saved ones, so an external change landing beside
/// (or in the same burst as) our move still rescans on schedule.
static RECENT_SAVES: OnceLock<Mutex<HashMap<PathBuf, Instant>>> = OnceLock::new();

/// How long a moved path stays silent: must cover the debounce (2s) + poll
/// tick (0.5s) + watcher min-interval (5s) with margin for scheduling jitter.
/// inotify delivers the move events in milliseconds, so this is generous —
/// and the cost of over-length is narrow: only a change to one of THESE paths
/// inside the window waits for its next change instead of rescanning now.
pub const SUPPRESS_WINDOW: Duration = Duration::from_secs(10);

/// Remember paths moved by our own save, starting (or refreshing) their
/// silence window. Must be called BEFORE the first write lands: the watcher
/// evaluates events at delivery, so registering after the moves (or between
/// them) loses the race against the very first write — the 2s debounce only
/// delays the rescan, never the evaluation. Expired entries are pruned on
/// the way in so the map can never grow past one save's worth of paths plus
/// one stale generation.
pub fn suppress_paths(paths: &[PathBuf]) {
    let now = Instant::now();
    let mut map = RECENT_SAVES
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap();
    map.retain(|_, t| now.duration_since(*t) < SUPPRESS_WINDOW);
    for p in paths {
        map.insert(p.clone(), now);
    }
}

/// Forget exactly the paths passed in, nothing wider: the error-path
/// counterpart to [`suppress_paths`]. A failed save calls this with the
/// vec it registered so no residual silence swallows a real change.
/// Entries a live save still needs stay put — removal is per-path, so an
/// overlapping save on a different folder keeps its own silence. (Two
/// saves sharing one folder clear the shared entry and fail open into
/// one extra scan; saves serialize through one command path, so that
/// edge is documented, not handled.)
pub fn unsuppress_paths(paths: &[PathBuf]) {
    let Some(lock) = RECENT_SAVES.get() else {
        return;
    };
    let mut map = lock.lock().unwrap();
    for p in paths {
        map.remove(p);
    }
}

/// Whether every path in this event is one our own save just moved (and still
/// inside its window). Folder-granular on the inside: a suppressed folder
/// covers the names within it, because the ` (N)` suffix a collision earns
/// is unknowable before the move — every name it can pick lives in the
/// destination folder. Empty path lists are unattributable and count — same
/// fail-open rule as the sidecar filter below.
fn is_own_save(paths: &[PathBuf]) -> bool {
    let Some(lock) = RECENT_SAVES.get() else {
        return false;
    };
    let mut map = lock.lock().unwrap();
    let now = Instant::now();
    map.retain(|_, t| now.duration_since(*t) < SUPPRESS_WINDOW);
    let fresh = |p: &Path| map.get(p).is_some_and(|t| now.duration_since(*t) < SUPPRESS_WINDOW);
    !paths.is_empty()
        && paths
            .iter()
            .all(|p| fresh(p) || p.parent().is_some_and(fresh))
}

/// Does this event mean the library CONTENT may have changed?
///
/// `EventKind::Access(_)` must be excluded. notify's inotify backend watches
/// `WatchMask::OPEN` too, so every `opendir`/`readdir`/`read` arrives as
/// `Access(Open)` / `Access(Close(Read))` — and the incremental scan reads the
/// whole tree. Counting those made the watcher feed on itself: scan → read
/// traffic → dirty → scan, forever, ~8k events per pass with zero actual
/// changes (live incident: 7723 pointless rescans in 24h, one every 5s = the
/// min-interval). Reads never change the library, so they are noise.
///
/// `Modify(Metadata(_))` stays IN: that is where notify maps `IN_ATTRIB`, and a
/// plain `touch` (the documented way to poke the watcher) lands there — the
/// incremental scan keys on mtime+size, so an attrib change is worth a pass.
/// `Any`/`Other` stay IN as the conservative catch-alls; neither is generated
/// by read traffic.
pub fn marks_library_dirty(ev: &Event) -> bool {
    let mutating = matches!(
        ev.kind,
        EventKind::Create(_)
            | EventKind::Remove(_)
            | EventKind::Modify(ModifyKind::Name(_))
            | EventKind::Modify(ModifyKind::Data(_))
            | EventKind::Modify(ModifyKind::Metadata(_))
            | EventKind::Modify(ModifyKind::Any)
            | EventKind::Modify(ModifyKind::Other)
            // Unknown/unsupported signals fail OPEN: a missed change means a
            // stale library, a false positive costs one ~60ms incremental
            // pass. Read traffic is the one thing that must never be in here.
            | EventKind::Any
            | EventKind::Other
    );
    if !mutating {
        return false;
    }
    // Our own `.songstress.json` is NOT library content. The scan writes them
    // (one per album, carrying the display state) and the app rewrites the one
    // you just styled, so counting them would re-arm the watcher on our own
    // writes — the same self-feeding bug read traffic caused (AGENTS.md). An
    // event with no path at all is unattributable, so it counts (fail open).
    let only_sidecars = !ev.paths.is_empty()
        && ev
            .paths
            .iter()
            .all(|p| crate::library::sidecar::is_sidecar_path(p));
    if only_sidecars {
        return false;
    }
    // Our own save just moved these paths (registered via `suppress_paths`):
    // the post-save rescan already indexed them, so counting them buys a
    // second full scan + dump for nothing.
    !is_own_save(&ev.paths)
}

/// Stopping the thread releases its watches (the notify watcher drops).
/// Drop to stop the watcher thread: the event channel disconnects, the
/// thread sees it and exits, and its inotify watches release with it.
/// (`rewatch` relies on exactly this — a dropped handle that kept its thread
/// alive would leak a full recursive watch per root change.)
pub struct WatchHandle {
    /// Only keeps the sender alive until the handle is dropped.
    #[allow(dead_code)]
    stop: Sender<()>,
}

/// Watch `roots` recursively; after every burst of activity goes quiet for
/// [`DEBOUNCE`], call `on_quiet`. Never panics; a failing inotify backend
/// just disables watching (logged to stderr).
pub fn spawn(roots: Vec<PathBuf>, on_quiet: impl Fn() + Send + 'static) -> WatchHandle {
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    std::thread::Builder::new()
        .name("library-watch".into())
        .spawn(move || {
            let (event_tx, event_rx) = mpsc::channel::<()>();
            let mut watcher = match notify::recommended_watcher(
                move |res: std::result::Result<notify::Event, notify::Error>| {
                    // create/remove/rename/data/attrib mean "the library may
                    // have changed" — the incremental scan sorts out what
                    // actually happened. Access events are OUR OWN read
                    // traffic and must not re-arm the loop (see the filter).
                    if res.as_ref().is_ok_and(marks_library_dirty) {
                        let _ = event_tx.send(());
                    }
                },
            ) {
                Ok(w) => w,
                Err(e) => {
                    eprintln!("[watch] inotify unavailable, live watching disabled: {e}");
                    return;
                }
            };
            for r in &roots {
                match watcher.watch(r, RecursiveMode::Recursive) {
                    Ok(()) => eprintln!("[watch] watching {}", r.display()),
                    Err(e) => eprintln!("[watch] cannot watch {}: {e}", r.display()),
                }
            }
            // The watcher must stay alive for events to flow (drop = unwatch).
            let mut pending: Option<Instant> = None;
            loop {
                match stop_rx.try_recv() {
                    // Handle dropped → channel disconnected → exit. This MUST
                    // also catch Disconnected: the sender lives in WatchHandle,
                    // so dropping it (rewatch) is the only stop signal.
                    Ok(()) | Err(TryRecvError::Disconnected) => return,
                    Err(TryRecvError::Empty) => {}
                }
                let mut got = false;
                loop {
                    match event_rx.try_recv() {
                        Ok(()) => got = true,
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => return,
                    }
                }
                if got {
                    pending = Some(Instant::now());
                }
                if let Some(t) = pending
                    && t.elapsed() >= DEBOUNCE {
                        pending = None;
                        on_quiet();
                    }
                std::thread::sleep(TICK);
            }
        })
        .expect("spawn library-watch thread");
    WatchHandle { stop: stop_tx }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn fires_after_a_write_and_debounces_a_burst() {
        let dir = std::env::temp_dir().join(format!("songstress-watch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (tx, rx) = mpsc::channel::<()>();
        let handle = spawn(vec![dir.clone()], move || {
            let _ = tx.send(());
        });
        // Give the thread a moment to arm its watches.
        std::thread::sleep(Duration::from_millis(300));
        // Burst: several quick writes must coalesce into ONE callback.
        for i in 0..3 {
            std::fs::write(dir.join(format!("t{i}.txt")), b"x").unwrap();
        }
        let first = rx.recv_timeout(Duration::from_secs(10));
        assert!(first.is_ok(), "watcher callback never fired");
        // The 2s quiet window means the 3 quick writes land inside ONE burst:
        // no second callback may have queued up by now.
        assert!(
            rx.recv_timeout(Duration::from_secs(3)).is_err(),
            "burst should debounce into a single callback"
        );
        drop(handle);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Regression test for the live feedback loop: the incremental scan reads
    /// every directory, notify reports that as Access(Open)/Access(Close(Read)),
    /// and counting those re-armed the watcher forever (one rescan every 5s,
    /// ~8k events per pass, zero changes). Reads must be silent — but the
    /// watcher must still be armed, so the test ends with a real write.
    #[test]
    fn read_traffic_does_not_rearm_the_watcher() {
        use std::io::Read;
        let dir =
            std::env::temp_dir().join(format!("songstress-watch-read-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        for i in 0..3 {
            std::fs::write(dir.join("sub").join(format!("t{i}.txt")), b"payload").unwrap();
        }
        let (tx, rx) = mpsc::channel::<()>();
        let handle = spawn(vec![dir.clone()], move || {
            let _ = tx.send(());
        });
        std::thread::sleep(Duration::from_millis(300));
        // Walk + read the tree the way walkdir/lofty do: opendir, readdir, read.
        for _ in 0..6 {
            for entry in std::fs::read_dir(&dir).unwrap() {
                let p = entry.unwrap().path();
                if p.is_dir() {
                    for inner in std::fs::read_dir(&p).unwrap() {
                        let mut buf = Vec::new();
                        std::fs::File::open(inner.unwrap().path())
                            .unwrap()
                            .read_to_end(&mut buf)
                            .unwrap();
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        assert!(
            rx.recv_timeout(Duration::from_secs(4)).is_err(),
            "reading the library must not mark it dirty (scanner feeds the watcher)"
        );
        // Control: the thread is alive and watching — a real write still fires.
        std::fs::write(dir.join("new.txt"), b"x").unwrap();
        assert!(
            rx.recv_timeout(Duration::from_secs(10)).is_ok(),
            "a real write must still fire after read traffic"
        );
        drop(handle);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn access_is_noise_everything_mutating_counts() {
        use notify::event::{
            AccessKind, AccessMode, CreateKind, DataChange, MetadataKind, RemoveKind, RenameMode,
        };
        for kind in [
            EventKind::Access(AccessKind::Open(AccessMode::Any)),
            EventKind::Access(AccessKind::Open(AccessMode::Read)),
            EventKind::Access(AccessKind::Close(AccessMode::Read)),
            EventKind::Access(AccessKind::Read),
        ] {
            assert!(
                !marks_library_dirty(&Event::new(kind)),
                "{kind:?} must be noise"
            );
        }
        for kind in [
            EventKind::Create(CreateKind::Any),
            EventKind::Remove(RemoveKind::Any),
            EventKind::Modify(ModifyKind::Name(RenameMode::Any)),
            EventKind::Modify(ModifyKind::Data(DataChange::Any)),
            // where notify maps IN_ATTRIB — a plain `touch` lands here
            EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)),
            EventKind::Any,
        ] {
            assert!(
                marks_library_dirty(&Event::new(kind)),
                "{kind:?} must count"
            );
        }
    }

    /// Our own display-state file must never re-arm the watcher: the scan
    /// writes one per album and the app rewrites the styled one, so counting
    /// them would schedule a rescan on every gradient edit — and the rescan
    /// would write again. Same self-feeding shape as the read-traffic bug.
    #[test]
    fn sidecar_writes_are_not_library_changes() {
        use crate::library::sidecar;
        use notify::event::{CreateKind, RenameMode};
        let folder = std::path::Path::new("/music/Ghost/Meliora (2015)");
        let mut ev = Event::new(EventKind::Create(CreateKind::Any));
        ev.paths = vec![sidecar::path_in(folder)];
        assert!(!marks_library_dirty(&ev), "our file is not library content");

        // The temp of an atomic write, and the rename that lands it.
        let mut tmp = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Any)));
        tmp.paths = vec![
            folder.join(".songstress.json.tmp-1234"),
            sidecar::path_in(folder),
        ];
        assert!(!marks_library_dirty(&tmp));

        // A real file next to it still counts — including in the same event.
        let mut mixed = Event::new(EventKind::Create(CreateKind::Any));
        mixed.paths = vec![sidecar::path_in(folder), folder.join("01 - Spirit.flac")];
        assert!(marks_library_dirty(&mixed));
    }

    /// Our own save must not echo — and the silence must start BEFORE the
    /// first write, because the watcher evaluates at delivery: registering
    /// after the moves loses the race no debounce can cover (measured live:
    /// an end-of-save registration still echoed). Folder-granular inside:
    /// a suppressed folder covers the collision-suffix names within it,
    /// while anything whose parent is not suppressed still counts. No sleeps
    /// here: expiry is covered by the retain logic, and the window itself is
    /// long (SUPPRESS_WINDOW) precisely so a test never has to race it.
    #[test]
    fn own_save_is_silent_per_path_with_folder_cover() {
        use notify::event::CreateKind;
        let dir = std::path::PathBuf::from("/music/Ghost/Impera");
        let moved = dir.join("99 - Probe.flac");
        let sibling = dir.join("02 - Real.flac");
        let other = std::path::PathBuf::from("/music/Other/track.mp3");
        let ev = |p: std::path::PathBuf| {
            let mut e = Event::new(EventKind::Create(CreateKind::File));
            e.paths = vec![p];
            e
        };
        assert!(
            marks_library_dirty(&ev(moved.clone())),
            "unsuppressed create must count"
        );
        suppress_paths(&[moved.clone(), dir.clone()]);
        assert!(
            !marks_library_dirty(&ev(moved)),
            "our own move must not echo"
        );
        assert!(
            !marks_library_dirty(&ev(sibling)),
            "a name the collision walk could have picked is covered by its folder"
        );
        assert!(
            marks_library_dirty(&ev(other.clone())),
            "other folders still count"
        );
        let mut mixed = Event::new(EventKind::Create(CreateKind::File));
        mixed.paths = vec![dir.join("x.flac"), other];
        assert!(
            marks_library_dirty(&mixed),
            "an external path in the burst still counts"
        );
    }

    /// B1 AC2: with only landed paths suppressed, an event in a
    /// predicted-but-unlanded folder counts — including a sibling-name path
    /// of the kind the folder-cover rule would have hidden had that folder
    /// been registered. Touches only the suppression-set input, never the
    /// `is_own_save` rule itself.
    #[test]
    fn b1_unlanded_folder_counts_with_landed_only_suppression() {
        use notify::event::CreateKind;
        let base =
            std::env::temp_dir().join(format!("songstress-b1-ac2-{}", std::process::id()));
        let landed_dir = base.join("Landed Album");
        let landed_file = landed_dir.join("01 - Landed.flac");
        let unlanded_dir = base.join("Predicted Album");
        // B1-shaped set: only what a save actually landed.
        suppress_paths(&[landed_file.clone(), landed_dir.clone()]);
        let ev = |p: std::path::PathBuf| {
            let mut e = Event::new(EventKind::Create(CreateKind::File));
            e.paths = vec![p];
            e
        };
        assert!(
            marks_library_dirty(&ev(unlanded_dir.join("external.mp3"))),
            "AC2: an event in a predicted-but-unlanded folder counts"
        );
        assert!(
            marks_library_dirty(&ev(unlanded_dir.join("02 - Real.flac"))),
            "AC2 via the folder-cover path: the parent was never registered"
        );
        // Coda: this sibling-name path IS one the cover hides — registering
        // its folder flips it silent, proving the probe above ran through the
        // cover rule and not around it.
        suppress_paths(&[unlanded_dir.clone()]);
        assert!(
            !marks_library_dirty(&ev(unlanded_dir.join("02 - Real.flac"))),
            "cover sanity: registering the folder covers its names"
        );
    }

    /// B1 AC3: after a B1-shaped save, a sibling of a LANDED file is still
    /// silent — the narrowed set must not regress echo suppression. The set
    /// mirrors the ancestor rule (folder plus its parent), and the existing
    /// `own_save_is_silent_per_path_with_folder_cover` stays unmodified.
    #[test]
    fn b1_landed_sibling_stays_silent() {
        use notify::event::CreateKind;
        let base =
            std::env::temp_dir().join(format!("songstress-b1-ac3-{}", std::process::id()));
        let parent = base.join("Coll").join("B1ArtistA");
        let landed_dir = parent.join("B1AlbumA");
        let landed_file = landed_dir.join("01 - Landed.flac");
        let elsewhere = base.join("Elsewhere");
        suppress_paths(&[landed_file.clone(), landed_dir.clone(), parent.clone()]);
        let ev = |p: std::path::PathBuf| {
            let mut e = Event::new(EventKind::Create(CreateKind::File));
            e.paths = vec![p];
            e
        };
        assert!(
            !marks_library_dirty(&ev(landed_file)),
            "our own landed file must not echo"
        );
        assert!(
            !marks_library_dirty(&ev(landed_dir.join("02 - Real.flac"))),
            "AC3: a sibling of the landed file stays silent (cover applies)"
        );
        assert!(
            marks_library_dirty(&ev(elsewhere.join("other.mp3"))),
            "unsuppressed folders still count"
        );
    }

    /// B2 unit probe: suppress then unsuppress the same set and the folder
    /// counts again — the primitive clears exactly what it was given,
    /// nothing wider. A second, unrelated registration survives the
    /// clearing, proving removal is per-path.
    #[test]
    fn b2_unsuppress_clears_exactly_the_given_set() {
        use notify::event::CreateKind;
        let base =
            std::env::temp_dir().join(format!("songstress-b2-unsuppress-{}", std::process::id()));
        let failed_dir = base.join("Failed Album");
        let failed_file = failed_dir.join("01 - Failed.flac");
        let other_dir = base.join("Other Album");
        let other_file = other_dir.join("01 - Other.flac");
        let failed_set = vec![failed_file.clone(), failed_dir.clone()];
        suppress_paths(&failed_set);
        suppress_paths(&[other_file.clone(), other_dir.clone()]);
        let ev = |p: std::path::PathBuf| {
            let mut e = Event::new(EventKind::Create(CreateKind::File));
            e.paths = vec![p];
            e
        };
        assert!(
            !marks_library_dirty(&ev(failed_dir.join("02 - Real.flac"))),
            "sanity: the folder is silent while registered"
        );
        unsuppress_paths(&failed_set);
        assert!(
            marks_library_dirty(&ev(failed_dir.join("external.mp3"))),
            "B2 unit: after clearing, an event in that folder counts"
        );
        assert!(
            marks_library_dirty(&ev(failed_file)),
            "B2 unit: the cleared file itself counts too"
        );
        assert!(
            !marks_library_dirty(&ev(other_dir.join("02 - Real.flac"))),
            "exactness: an unrelated registration survives the clearing"
        );
    }

    /// Dropping the handle stops the thread and releases its watches.
    #[test]
    fn stop_releases_the_thread() {
        let dir =
            std::env::temp_dir().join(format!("songstress-watch-stop-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (tx, rx) = mpsc::channel::<()>();
        let handle = spawn(vec![dir.clone()], move || {
            let _ = tx.send(());
        });
        drop(handle);
        // After drop, writes must NOT reach the (dead) callback channel.
        std::thread::sleep(Duration::from_millis(300));
        std::fs::write(dir.join("late.txt"), b"x").unwrap();
        assert!(
            rx.recv_timeout(Duration::from_secs(3)).is_err(),
            "stopped watcher must not fire"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
