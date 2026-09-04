// Remember the window's SIZE across launches — size only, never position.
//
// Why size alone (owner ruling, 2026-09-05; the plugin attempt of 2026-09-01
// is measured history in PLAN.md): a Wayland client cannot see where the
// compositor put it — Tauri's `outer_position()` reports (0,0) while KWin
// knows the truth — so any saved position is a lie that would shove the
// window into a corner on restore. The owner places the window himself
// (bottom-left quarter of his grid) as part of the restart protocol. Size,
// however, IS knowable (`inner_size()` round-trips), and remembering it is
// the difference between one drag to tile and a full resize every launch.
//
// Why hand-rolled and not tauri-plugin-window-state: the plugin writes on
// RunEvent::Exit only, and `systemctl restart songstress-dev` / a `tauri dev`
// rebuild kill the process before any exit handler runs (verified 2026-09-01
// — three rebuild-kills, no state file). Writing on every Resized event is
// a few-hundred-byte tempfile+rename into the app config dir, which costs
// nothing and survives SIGTERM: the file always holds the last size the
// compositor agreed to, so a kill before "close" loses at most an ongoing
// drag.
//
// The file lives in the app CONFIG dir (survives cache clears; this is user
// preference, not cache).

use std::path::PathBuf;

use tauri::{AppHandle, Manager, PhysicalSize};

/// Smallest restored size worth honoring — a corrupt or absurd value must
/// not boot the app into an unusable sliver (the shell breaks below this).
const MIN_LOGICAL: u32 = 600;
const MIN_LOGICAL_H: u32 = 400;

#[derive(serde::Serialize, serde::Deserialize)]
struct SavedSize {
    width: u32,
    height: u32,
}

fn file(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("window-size.json"))
}

/// The size the last session ended at, in logical px. Any failure (no
/// file, bad JSON, silly numbers) is simply "no memory": launch defaults.
pub fn load(app: &AppHandle) -> Option<(u32, u32)> {
    let raw = std::fs::read_to_string(file(app)?).ok()?;
    let saved: SavedSize = serde_json::from_str(&raw).ok()?;
    if saved.width >= MIN_LOGICAL && saved.height >= MIN_LOGICAL_H {
        Some((saved.width, saved.height))
    } else {
        None
    }
}

/// Persist the window's current size (physical px → logical, the same
/// units the restore side consumes). tempfile+rename so a kill mid-write
/// cannot truncate the only copy.
pub fn save(app: &AppHandle, size: PhysicalSize<u32>) {
    let Some(win) = app.get_webview_window("main") else {
        return;
    };
    let scale = win.scale_factor().unwrap_or(1.0).max(1.0);
    let saved = SavedSize {
        width: (size.width as f64 / scale).round() as u32,
        height: (size.height as f64 / scale).round() as u32,
    };
    let Some(path) = file(app) else { return };
    let Ok(json) = serde_json::to_string(&saved) else {
        return;
    };
    let tmp = path.with_extension("json.tmp");
    if std::fs::create_dir_all(&path.parent().expect("config dir has a parent")).is_err() {
        return;
    }
    if std::fs::write(&tmp, json).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}
