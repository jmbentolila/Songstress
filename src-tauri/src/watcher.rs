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

use std::path::PathBuf;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::time::{Duration, Instant};

use notify::event::{EventKind, ModifyKind};
use notify::{Event, RecursiveMode, Watcher};

/// Quiet period: events must stop arriving this long before a rescan fires.
pub const DEBOUNCE: Duration = Duration::from_secs(2);

/// Poll tick for the stop channel + event drain.
const TICK: Duration = Duration::from_millis(50);

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
    matches!(
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
    )
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
                if let Some(t) = pending {
                    if t.elapsed() >= DEBOUNCE {
                        pending = None;
                        on_quiet();
                    }
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
            assert!(!marks_library_dirty(&Event::new(kind)), "{kind:?} must be noise");
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
            assert!(marks_library_dirty(&Event::new(kind)), "{kind:?} must count");
        }
    }

    /// Dropping the handle stops the thread and releases its watches.
    #[test]
    fn stop_releases_the_thread() {
        let dir = std::env::temp_dir().join(format!("songstress-watch-stop-{}", std::process::id()));
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
