//! Screen color picker, mark II: our OWN dropper instead of the compositor's
//! `Screenshot.PickColor` crosshair (Step 9b's original). Mutter's crosshair
//! has no visible hotspot, so aiming it is a guess — and its look is
//! compositor-owned, unchangeable from here.
//!
//! Flow: `begin_screen_pick` screenshots the screen through the portal
//! (non-interactive, so no compositor UI appears), holds the PNG in memory,
//! and opens a fullscreen overlay window (`index.html#screen-pick`) that
//! renders the shot with a standard crosshair cursor plus a magnifier loupe;
//! a click lifts the pixel, Escape/right-click cancels. `finish_screen_pick`
//! reports the hex (or null) to the main window and tears the overlay down.
//!
//! Known limit: the shot can span monitors while the overlay covers one, so
//! on multi-monitor setups the loupe maps the overlay's monitor exactly and
//! anywhere else approximately. No screenshot files touch the library — the
//! bytes live in memory and die with the session.

use std::collections::HashMap;
use std::sync::Mutex;

use base64::Engine as _;
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter, Manager};
use zbus::proxy;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

#[proxy(
    interface = "org.freedesktop.portal.Screenshot",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait ScreenshotPortal {
    /// Takes the shot; the returned path is the Request to wait on.
    fn screenshot(
        &self,
        parent_window: &str,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.portal.Request",
    default_service = "org.freedesktop.portal.Desktop"
)]
trait PortalRequest {
    /// (response code, results). 0 = shot taken (`results["uri"]` is a
    /// `file://` URI); 1 = cancelled; 2 = failed.
    #[zbus(signal)]
    fn response(&self, response: u32, results: HashMap<String, OwnedValue>) -> zbus::Result<()>;
}

/// The in-flight shot, held between `begin_screen_pick` (main window) and
/// `take_pick_image` (overlay window). Taken, never replaced: a second call
/// while one is open just refocuses the overlay.
static SESSION: Mutex<Option<Vec<u8>>> = Mutex::new(None);

async fn screenshot_png() -> Result<Vec<u8>, String> {
    let conn = zbus::Connection::session()
        .await
        .map_err(|e| format!("could not reach the session bus: {e}"))?;
    let portal = ScreenshotPortalProxy::new(&conn)
        .await
        .map_err(|e| format!("screenshot unavailable: {e}"))?;
    let mut options: HashMap<&str, Value<'_>> = HashMap::new();
    options.insert(
        "handle_token",
        Value::from(format!("songstress_shot_{}", std::process::id())),
    );
    // Non-interactive: no compositor screenshot UI, just the pixels. An
    // empty parent is legal and keeps this working on Wayland and X11.
    options.insert("modal", Value::from(false));
    options.insert("interactive", Value::from(false));
    let request = portal
        .screenshot("", options)
        .await
        .map_err(|e| format!("screenshot failed to start: {e}"))?;
    let req = PortalRequestProxy::builder(&conn)
        .path(request.clone())
        .map_err(|e| format!("bad screenshot request path: {e}"))?
        .build()
        .await
        .map_err(|e| format!("screenshot failed: {e}"))?;
    let mut stream = req
        .receive_response()
        .await
        .map_err(|e| format!("screenshot failed: {e}"))?;
    let signal = stream
        .next()
        .await
        .ok_or_else(|| "screenshot closed without answering".to_string())?;
    let args = signal
        .args()
        .map_err(|e| format!("unreadable screenshot answer: {e}"))?;
    if args.response != 0 {
        return Err("screenshot was cancelled".to_string());
    }
    let uri: String = args
        .results
        .get("uri")
        .and_then(|v| String::try_from(v.clone()).ok())
        .ok_or_else(|| "screenshot answered without a URI".to_string())?;
    // Same file-URI decoding the file pickers use (portal_files.rs) — the
    // shot lands wherever the compositor puts it, and only HERE is it read.
    let path = crate::portal_files::uri_to_path(&uri)
        .ok_or_else(|| "screenshot URI was not a local file".to_string())?;
    std::fs::read(&path).map_err(|e| format!("could not read screenshot: {e}"))
}

/// Screenshot, stash the bytes, raise the overlay. `Ok` means the overlay
/// is up (the shot is already taken); the picked hex arrives later as a
/// `screen-picked` event on the main window — null when cancelled.
#[tauri::command]
pub async fn begin_screen_pick(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("screen-pick") {
        let _ = win.set_focus();
        return Ok(());
    }
    let png = screenshot_png().await?;
    *SESSION
        .lock()
        .map_err(|_| "screen pick session is poisoned".to_string())? = Some(png);
    tauri::WebviewWindowBuilder::new(
        &app,
        "screen-pick",
        tauri::WebviewUrl::App("index.html#screen-pick".into()),
    )
    .title("Pick a screen color")
    .fullscreen(true)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .build()
    .map_err(|e| format!("picker overlay failed to open: {e}"))?;
    Ok(())
}

/// Hand the stashed shot to the overlay as a data URL. Kept (not taken):
/// a reload or HMR re-mount must not lose the session.
#[tauri::command]
pub async fn take_pick_image() -> Result<String, String> {
    let session = SESSION
        .lock()
        .map_err(|_| "screen pick session is poisoned".to_string())?;
    let png = session
        .as_ref()
        .ok_or_else(|| "no screen pick in flight".to_string())?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    ))
}

/// Report the pick (hex `rrggbb`, or `None` = cancelled) to the main window
/// and tear the overlay down. Cancellation is silence, as with every picker.
#[tauri::command]
pub async fn finish_screen_pick(app: AppHandle, hex: Option<String>) -> Result<(), String> {
    *SESSION
        .lock()
        .map_err(|e| format!("screen pick session is poisoned: {e}"))? = None;
    let _ = app.emit("screen-picked", &hex);
    if let Some(win) = app.get_webview_window("screen-pick") {
        let _ = win.destroy();
    }
    Ok(())
}
