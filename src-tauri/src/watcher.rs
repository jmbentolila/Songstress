//! Live library watching (Step 7d): inotify via the `notify` crate.
//!
//! One recursive watcher per root on a dedicated thread; raw events are
//! debounced (fires only after a 2s quiet period), then a callback runs.
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

use notify::{RecursiveMode, Watcher};

/// Quiet period: events must stop arriving this long before a rescan fires.
pub const DEBOUNCE: Duration = Duration::from_secs(2);

/// Poll tick for the stop channel + event drain.
const TICK: Duration = Duration::from_millis(50);

/// Stopping the thread releases its watches (the notify watcher drops).
pub struct WatchHandle {
    stop: Sender<()>,
}

impl WatchHandle {
    pub fn stop(&self) {
        let _ = self.stop.send(());
    }
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
                    // Any event kind counts: create/modify/remove/rename all
                    // mean "the library may have changed" — the incremental
                    // scan sorts out what actually happened.
                    if res.is_ok() {
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
                if stop_rx.try_recv().is_ok() {
                    return;
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
        handle.stop();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stop_releases_the_thread() {
        let dir = std::env::temp_dir().join(format!("songstress-watch-stop-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (tx, rx) = mpsc::channel::<()>();
        let handle = spawn(vec![dir.clone()], move || {
            let _ = tx.send(());
        });
        handle.stop();
        // After stop, writes must NOT reach the (dead) callback channel.
        std::thread::sleep(Duration::from_millis(300));
        std::fs::write(dir.join("late.txt"), b"x").unwrap();
        assert!(
            rx.recv_timeout(Duration::from_secs(3)).is_err(),
            "stopped watcher must not fire"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
